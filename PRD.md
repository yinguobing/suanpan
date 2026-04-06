# 国冰财务管理系统 PRD

## 1. 产品概述

### 1.1 产品定位
高度私人化的本地财务管理系统，以 AI 助手为核心交互入口，实现自然语言记账、多源数据整合和智能财务分析。

### 1.2 核心理念
- **AI 优先**：人类通过自然语言与系统交互，AI 负责解析和结构化
- **资金流动为核心**：所有财务活动抽象为资金的流动，简化数据模型
- **渐进式演进**：分阶段迭代，MVP 聚焦核心记账和查询功能

---

## 2. 目标用户

**主要用户**：尹国冰
- 需要长期、持续的财务记录
- 偏好自然语言交互
- 重视数据隐私和本地控制
- 有技术背景，可进行二次开发

---

## 3. 阶段规划

### 阶段一：MVP（当前）
目标：建立基础记账能力，验证核心交互流程

**功能范围：**
- ✅ 自然语言录入（AI 解析）
- ✅ 流水记录存储与查询
- ✅ 基础统计（月度汇总、分类占比）
- ✅ CLI 界面

**不包含：**
- ❌ 银行 CSV 自动导入
- ❌ 复杂报表和可视化
- ❌ GUI 界面
- ❌ 多账本管理

### 阶段二：数据整合
- 银行/支付宝/微信 CSV 导入适配
- 数据清洗和去重
- 历史数据迁移

### 阶段三：智能分析
- 财务健康度评估
- 异常消费提醒
- 预算管理
- 可视化报表

---

## 4. 技术架构

### 4.1 技术栈
| 层级 | 选型 | 理由 |
|------|------|------|
| 编程语言 | Rust | 类型安全、性能优异、Decimal 精度保障 |
| 数据库 | SurrealDB | 嵌入式模式、现代 API、无需手写 SQL、支持多模型查询 |
| CLI 框架 | clap | Rust 生态标准 |
| 序列化 | serde | 与 SurrealDB 原生集成 |
| 时间处理 | chrono | Rust 标准 |
| 金额计算 | rust_decimal | 避免浮点误差 |

### 4.2 项目结构
```
finance-cli/
├── Cargo.toml
├── src/
│   ├── main.rs              # CLI 入口
│   ├── lib.rs
│   ├── models/              # 数据模型
│   │   ├── mod.rs
│   │   ├── transaction.rs   # 交易记录
│   │   ├── category.rs      # 分类定义
│   │   └── account.rs       # 账户定义
│   ├── db/                  # 数据库层
│   │   └── surreal.rs       # SurrealDB 封装
│   ├── parser/              # AI 解析层
│   │   └── natural_language.rs
│   ├── commands/            # CLI 子命令
│   │   ├── add.rs
│   │   ├── list.rs
│   │   └── stats.rs
│   └── error.rs             # 错误处理
└── migrations/
    └── init.surql           # 数据库初始化
```

---

## 5. 数据模型

### 5.1 交易记录（Transaction）

```rust
/// 交易类型
enum TxType {
    Expense,        // 支出
    Income,         // 收入
    Transfer,       // 转账（账户间）
    DebtChange,     // 债务变动（借入/偿还债务）
    CreditChange,   // 债权变动（借出/收回借款）
}

/// 数据来源
enum TxSource {
    AiParsed,       // AI 解析自然语言
    CsvImport,      // 银行账单导入
    Manual,         // 手动录入
}

/// 交易记录
struct Transaction {
    id: Uuid,                      // 全局唯一标识
    timestamp: DateTime<Local>,    // 交易发生时间
    amount: Decimal,               // 金额（正数，Decimal 精度）
    currency: String,              // 货币代码：CNY, USD...
    tx_type: TxType,               // 交易类型
    
    // 复式记账核心
    account_from: String,          // 来源账户
    account_to: Option<String>,    // 去向账户/商户/收入方
    
    category: String,              // 分类（受控词表）
    description: Option<String>,   // 原始自然语言或备注
    
    // 扩展字段
    tags: Vec<String>,             // 标签（自由输入）
    metadata: Option<Value>,       // 任意扩展数据（JSON）
    
    // 系统字段
    created_at: DateTime<Local>,   // 记录创建时间
    updated_at: Option<DateTime<Local>>, // 修改时间
    source: TxSource,              // 数据来源
}
```

### 5.2 字段设计说明

#### 金额设计
- 使用 `rust_decimal::Decimal` 存储
- 避免浮点数精度问题
- 支持多币种（日元无小数、科威特第纳尔 3 位小数）

#### 账户设计（阶段一简化版）
- `account_from` 和 `account_to` 使用 `String`
- 不进行严格的外键约束
- AI 解析时自动创建新账户
- 阶段二再考虑账户管理和命名规范化

#### 分类、标签、元数据三层设计

| 字段 | 性质 | 用途 | 示例 |
|------|------|------|------|
| **category** | 单值、必填、受控词表 | 标准财务分类，用于常规统计 | `餐饮` / `交通` / `工资` |
| **tags** | 多值、可选、自由输入 | 跨分类的临时标记 | `旅游` / `2026-Q1` / `家人` |
| **metadata** | JSON、可选、机器数据 | 原始数据保留，系统集成 | `{"confidence": 0.95, "raw_input": "..."}` |

### 5.3 交易类型详解

#### Expense（支出）
资金从你的账户流向外部。
- `account_from`: 你的账户（支付宝/现金/招行卡）
- `account_to`: 商户/收款方（麦当劳/超市/房东）

#### Income（收入）
外部资金流入你的账户。
- `account_from`: 付款方（公司/客户/理财）
- `account_to`: 你的账户（招行卡/支付宝）

#### Transfer（转账）
资金在你的账户之间流动。
- `account_from`: 来源账户（招行卡）
- `account_to`: 目标账户（支付宝/招招理财）

#### DebtChange（债务变动）
**你欠别人的钱**发生变化。
- 借入：`account_from: "朋友-小明"`, `account_to: "现金"`
- 偿还：`account_from: "现金"`, `account_to: "朋友-小明"`

#### CreditChange（债权变动）
**别人欠你的钱**发生变化。
- 借出：`account_from: "现金"`, `account_to: "朋友-小明"`
- 收回：`account_from: "朋友-小明"`, `account_to: "现金"`

---

## 6. 交互设计

### 6.1 自然语言输入示例

用户输入 → AI 解析 → 确认 → 存储

```
用户：中午食堂吃饭花了35，用支付宝付的

AI 解析：
- amount: 35.00
- currency: CNY
- tx_type: Expense
- account_from: "支付宝"
- account_to: "食堂"
- category: "餐饮"
- description: "中午食堂吃饭"

AI 回复：确认记录：餐饮支出 ¥35.00，支付宝 → 食堂。确认？[Y/n]
```

### 6.2 CLI 命令

```bash
# 添加流水（自然语言）
finance add "午餐35支付宝"

# 列出最近流水
finance list --limit 20

# 按月统计
finance stats --month 2025-04

# 按分类统计
finance stats --category 餐饮 --from 2025-01-01 --to 2025-04-30
```

---

## 7. 数据存储

### 7.1 存储位置
- **默认路径**: `~/.local/share/finance-cli/`
- **数据库文件**: `data.db`
- **配置文件**: `config.toml`

### 7.2 SurrealDB 表结构

```surql
-- 交易记录表
DEFINE TABLE transaction SCHEMAFULL;

DEFINE FIELD id ON transaction TYPE record;
DEFINE FIELD timestamp ON transaction TYPE datetime;
DEFINE FIELD amount ON transaction TYPE decimal;
DEFINE FIELD currency ON transaction TYPE string;
DEFINE FIELD tx_type ON transaction TYPE string;
DEFINE FIELD account_from ON transaction TYPE string;
DEFINE FIELD account_to ON transaction TYPE option<string>;
DEFINE FIELD category ON transaction TYPE string;
DEFINE FIELD description ON transaction TYPE option<string>;
DEFINE FIELD tags ON transaction TYPE array<string>;
DEFINE FIELD metadata ON transaction TYPE option<object>;
DEFINE FIELD created_at ON transaction TYPE datetime;
DEFINE FIELD updated_at ON transaction TYPE option<datetime>;
DEFINE FIELD source ON transaction TYPE string;

-- 索引
DEFINE INDEX idx_timestamp ON transaction COLUMNS timestamp;
DEFINE INDEX idx_category ON transaction COLUMNS category;
DEFINE INDEX idx_account_from ON transaction COLUMNS account_from;
DEFINE INDEX idx_tx_type ON transaction COLUMNS tx_type;
```

---

## 8. 安全与隐私

### 8.1 本地优先
- 所有数据存储在本地文件系统
- 无需网络连接
- 数据文件可由用户完全控制

### 8.2 备份策略
- 数据库文件天然适合 Git 版本控制（SurrealDB 单文件存储）
- 定期导出为可读的 JSON/CSV 备份

---

## 9. 后续迭代方向

### 阶段二
- 银行 CSV 导入适配器（招行、支付宝、微信）
- 账户管理（命名规范化、余额追踪）
- 数据去重和冲突解决

### 阶段三
- 预算设置与提醒
- 财务健康度评分
- 趋势预测
- GUI 界面（可选）

---

## 10. 术语表

| 术语 | 定义 |
|------|------|
| Transaction | 交易记录，资金流动的一次记录 |
| TxType | 交易类型（支出/收入/转账/债务/债权） |
| Account | 账户，资金的容器（银行卡、支付宝、现金等） |
| Category | 分类，标准化的财务类别 |
| Tag | 标签，自由输入的标记，可多选 |
| Metadata | 元数据，机器生成的扩展信息 |
| Decimal | 定点数，用于精确表示金额 |
