# 算盘 (suanpan)

智能体时代的个人财务管理 CLI 工具。算盘是中国传统计算工具，象征精确与效率。

基于 Rust + SurrealDB 构建，支持本地数据存储和多种交易类型管理。

将此页面发给你的智能体，让它协助你管理自己的财务。

## 特性

- 📊 **本地优先**：所有数据存储在本地，无需网络连接
- 💾 **嵌入式数据库**：使用 SurrealKV 存储，单文件便于备份
- 📝 **多种交易类型**：支持支出、收入、转账、债务、债权
- 📈 **统计报表**：月度收支统计、分类占比分析
- 📊 **趋势分析**：支持日/周/月/季度/年多周期趋势分析
- 🔍 **智能查询**：模糊搜索、组合筛选、金额范围筛选
- 📉 **对比分析**：环比/同比分析，洞察财务变化
- 📄 **数据导出**：支持 CSV 和文本格式导出
- 📊 **可视化报表**：生成 HTML 报表（饼图、折线图、柱状图）
- 📁 **数据导入**：支持随手记 XLS/XLSX/CSV 导入，自动去重
- 🏷️ **灵活分类**：多级分类体系，支持层级汇总
- 💰 **精确计算**：使用 Decimal 类型避免浮点精度问题

## 安装

### 从源码编译

```bash
# 克隆仓库
git clone https://github.com/yinguobing/suanpan.git
cd suanpan

# 编译发布版本
cargo build --release

# 将二进制文件添加到 PATH
sudo cp target/release/suanpan /usr/local/bin/suanpan
```

### 依赖

- Rust 1.70+
- Cargo

## 快速开始

```bash
# 添加一笔支出
suanpan add -a 35 -f 支付宝 -c "餐饮/午餐" -d "午餐"

# 添加一笔收入
suanpan add -a 8500 -t income -f 公司 -c "收入/工资" -d "三月工资"

# 查看最近记录
suanpan list

# 查看月度统计
suanpan stats --month 2026-04

# 查看趋势分析
suanpan trend --period month

# 生成可视化报表
suanpan report --month 2026-04
```

## 命令参考

### `add` - 添加交易记录

```bash
suanpan add [OPTIONS] --amount <AMOUNT> --from <FROM>

选项:
  -a, --amount <AMOUNT>            金额（必填）
  -t, --tx-type <TX_TYPE>          交易类型 [默认: expense]
  -f, --from <FROM>                来源账户（必填）
  -o, --to <TO>                    去向账户/商户（可选）
  -c, --category <CATEGORY>        分类路径 [默认: 其他]
  -d, --description <DESCRIPTION>  描述/备注（可选）
  -y, --currency <CURRENCY>        货币 [默认: CNY]
  -g, --tag <TAG>                  标签，可多次使用
  -h, --help                       帮助信息
```

**交易类型**:
- `expense` / `支出` / `e` - 支出（默认）
- `income` / `收入` / `i` - 收入
- `transfer` / `转账` / `t` - 转账
- `debtchange` / `债务变动` - 债务变动（借入/偿还）
- `creditchange` / `债权变动` - 债权变动（借出/收回）

**示例**:

```bash
# 支出（分类使用路径格式）
suanpan add -a 35 -f 支付宝 -c "餐饮/午餐" -d "工作午餐"

# 收入
suanpan add -a 8500 -t income -f 公司 -c "收入/工资" -d "三月工资"

# 转账
suanpan add -a 1000 -t transfer -f 招行卡 -o 支付宝 -c "转账"

# 添加标签
suanpan add -a 200 -f 现金 -t debtchange -c "借贷/借入" -d "临时周转" -g 借款 -g 2026-Q1
```

### `list` - 列出交易记录

```bash
suanpan list [OPTIONS]

选项:
  -n, --limit <LIMIT>              显示条数 [默认: 20]
      --from <FROM>                起始日期 (YYYY-MM-DD)
      --to <TO>                    结束日期 (YYYY-MM-DD)
  -c, --category <CATEGORY>        按分类筛选
  -t, --tx-type <TX_TYPE>          按类型筛选
      --account <ACCOUNT>          按账户筛选
      --search <SEARCH>            模糊搜索描述
      --min-amount <MIN_AMOUNT>    最小金额
      --max-amount <MAX_AMOUNT>    最大金额
      --output <OUTPUT>            导出到文件 (CSV/文本)
  -h, --help                       帮助信息
```

**示例**:

```bash
# 列出最近20条
suanpan list

# 列出最近10条
suanpan list -n 10

# 按分类筛选
suanpan list -c "餐饮/午餐"

# 按日期范围筛选
suanpan list --from 2026-04-01 --to 2026-04-30

# 模糊搜索（搜索描述）
suanpan list --search "鸡蛋"

# 组合筛选 + 导出 CSV
suanpan list --from 2026-01-01 --to 2026-01-31 --category "餐饮" --output food.csv
```

### `remove` - 移除交易记录

```bash
suanpan remove <ID>...

示例:
  suanpan remove f4sp877fxbwc
  suanpan remove f4sp877fxbwc abc123dexy78
```

### `update` - 更新交易记录

```bash
suanpan update <ID> [OPTIONS]

选项:
  -a, --amount <AMOUNT>            金额
  -t, --tx-type <TX_TYPE>          交易类型
  -f, --from <FROM>                来源账户
  -o, --to <TO>                    去向账户
  -c, --category <CATEGORY>        分类
  -d, --description <DESCRIPTION>  描述
  -y, --currency <CURRENCY>        货币

示例:
  suanpan update f4sp877fxbwc -a 40 -d "午餐+饮料"
```

### `stats` - 统计报表

```bash
suanpan stats [OPTIONS]

选项:
  -m, --month <MONTH>              月份 (YYYY-MM) [默认: 当前月]
      --from <FROM>                起始日期 (YYYY-MM-DD)
      --to <TO>                    结束日期 (YYYY-MM-DD)
      --by-category                按分类统计（层级汇总）
      --by-account                 按账户统计
      --account <ACCOUNT>          指定账户统计
  -h, --help                       帮助信息
```

**示例**:

```bash
# 本月统计
suanpan stats

# 按分类统计（层级汇总）
suanpan stats --by-category

# 指定月份
suanpan stats -m 2026-03

# 自定义日期范围
suanpan stats --from 2026-01-01 --to 2026-03-31 --by-category
```

### `trend` - 趋势分析

```bash
suanpan trend [OPTIONS]

选项:
  -p, --period <PERIOD>            周期类型 [默认: month]
                                   可选: day, week, month, quarter, year
      --from <FROM>                起始日期 (YYYY-MM-DD)
      --to <TO>                    结束日期 (YYYY-MM-DD)
      --by-category                按分类展示趋势
  -h, --help                       帮助信息
```

**示例**:

```bash
# 按月趋势（默认最近6个月）
suanpan trend --period month

# 季度趋势
suanpan trend --period quarter --from 2025-01-01 --to 2025-12-31

# 周趋势并按分类展示
suanpan trend --period week --from 2026-01-01 --to 2026-01-31 --by-category
```

### `compare` - 对比分析（环比/同比）

```bash
suanpan compare [OPTIONS]

选项:
  -m, --month <MONTH>              月份 (YYYY-MM) [默认: 当前月]
      --compare-type <TYPE>        对比类型 [默认: both]
                                   可选: mom (环比), yoy (同比), both
  -h, --help                       帮助信息
```

**示例**:

```bash
# 月度对比（环比+同比）
suanpan compare --month 2026-01

# 仅环比对比（与上月对比）
suanpan compare --month 2026-01 --compare-type mom

# 仅同比对比（与去年同月对比）
suanpan compare --month 2026-01 --compare-type yoy
```

### `report` - 生成可视化报表

```bash
suanpan report [OPTIONS]

选项:
  -m, --month <MONTH>              月份 (YYYY-MM) [默认: 当前月]
  -o, --output <OUTPUT>            输出目录 [默认: ./report]
      --charts-only                只生成图表（不生成 HTML）
  -h, --help                       帮助信息
```

**示例**:

```bash
# 生成月度 HTML 报表
suanpan report --month 2026-01

# 指定输出目录
suanpan report --month 2026-01 --output ./reports

# 只生成图表
suanpan report --month 2026-01 --charts-only
```

### `import` - 数据导入

```bash
suanpan import <SUBCOMMAND>

子命令:
  suishouji    导入随手记 XLS/XLSX/CSV 文件

选项:
      --dry-run              预览导入结果，不实际写入
      --skip-dedup           跳过重复检测
```

**示例**:

```bash
# 导入随手记文件
suanpan import suishouji ./path/to/suishouji.xlsx

# 预览导入（不实际写入）
suanpan import suishouji ./path/to/suishouji.xlsx --dry-run

# 跳过重复检测
suanpan import suishouji ./path/to/suishouji.xlsx --skip-dedup
```

### `account` - 账户管理

```bash
suanpan account <SUBCOMMAND>

子命令:
  list                         列出所有账户
  add <NAME>                   添加账户
  rename <ID> <NEW_NAME>       重命名账户
  remove <ID>                  移除账户
```

**示例**:

```bash
# 列出所有账户
suanpan account list

# 添加账户
suanpan account add "支付宝" -a e-wallet

# 添加子账户
suanpan account add "招招理财" -a investment --parent acc_cmb

# 重命名账户
suanpan account rename acc_alipay "Alipay"
```

### `category` - 分类管理

```bash
suanpan category <SUBCOMMAND>

子命令:
  list                         列出所有分类（树形）
  add <PATH>                   添加分类
  rename <PATH_OR_ID> <NEW_NAME>  重命名分类
  remove <PATH_OR_ID>          移除分类
```

**示例**:

```bash
# 查看分类树
suanpan category tree

# 添加分类（指定完整路径，各级分类自动创建）
suanpan category add "餐饮/午餐/食堂"

# 重命名分类（自动级联更新子分类路径）
suanpan category rename "餐饮" "吃饭"
```

### `tag` - 标签管理

```bash
suanpan tag <SUBCOMMAND>

子命令:
  list                         列出所有标签
  add <NAME>                   添加标签
  rename <ID_OR_NAME> <NEW_NAME>  重命名标签
  remove <ID_OR_NAME>          移除标签
```

**示例**:

```bash
# 列出所有标签
suanpan tag list

# 添加标签
suanpan tag add "2026-Q1" --color "#FF0000"

# 重命名标签
suanpan tag rename tag_q1 "第一季度"
```

## 数据存储

数据默认存储在：

- **Linux**: `~/.local/share/suanpan/data.db`
- **macOS**: `~/Library/Application Support/suanpan/data.db`
- **Windows**: `%APPDATA%\suanpan\data.db`

### 备份

数据文件为单一 SurrealKV 数据库文件，可直接复制备份：

```bash
# 备份
cp ~/.local/share/suanpan/data.db ~/suanpan-backup-$(date +%Y%m%d).db

# 恢复
cp ~/suanpan-backup-20260406.db ~/.local/share/suanpan/data.db
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

### 阶段一：MVP（已完成）
- ✅ 基础记账
- ✅ 交易账户
- ✅ 交易分类（支持多级）
- ✅ 交易标签
- ✅ 本地数据持久化
- ✅ 基本统计：月度统计报表

### 阶段二：数据整合（已完成）
- ✅ 第三方数据导入（随手记 XLS/XLSX/CSV）
- ✅ 数据清洗和去重
- ✅ 自动创建账户和分类

### 阶段三：数据分析（已完成）
- ✅ 趋势分析（日/周/月/季度/年）
- ✅ 对比分析（环比/同比）
- ✅ 层级分类统计
- ✅ 模糊搜索和组合筛选
- ✅ CSV 导出
- ✅ 数据可视化报表（HTML + 图表）

## 技术栈

| 组件 | 技术 |
|------|------|
| 编程语言 | Rust |
| 数据库 | SurrealDB (SurrealKV) |
| CLI 框架 | clap |
| 时间处理 | chrono |
| 金额计算 | rust_decimal |
| 表格输出 | comfy-table |
| 图表生成 | plotters |
| 树形结构 | tree-ds |

## 文档

详细产品文档见 [docs/](./docs/) 目录：

- [docs/README.md](./docs/README.md) - 文档索引
- [docs/commands.md](./docs/commands.md) - 完整命令参考
- [docs/phases/](./docs/phases/) - 各阶段开发文档

## 许可证

本项目采用 GPL-3.0 许可证。

## 作者

尹国冰
