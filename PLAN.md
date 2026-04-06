# 国冰财务管理系统 - MVP 实施计划

## 项目概述
从零开始构建一个基于 Rust + SurrealDB 的个人财务管理 CLI 工具，支持自然语言记账。

---

## 阶段一：项目初始化与基础架构

### 任务 1.1：创建 Cargo 项目配置
**目标**：初始化 Rust 项目并配置依赖

**文件**：`Cargo.toml`

**依赖清单**：
- `clap` - CLI 框架（v4.x，features: ["derive"])
- `surrealdb` - 嵌入式数据库（features: ["kv-mem"] 或 ["kv-rocksdb"])
- `serde` - 序列化（features: ["derive"])
- `chrono` - 时间处理
- `rust_decimal` - 精确金额计算
- `uuid` - 唯一标识符（features: ["v4"])
- `tokio` - 异步运行时（features: ["full"])
- `anyhow` - 错误处理
- `dirs` - 获取系统目录路径

---

### 任务 1.2：创建目录结构

```
src/
├── main.rs              # CLI 入口
├── lib.rs               # 库入口
├── models/              # 数据模型
│   ├── mod.rs
│   ├── transaction.rs   # 交易记录
│   └── types.rs         # 枚举类型 (TxType, TxSource)
├── db/                  # 数据库层
│   └── surreal.rs       # SurrealDB 封装
├── parser/              # AI 解析层
│   └── natural_language.rs
├── commands/            # CLI 子命令
│   ├── mod.rs
│   ├── add.rs
│   ├── list.rs
│   └── stats.rs
└── error.rs             # 错误处理

migrations/
└── init.surql           # 数据库初始化脚本
```

---

## 阶段二：数据模型实现

### 任务 2.1：定义枚举类型
**文件**：`src/models/types.rs`

定义：
- `TxType` - 交易类型（Expense/Income/Transfer/DebtChange/CreditChange）
- `TxSource` - 数据来源（AiParsed/CsvImport/Manual）

实现 `Display` 和 `FromStr` trait 用于字符串解析。

---

### 任务 2.2：实现 Transaction 结构体
**文件**：`src/models/transaction.rs`

字段：
```rust
pub struct Transaction {
    pub id: Uuid,
    pub timestamp: DateTime<Local>,
    pub amount: Decimal,
    pub currency: String,
    pub tx_type: TxType,
    pub account_from: String,
    pub account_to: Option<String>,
    pub category: String,
    pub description: Option<String>,
    pub tags: Vec<String>,
    pub metadata: Option<Value>,
    pub created_at: DateTime<Local>,
    pub updated_at: Option<DateTime<Local>>,
    pub source: TxSource,
}
```

实现 `Default` 用于创建新交易。

---

## 阶段三：数据库层实现

### 任务 3.1：SurrealDB 封装
**文件**：`src/db/surreal.rs`

功能：
1. `Database::new()` - 初始化数据库连接
2. `Database::init()` - 执行初始化脚本
3. `Database::create_transaction(&self, tx: Transaction)` - 创建交易
4. `Database::list_transactions(&self, limit: usize)` - 列出交易
5. `Database::query_by_date_range(&self, from: DateTime, to: DateTime)` - 日期范围查询
6. `Database::query_by_category(&self, category: &str)` - 分类查询
7. `Database::get_monthly_stats(&self, year: i32, month: u32)` - 月度统计

存储路径：`~/.local/share/finance-cli/data.db`

---

### 任务 3.2：数据库初始化脚本
**文件**：`migrations/init.surql`

内容：
- 定义 `transaction` 表
- 定义所有字段类型
- 创建索引（timestamp, category, account_from, tx_type）

---

## 阶段四：AI 解析层

### 任务 4.1：自然语言解析器
**文件**：`src/parser/natural_language.rs`

功能：
1. `ParsedTransaction` 结构体 - 解析结果
2. `parse(input: &str) -> ParsedTransaction` - 解析函数

解析规则（简化版规则引擎）：
- 金额识别：正则匹配数字（支持 "35"、"35.5"、"35元"）
- 交易类型推断：
  - 关键词"收"、"工资"、"退款" → Income
  - 关键词"转"、"转到" → Transfer
  - 关键词"借" + "给" → CreditChange
  - 关键词"借" + "从" → DebtChange
  - 默认 → Expense
- 账户识别：
  - 支付关键词"用"、"从"后的词 → account_from
  - 收款方/商户 → account_to
- 分类映射表（简化）：
  - 关键词匹配预定义分类（餐饮、交通、工资等）

输出结构：
```rust
pub struct ParsedTransaction {
    pub amount: Decimal,
    pub currency: String,
    pub tx_type: TxType,
    pub account_from: String,
    pub account_to: Option<String>,
    pub category: String,
    pub description: String,
}
```

---

## 阶段五：CLI 命令实现

### 任务 5.1：命令结构定义
**文件**：`src/commands/mod.rs`

使用 `clap` derive 宏定义：
```rust
#[derive(Parser)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Commands,
}

pub enum Commands {
    Add(AddArgs),
    List(ListArgs),
    Stats(StatsArgs),
}
```

---

### 任务 5.2：Add 命令
**文件**：`src/commands/add.rs`

功能：
1. 接收自然语言输入
2. 调用解析器生成 ParsedTransaction
3. 展示解析结果，询问用户确认
4. 确认后存入数据库
5. 输出成功/失败信息

CLI 格式：
```bash
finance add "午餐35支付宝"
finance add "收到工资8500招行卡"
```

---

### 任务 5.3：List 命令
**文件**：`src/commands/list.rs`

参数：
- `--limit` / `-n`: 显示条数（默认 20）
- `--from`: 起始日期
- `--to`: 结束日期
- `--category`: 分类筛选

输出格式：表格形式展示交易记录

---

### 任务 5.4：Stats 命令
**文件**：`src/commands/stats.rs`

参数：
- `--month` / `-m`: 月份（如 2025-04）
- `--category`: 按分类统计
- `--from` / `--to`: 日期范围

功能：
1. 月度收支汇总
2. 分类占比统计
3. 输出格式：文本表格

---

## 阶段六：主入口与错误处理

### 任务 6.1：错误类型定义
**文件**：`src/error.rs`

定义 `FinanceError` 枚举：
- DatabaseError
- ParseError
- ValidationError
- IoError

实现 `From` trait 进行错误转换。

---

### 任务 6.2：主函数实现
**文件**：`src/main.rs`

流程：
1. 确保数据目录存在（`~/.local/share/finance-cli/`）
2. 初始化数据库连接
3. 解析 CLI 参数
4. 匹配子命令并执行
5. 错误处理与输出

---

## 阶段七：测试与验证

### 任务 7.1：单元测试
- 测试解析器各种输入
- 测试数据库 CRUD 操作
- 测试命令行参数解析

### 任务 7.2：集成测试
- 完整流程测试：add → list → stats

### 任务 7.3：构建验证
```bash
cargo build --release
cargo test
```

---

## 实施顺序

```
1.1 → 1.2 → 2.1 → 2.2 → 3.2 → 3.1 → 6.1 → 6.2 → 4.1 → 5.1 → 5.2 → 5.3 → 5.4 → 7.x
```

---

## 依赖安装检查

执行前请确认已安装：
- Rust 工具链（1.70+）
- Cargo

---

## 风险评估

| 风险 | 可能性 | 缓解措施 |
|------|--------|----------|
| SurrealDB 嵌入式模式配置复杂 | 中 | 使用 kv-mem 模式先验证，再迁移到 kv-rocksdb |
| 自然语言解析准确率 | 高 | MVP 使用简化规则引擎，后续迭代改进 |
| Decimal 序列化问题 | 低 | 使用 serde 特性确保 SurrealDB 兼容 |
