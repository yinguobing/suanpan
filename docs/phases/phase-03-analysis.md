# 阶段三：数据分析

> **状态**: [OK] 已完成

## 目标

提供丰富的数据分析能力，帮助用户理解财务状况。

## 功能范围

### 已实现功能

#### 高级统计

| 功能 | 命令 | 说明 |
|------|------|------|
| 自定义时间范围统计 | `suanpan stats --from YYYY-MM-DD --to YYYY-MM-DD` | 任意日期范围 |
| 按账户统计 | `suanpan stats --by-account` | 各账户收支汇总 |
| 指定账户统计 | `suanpan stats --account <ID>` | 单个账户详情 |
| 层级分类统计 | `suanpan stats --by-category` | 多级分类树形展示 |

#### 趋势分析

| 功能 | 命令 | 说明 |
|------|------|------|
| 多周期趋势 | `suanpan trend --period <type>` | day/week/month/quarter/year |
| 日期范围 | `--from/--to` | 指定分析时间段 |
| 分类趋势 | `--by-category` | 按分类展示趋势 |

#### 对比分析

| 功能 | 命令 | 说明 |
|------|------|------|
| 月度对比 | `suanpan compare --month YYYY-MM` | 环比+同比 |
| 环比分析 | `--compare-type mom` | 与上月对比 |
| 同比分析 | `--compare-type yoy` | 与去年同月对比 |

#### 查询增强

| 功能 | 命令 | 说明 |
|------|------|------|
| 模糊搜索 | `suanpan list --search <关键词>` | 匹配描述和备注 |
| 组合筛选 | - | 时间+分类+账户+金额范围 |
| 金额范围 | `--min-amount/--max-amount` | 金额筛选 |
| 导出结果 | `--output <路径>` | CSV/文本格式 |

#### 可视化报表

| 功能 | 命令 | 说明 |
|------|------|------|
| HTML 报表 | `suanpan report --month YYYY-MM` | 生成交互式报表 |
| 支出饼图 | - | Top 8 + 其他聚合 |
| 趋势折线图 | - | 最近12个月收支趋势 |
| 每日柱状图 | - | 每日收支柱状图 |

## 核心命令示例

### 统计命令

```bash
# 自定义日期范围统计
suanpan stats --from 2025-01-01 --to 2025-03-31

# 层级分类统计
suanpan stats --by-category

# 按账户统计
suanpan stats --by-account --month 2025-04
```

### 趋势分析

```bash
# 按月趋势（默认最近6个月）
suanpan trend --period month

# 季度趋势（指定日期范围）
suanpan trend --period quarter --from 2025-01-01 --to 2025-12-31

# 周趋势并按分类展示
suanpan trend --period week --from 2026-01-01 --to 2026-01-31 --by-category
```

### 对比分析

```bash
# 月度对比（环比+同比）
suanpan compare --month 2026-01

# 仅环比对比
suanpan compare --month 2026-01 --compare-type mom

# 仅同比对比
suanpan compare --month 2026-01 --compare-type yoy
```

### 查询增强

```bash
# 模糊搜索
suanpan list --search "午餐"

# 组合筛选：时间范围 + 账户 + 金额范围
suanpan list --from 2026-01-01 --to 2026-01-31 --account "招行卡" --min-amount 100

# 导出查询结果
suanpan list --from 2026-01-01 --to 2026-01-31 --output january.csv

# 组合筛选 + 导出
suanpan list --category "餐饮" --tx-type expense --min-amount 50 --output food_expenses.csv
```

### 可视化报表

```bash
# 生成月度 HTML 报表
suanpan report --month 2026-01

# 指定输出目录
suanpan report --month 2026-01 --output ./reports

# 只生成图表
suanpan report --month 2026-01 --charts-only
```

## 技术要点

- **层级分类统计**: 使用 `tree-ds` 库展示分类树，但绕过其单根限制实现多根分类聚合
- **趋势分析**: 支持多种周期类型，通过日期格式化生成周期键
- **对比分析**: 使用 `tokio::try_join!` 并发查询当前、上月、去年同月数据
- **模糊搜索**: 使用 `string::contains` 进行不区分大小写的匹配
- **报表生成**: 使用 `plotters` 库生成图表，嵌入 HTML 模板
