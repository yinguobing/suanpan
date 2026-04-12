# 数据模型与存储

## 5.1 交易记录（Transaction）

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

## 5.2 字段设计说明

### 金额设计
- 使用 `rust_decimal::Decimal` 存储
- 避免浮点数精度问题
- 支持多币种（日元无小数、科威特第纳尔 3 位小数）

### 账户设计（ID 与名称解耦）
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

### 分类设计（ID 与名称解耦）
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

### 标签设计（ID 与名称解耦）
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

### 分类、标签、元数据三层设计

| 字段 | 性质 | 用途 | 示例 |
|------|------|------|------|
| **category** | 单值、必填、受控词表 | 标准财务分类，用于常规统计 | `餐饮` / `交通` / `工资` |
| **tags** | 多值、可选、自由输入 | 跨分类的临时标记 | `旅游` / `2026-Q1` / `家人` |
| **metadata** | JSON、可选、机器数据 | 原始数据保留，系统集成 | `{"confidence": 0.95, "raw_input": "..."}` |

## 5.3 交易类型详解

### Expense（支出）
资金从你的账户流向外部。
- `account_from`: 你的账户（支付宝/现金/招行卡）
- `account_to`: 商户/收款方（麦当劳/超市/房东）

### Income（收入）
外部资金流入你的账户。
- `account_from`: 付款方（公司/客户/理财）
- `account_to`: 你的账户（招行卡/支付宝）

### Transfer（转账）
资金在你的账户之间流动。
- `account_from`: 来源账户（招行卡）
- `account_to`: 目标账户（支付宝/招招理财）

### DebtChange（债务变动）
**你欠别人的钱**发生变化。
- 借入：`account_from: "朋友-小明"`, `account_to: "现金"`
- 偿还：`account_from: "现金"`, `account_to: "朋友-小明"`

### CreditChange（债权变动）
**别人欠你的钱**发生变化。
- 借出：`account_from: "现金"`, `account_to: "朋友-小明"`
- 收回：`account_from: "朋友-小明"`, `account_to: "现金"`

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
