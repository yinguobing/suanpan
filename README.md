# 国冰财务管理系统 (finance-cli)

智能体时代的个人财务管理CLI工具，基于Rust + SurrealDB构建，支持本地数据存储和多种交易类型管理。

将此页面发给你的智能体，让它协助你管理自己的财务。

## 特性

- 📊 **本地优先**：所有数据存储在本地，无需网络连接
- 💾 **嵌入式数据库**：使用 SurrealKV 存储，单文件便于备份
- 📝 **多种交易类型**：支持支出、收入、转账、债务、债权
- 📈 **统计报表**：月度收支统计、分类占比分析
- 🔍 **灵活查询**：按分类、类型、日期范围筛选
- 💰 **精确计算**：使用 Decimal 类型避免浮点精度问题

## 安装

### 从源码编译

```bash
# 克隆仓库
git clone https://github.com/yinguobing/finance-cli.git
cd finance-cli

# 编译发布版本
cargo build --release

# 将二进制文件添加到 PATH
sudo cp target/release/finance /usr/local/bin/finance
```

### 依赖

- Rust 1.70+
- Cargo

## 快速开始

```bash
# 添加一笔支出
finance add -a 35 -f 支付宝 -o 食堂 -c 餐饮 -d "午餐"

# 添加一笔收入
finance add -a 8500 -t income -f 公司 -o 招行卡 -c 工资 -d "三月工资"

# 查看最近记录
finance list

# 查看月度统计
finance stats --by-category
```

## 命令参考

### `add` - 添加交易记录

```bash
finance add [OPTIONS] --amount <AMOUNT> --from <FROM>

选项:
  -a, --amount <AMOUNT>            金额（必填）
  -t, --tx-type <TX_TYPE>          交易类型 [默认: expense]
  -f, --from <FROM>                来源账户（必填）
  -o, --to <TO>                    去向账户/商户（可选）
  -c, --category <CATEGORY>        分类 [默认: 其他]
  -d, --description <DESCRIPTION>  描述/备注（可选）
  -y, --currency <CURRENCY>        货币 [默认: CNY]
  -g, --tag <TAG>                  标签，可多次使用
  -h, --help                       帮助信息
```

**交易类型**:
- `expense` / `支出` / `e` - 支出（默认）
- `income` / `收入` / `i` - 收入
- `transfer` / `转账` / `t` - 转账
- `debt` / `债务` / `d` - 债务变动（借入/偿还）
- `credit` / `债权` / `c` - 债权变动（借出/收回）

**示例**:

```bash
# 支出
finance add -a 35 -f 支付宝 -o 食堂 -c 餐饮

# 收入
finance add -a 8500 -t income -f 公司 -o 招行卡 -c 工资

# 转账
finance add -a 1000 -t transfer -f 招行卡 -o 支付宝

# 添加标签
finance add -a 200 -f 现金 -o 朋友 -c 人情 -g 借款 -g 2026-Q1
```

### `list` - 列出交易记录

```bash
finance list [OPTIONS]

选项:
  -n, --limit <LIMIT>      显示条数 [默认: 20]
      --from <FROM>        起始日期 (YYYY-MM-DD)
      --to <TO>            结束日期 (YYYY-MM-DD)
  -c, --category <CATEGORY>  按分类筛选
  -t, --tx-type <TX_TYPE>   按类型筛选
  -h, --help               帮助信息
```

**示例**:

```bash
# 列出最近20条
finance list

# 列出最近10条
finance list -n 10

# 按分类筛选
finance list -c 餐饮

# 按日期范围筛选
finance list --from 2026-04-01 --to 2026-04-30
```

### `stats` - 统计报表

```bash
finance stats [OPTIONS]

选项:
  -m, --month <MONTH>      月份 (YYYY-MM) [默认: 当前月]
      --by-category        显示分类占比
  -h, --help               帮助信息
```

**示例**:

```bash
# 本月统计
finance stats

# 本月统计（含分类占比）
finance stats --by-category

# 指定月份
finance stats -m 2026-03
```

## 数据存储

数据默认存储在：

- **Linux**: `~/.local/share/finance-cli/data.db`
- **macOS**: `~/Library/Application Support/finance-cli/data.db`
- **Windows**: `%APPDATA%\finance-cli\data.db`

### 备份

数据文件为单一 SurrealKV 数据库文件，可直接复制备份：

```bash
# 备份
cp ~/.local/share/finance-cli/data.db ~/finance-backup-$(date +%Y%m%d).db

# 恢复
cp ~/finance-backup-20260406.db ~/.local/share/finance-cli/data.db
```

## 交易类型说明

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

## 开发计划

### 阶段一：MVP
- ✅ 基础记账
- ✅ 交易账户
- ✅ 交易分类
- ✅ 交易标签
- ✅ 本地数据持久化
- ✅ 基本统计：月度统计报表

### 阶段二：数据整合
- ⬜ 第三方数据CSV导入
- ⬜ 数据清洗和去重

### 阶段三：数据分析
- ⬜ 高级数据统计
- ⬜ 数据可视化

## 技术栈

| 组件 | 技术 |
|------|------|
| 编程语言 | Rust |
| 数据库 | SurrealDB (SurrealKV) |
| CLI 框架 | clap |
| 时间处理 | chrono |
| 金额计算 | rust_decimal |
| 表格输出 | comfy-table |

## 许可证

本项目采用 GPL-3.0 许可证。

## 作者

尹国冰
