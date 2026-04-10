# 算盘 PRD

## 1. 产品概述

### 1.1 产品定位
智能体时代的个人财务管理CLI工具，以CLI为核心交互入口，通过结构化参数让你的智能体实现高效记账、多源数据整合和财务分析。

### 1.2 核心理念
- **CLI 优先**：通过结构化命令行参数进行交互，简洁高效
- **资金流动为核心**：所有财务活动抽象为资金的流动，简化数据模型
- **渐进式演进**：分阶段迭代，MVP 聚焦核心记账和查询功能

---

## 2. 目标用户

**主要用户**：尹国冰
- 需要长期、持续的财务记录
- 偏好简洁高效的命令行操作
- 重视数据隐私和本地控制
- 有技术背景，可进行二次开发

---

## 3. 阶段规划

### 阶段一：MVP（已完成）
目标：建立基础记账能力，验证核心交互流程

**功能范围：**
- ✅ 结构化参数录入（CLI 参数）
- ✅ 流水记录存储、查询、移除、更新
- ✅ 基础统计（月度汇总、分类占比）
- ✅ CLI 界面

### 阶段二：数据整合（已完成）
- ✅ 第三方数据导入。已实现随手记 XLS/XLSX/CSV 导入，支持多 sheet、自动创建账户和分类、重复检测。
- ✅ 数据清洗和去重。基于时间+金额+账户+描述的重复检测，支持 `--dry-run` 预览和 `--skip-dedup` 跳过。

### 阶段三：数据分析（进行中）
- ✅ 高级统计（部分完成）
  - ✅ 自定义时间范围统计: `suanpan stats --from YYYY-MM-DD --to YYYY-MM-DD`
  - ✅ 按账户统计: `suanpan stats --by-account` / `--account <ID>`
  - ⬜ 按分类层级汇总（支持多级分类）
  - ⬜ 趋势分析（日/周/月/季度/年）
  - ⬜ 按时间段对比（环比、同比）
- ⬜ 查询增强
  - 模糊搜索描述和备注
  - 组合筛选（时间+分类+账户+金额范围）
  - 导出查询结果到 CSV/Excel
- ✅ 数据可视化报表
  - 生成 HTML 报表（交互式图表）
  - 支出分类饼图（Top 8 + 其他聚合）
  - 收支趋势折线图（最近12个月）
  - 每日收支柱状图
  - 命令: `suanpan report --month YYYY-MM`

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
suanpan/
├── Cargo.toml
├── src/
│   ├── main.rs              # CLI 入口
│   ├── lib.rs
│   ├── models/              # 数据模型
│   │   ├── mod.rs
│   │   ├── transaction.rs   # 交易记录
│   │   ├── account.rs       # 账户
│   │   ├── category.rs      # 分类
│   │   ├── tag.rs           # 标签
│   │   └── types.rs         # 枚举类型
│   ├── db/                  # 数据库层
│   │   ├── mod.rs
│   │   └── surreal.rs       # SurrealDB 封装
│   ├── commands/            # CLI 子命令
│   │   ├── mod.rs
│   │   ├── add.rs
│   │   ├── remove.rs        # 移除交易
│   │   ├── update.rs
│   │   ├── list.rs
│   │   ├── stats.rs
│   │   ├── migrate.rs       # 数据迁移
│   │   ├── account.rs       # 账户管理
│   │   ├── category.rs      # 分类管理
│   │   └── tag.rs           # 标签管理
│   └── error.rs             # 错误处理
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
    CsvImport,      // 银行账单导入
    Manual,         // 手动录入（CLI 参数）
}

/// 交易记录
struct Transaction {
    id: Option<RecordId>,          // 全局唯一标识（数据库自动生成）
    timestamp: Datetime,           // 交易发生时间
    amount: Decimal,               // 金额（正数，Decimal 精度）
    currency: String,              // 货币代码：CNY, USD...
    tx_type: TxType,               // 交易类型
    
    // 复式记账核心（存账户ID而非名称）
    account_from_id: String,       // 来源账户ID（外键，关联 account 表）
    account_to_id: Option<String>, // 去向账户ID（外键，关联 account 表）
    
    category_id: String,           // 分类ID（外键，关联 category 表）
    description: Option<String>,   // 备注描述
    
    // 扩展字段（存标签ID而非名称）
    tag_ids: Vec<String>,          // 标签ID列表（外键，关联 tag 表）
    metadata: Option<Value>,       // 任意扩展数据（JSON）
    
    // 系统字段
    created_at: Datetime,          // 记录创建时间
    updated_at: Option<Datetime>,  // 修改时间
    source: TxSource,              // 数据来源
}

/// 账户
struct Account {
    id: String,                    // 永久唯一ID（如 "acc_xxx"）
    name: String,                  // 显示名称（可修改）
    account_type: AccountType,     // 账户类型
    parent_id: Option<String>,     // 父账户ID（用于子账户，如信用卡下挂副卡）
    created_at: Datetime,          // 创建时间
}

/// 账户类型
enum AccountType {
    BankCard,      // 银行卡
    EWallet,       // 电子钱包（支付宝、微信）
    Cash,          // 现金
    Investment,    // 投资理财
    Credit,        // 信用卡
    Other,         // 其他
}

/// 分类（支持层级）
struct Category {
    id: String,                    // 永久唯一ID（如 "cat_xxx"）
    name: String,                  // 显示名称（可修改）
    parent_id: Option<String>,     // 父分类ID（None 表示根分类）
    full_path: String,             // 预计算完整路径（如 "餐饮/午餐"）
    level: u32,                    // 层级深度
    created_at: Datetime,          // 创建时间
}

/// 标签
struct Tag {
    id: String,                    // 永久唯一ID（如 "tag_xxx"）
    name: String,                  // 显示名称（可修改）
    color: Option<String>,         // 显示颜色（可选）
    created_at: Datetime,          // 创建时间
}
```

### 5.2 字段设计说明

#### 金额设计
- 使用 `rust_decimal::Decimal` 存储
- 避免浮点数精度问题
- 支持多币种（日元无小数、科威特第纳尔 3 位小数）

#### 账户设计（ID 与名称解耦）
- **独立账户表**：`account` 表存储 ID、名称、账户类型
- **Transaction 存 ID**：`account_from_id`/`account_to_id` 字段存储永久 ID
- **名称可改**：修改账户名称只需更新 `account` 表，不影响历史流水
- **支持子账户**：通过 `parent_id` 实现（如"招行卡"下挂"招招理财"）

**账户设计示例：**
```
account 表:
┌─────────────┬──────────┬─────────────┬────────────┐
│ id          │ name     │ account_type│ parent_id  │
├─────────────┼──────────┼─────────────┼────────────┤
│ acc_alipay  │ 支付宝   │ EWallet     │ null       │
│ acc_cmb     │ 招行卡   │ BankCard    │ null       │
│ acc_cmb_li  │ 招招理财 │ Investment  │ acc_cmb    │ ← 子账户
└─────────────┴──────────┴─────────────┴────────────┘

修改名称后（把"支付宝"改为"Alipay"）:
├─────────────┼──────────┼─────────────┼────────────┤
│ acc_alipay  │ Alipay   │ EWallet     │ null       │
└─────────────┴──────────┴─────────────┴────────────┘

所有 Transaction.account_from_id 仍为 "acc_alipay"，无需修改
```

#### 分类设计（ID 与名称解耦）
- **独立分类表**：`category` 表存储 ID、名称、父子关系
- **Transaction 存 ID**：`category_id` 字段存储永久 ID，不是名称
- **层级支持**：通过 `parent_id` 指针实现树形结构
- **名称可改**：修改分类名称只需更新 `category` 表，不影响历史流水
- **预计算路径**：`full_path` 字段缓存完整路径（如 "餐饮/午餐"），加速查询

**分类设计示例：**
```
category 表:
┌─────────────┬──────────┬────────────┬─────────────────┐
│ id          │ name     │ parent_id  │ full_path       │
├─────────────┼──────────┼────────────┼─────────────────┤
│ cat_root    │ 根       │ null       │                 │
│ cat_food    │ 餐饮     │ cat_root   │ 餐饮            │
│ cat_lunch   │ 午餐     │ cat_food   │ 餐饮/午餐       │
│ cat_traffic │ 交通     │ cat_root   │ 交通            │
└─────────────┴──────────┴────────────┴─────────────────┘

修改名称后（把"餐饮"改为"吃饭"）:
├─────────────┼──────────┼────────────┼─────────────────┤
│ cat_food    │ 吃饭     │ cat_root   │ 吃饭            │
│ cat_lunch   │ 午餐     │ cat_food   │ 吃饭/午餐       │ ← 级联更新
└─────────────┴──────────┴────────────┴─────────────────┘

所有 Transaction.category_id 仍为 "cat_food"，无需修改
```

#### 标签设计（ID 与名称解耦）
- **独立标签表**：`tag` 表存储 ID、名称、颜色
- **Transaction 存 ID**：`tag_ids` 字段存储标签 ID 列表
- **名称可改**：修改标签名称只需更新 `tag` 表，不影响历史流水
- **颜色支持**：可为标签指定显示颜色，便于可视化

**标签设计示例：**
```
tag 表:
┌──────────────┬──────────┬────────┐
│ id           │ name     │ color  │
├──────────────┼──────────┼────────┤
│ tag_q1       │ 2026-Q1  │ #FF0000│
│ tag_trip     │ 旅游     │ #00FF00│
│ tag_family   │ 家人     │ #0000FF│
└──────────────┴──────────┴────────┘

修改名称后（把"2026-Q1"改为"第一季度"）:
├──────────────┼──────────┼────────┤
│ tag_q1       │ 第一季度 │ #FF0000│
└──────────────┴──────────┴────────┘

所有 Transaction.tag_ids 仍包含 "tag_q1"，无需修改
```

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

### 6.1 命令行参数输入示例

CLI 通过结构化参数接收交易信息，直接录入无需确认：

```bash
# 添加一笔支出（分类使用路径格式）
suanpan add -a 35 -f 支付宝 -t expense -c "餐饮/午餐" -d "午餐"

# 添加一笔收入（分类使用路径格式）
suanpan add -a 8500 -t income -f 公司 -c "收入/工资" -d "三月工资"

# 添加转账记录
suanpan add -a 1000 -t transfer -f 招行卡 -o 支付宝 -c "转账"
```

参数说明：
- `-a, --amount`: 金额（必填）
- `-t, --tx-type`: 交易类型，默认为 expense（可选值: expense, income, transfer, debtchange, creditchange）
- `-f, --from`: 来源账户ID（必填）
- `-o, --to`: 去向账户ID（可选，transfer/debtchange/creditchange 类型时建议填写）
- `-c, --category`: 分类路径（如 "餐饮/午餐"），默认为"其他"
- `-d, --description`: 描述/备注（可选）
- `-y, --currency`: 货币，默认为 CNY
- `-g, --tag`: 标签，可多次使用

### 6.2 CLI 命令参考

```bash
# 添加交易记录（结构化参数，分类使用路径）
suanpan add -a 35 -f 支付宝 -t expense -c "餐饮/午餐" -d "午餐"

# 移除交易记录（通过短 ID，支持批量）
suanpan remove f4sp877fxbwc
suanpan remove f4sp877fxbwc abc123dexy78 xyz789gh1234

# 更新交易记录（通过短 ID，只更新指定字段）
suanpan update f4sp877fxbwc -a 40 -d "午餐+饮料"

# 列出最近流水（显示完整时间，ID 在末尾）
suanpan list --limit 20

# 按月统计
suanpan stats --month 2025-04

# 按分类统计（支持层级汇总）
suanpan stats --by-category

# 自定义日期范围统计
suanpan stats --from 2025-01-01 --to 2025-03-31
suanpan stats --from 2025-01-01 --to 2025-03-31 --by-category

# 按账户统计
suanpan stats --by-account
suanpan stats --by-account --month 2025-04

# 指定账户统计
suanpan stats --account 支付宝
suanpan stats --account 支付宝 --from 2025-01-01 --to 2025-01-31

# ========== 账户管理命令 ==========

# 列出所有账户
suanpan account list

# 添加账户（ID 自动生成）
suanpan account add <名称> -a <类型>
suanpan account add "支付宝" -a e-wallet

# 添加子账户（如信用卡副卡、理财子账户）
suanpan account add "招招理财" -a investment --parent acc_cmb

# 重命名账户
suanpan account rename <ID> <新名称>
suanpan account rename acc_alipay "Alipay"

# 移除账户（需确保无流水关联、无子账户）
suanpan account remove <ID>

# ========== 标签管理命令 ==========

# 列出所有标签
suanpan tag list

# 添加标签（ID 自动生成）
suanpan tag add <名称> [--color <颜色>]
suanpan tag add "2026-Q1" --color "#FF0000"

# 重命名标签
suanpan tag rename <ID或名称> <新名称>
suanpan tag rename tag_q1 "第一季度"

# 移除标签（会自动从所有流水移除关联）
suanpan tag remove <ID或名称>

# ========== 分类管理命令 ==========

# 查看分类树
suanpan category tree

# 添加分类（指定完整路径，各级分类自动创建）
suanpan category add <路径>
suanpan category add "餐饮/午餐/食堂"

# 重命名分类（自动级联更新子分类路径）
suanpan category rename <路径或ID> <新名称>
suanpan category rename "餐饮" "吃饭"

# 移除分类（需确保无流水关联）
suanpan category remove <路径或ID>
suanpan category remove "餐饮/午餐"
```

**List 输出格式：**

```
+---------------------+------+------+------+--------+------+------+----------+--------------+
| 时间                | 类型 | 金额 | 货币 | 账户   | 去向 | 分类 | 备注     | ID           |
+===========================================================================================+
| 2026-04-07 22:13:54 | 支出 | 120  | CNY  | 工资卡 | -    | 晚餐 | 朋友聚餐 | 1mfor6omfh30 |
|---------------------+------+------+------+--------+------+------+----------+--------------|
| 2026-04-07 22:12:51 | 支出 | 30   | CNY  | 现金   | -    | 地铁 | 地铁     | xf2mmnslromy |
+---------------------+------+------+------+--------+------+------+----------+--------------+
```

**说明：**
- **时间格式**：完整日期时间 `YYYY-MM-DD HH:MM:SS`
- **ID 位置**：表格最后一列，便于复制使用
- **短 ID**：显示 Record ID 的前 12 位（如 `f4sp877fxbwc`），平衡可读性与唯一性
- **移除/更新**：使用短 ID 即可，命令内部使用完整 ID 匹配

---

## 7. 数据存储

### 7.1 存储位置
- **默认路径**: `~/.local/share/suanpan/`
- **数据库文件**: `data.db`
- **配置文件**: `config.toml`

### 7.2 SurrealDB 表结构

```surql
-- 账户表
DEFINE TABLE account SCHEMAFULL;

DEFINE FIELD id ON account TYPE string;                     -- 永久ID（如 "acc_xxx"）
DEFINE FIELD name ON account TYPE string;                   -- 显示名称
DEFINE FIELD account_type ON account TYPE string;           -- 账户类型
DEFINE FIELD parent_id ON account TYPE option<string>;      -- 父账户ID
DEFINE FIELD created_at ON account TYPE datetime;           -- 创建时间

-- 账户索引
DEFINE INDEX idx_account_id ON account COLUMNS id UNIQUE;
DEFINE INDEX idx_account_parent ON account COLUMNS parent_id;

-- 分类表（支持层级）
DEFINE TABLE category SCHEMAFULL;

DEFINE FIELD id ON category TYPE string;                    -- 永久ID（如 "cat_xxx"）
DEFINE FIELD name ON category TYPE string;                  -- 显示名称
DEFINE FIELD parent_id ON category TYPE option<string>;     -- 父分类ID
DEFINE FIELD full_path ON category TYPE string;             -- 预计算完整路径
DEFINE FIELD level ON category TYPE int;                    -- 层级深度
DEFINE FIELD created_at ON category TYPE datetime;          -- 创建时间

-- 分类索引
DEFINE INDEX idx_category_id ON category COLUMNS id UNIQUE;
DEFINE INDEX idx_category_parent ON category COLUMNS parent_id;
DEFINE INDEX idx_category_path ON category COLUMNS full_path;

-- 标签表
DEFINE TABLE tag SCHEMAFULL;

DEFINE FIELD id ON tag TYPE string;                         -- 永久ID（如 "tag_xxx"）
DEFINE FIELD name ON tag TYPE string;                       -- 显示名称
DEFINE FIELD color ON tag TYPE option<string>;              -- 显示颜色
DEFINE FIELD created_at ON tag TYPE datetime;               -- 创建时间

-- 标签索引
DEFINE INDEX idx_tag_id ON tag COLUMNS id UNIQUE;

-- 交易记录表
DEFINE TABLE transaction SCHEMAFULL;

DEFINE FIELD id ON transaction TYPE record;
DEFINE FIELD timestamp ON transaction TYPE datetime;
DEFINE FIELD amount ON transaction TYPE decimal;
DEFINE FIELD currency ON transaction TYPE string;
DEFINE FIELD tx_type ON transaction TYPE string;            -- expense/income/transfer/debtchange/creditchange
DEFINE FIELD account_from_id ON transaction TYPE string;    -- 来源账户ID
DEFINE FIELD account_to_id ON transaction TYPE option<string>; -- 去向账户ID
DEFINE FIELD category_id ON transaction TYPE string;        -- 分类ID
DEFINE FIELD description ON transaction TYPE option<string>;
DEFINE FIELD tag_ids ON transaction TYPE array<string>;     -- 标签ID列表
DEFINE FIELD metadata ON transaction TYPE option<object>;
DEFINE FIELD created_at ON transaction TYPE datetime;
DEFINE FIELD updated_at ON transaction TYPE option<datetime>;
DEFINE FIELD source ON transaction TYPE string;             -- manual/csv_import

-- 索引
DEFINE INDEX idx_timestamp ON transaction COLUMNS timestamp;
DEFINE INDEX idx_tx_category ON transaction COLUMNS category_id;
DEFINE INDEX idx_account_from ON transaction COLUMNS account_from_id;
DEFINE INDEX idx_account_to ON transaction COLUMNS account_to_id;
DEFINE INDEX idx_tx_type ON transaction COLUMNS tx_type;
DEFINE INDEX idx_tag_ids ON transaction COLUMNS tag_ids;
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

## 9. 术语表

| 术语 | 定义 |
|------|------|
| Transaction | 交易记录，资金流动的一次记录 |
| TxType | 交易类型（支出/收入/转账/债务/债权） |
| Account | 账户，资金的容器（银行卡、支付宝、现金等） |
| Category | 分类，标准化的财务类别 |
| Tag | 标签，自由输入的标记，可多选 |
| Metadata | 元数据，机器生成的扩展信息 |
| Decimal | 定点数，用于精确表示金额 |
