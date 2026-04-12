# CLI 交互设计与命令参考

## 6.1 命令行参数输入示例

CLI 通过结构化参数接收交易信息，直接录入无需确认：

```bash
# 添加一笔支出（分类使用路径格式）
suanpan add -a 35 -f 支付宝 -t expense -c "餐饮/午餐" -d "午餐"

# 添加一笔收入（分类使用路径格式）
suanpan add -a 8500 -t income -f 公司 -c "收入/工资" -d "三月工资"

# 添加转账记录
suanpan add -a 1000 -t transfer -f 招行卡 -o 支付宝 -c "转账"
```

### 参数说明

| 参数 | 说明 | 必填 |
|------|------|------|
| `-a, --amount` | 金额 | 是 |
| `-t, --tx-type` | 交易类型（expense/income/transfer/debtchange/creditchange），默认 expense | 否 |
| `-f, --from` | 来源账户ID | 是 |
| `-o, --to` | 去向账户ID（transfer/debtchange/creditchange 类型时建议填写） | 否 |
| `-c, --category` | 分类路径（如 "餐饮/午餐"），默认"其他" | 否 |
| `-d, --description` | 描述/备注 | 否 |
| `-y, --currency` | 货币，默认为 CNY | 否 |
| `-g, --tag` | 标签，可多次使用 | 否 |

## 6.2 完整命令参考

### 交易管理

```bash
# 添加交易记录
suanpan add -a 35 -f 支付宝 -t expense -c "餐饮/午餐" -d "午餐"

# 移除交易记录（通过短 ID，支持批量）
suanpan remove f4sp877fxbwc
suanpan remove f4sp877fxbwc abc123dexy78 xyz789gh1234

# 更新交易记录（通过短 ID，只更新指定字段）
suanpan update f4sp877fxbwc -a 40 -d "午餐+饮料"

# 列出最近流水（显示完整时间，ID 在末尾）
suanpan list --limit 20

# 按日期范围查询
suanpan list --from 2026-01-01 --to 2026-01-31

# 按分类筛选
suanpan list --category "餐饮"

# 按账户筛选
suanpan list --account "支付宝"

# 按交易类型筛选
suanpan list --tx-type expense

# 模糊搜索描述
suanpan list --search "地铁"

# 按金额范围筛选
suanpan list --min-amount 100 --max-amount 500

# 组合筛选 + 导出 CSV
suanpan list --from 2026-01-01 --to 2026-01-31 --search "午餐" --output lunch.csv
```

### 统计分析

```bash
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
```

### 趋势分析

```bash
# 按月查看收支趋势（默认最近6个月）
suanpan trend --period month

# 查看季度趋势（指定日期范围）
suanpan trend --period quarter --from 2025-01-01 --to 2025-12-31

# 查看周趋势并按分类展示
suanpan trend --period week --from 2026-01-01 --to 2026-01-31 --by-category

# 支持的周期类型: day, week, month, quarter, year
```

### 对比分析

```bash
# 月度对比分析（环比+同比）
suanpan compare --month 2026-01

# 仅环比对比（与上月对比）
suanpan compare --month 2026-01 --compare-type mom

# 仅同比对比（与去年同月对比）
suanpan compare --month 2026-01 --compare-type yoy
```

### 可视化报表

```bash
# 生成月度 HTML 报表（含图表）
suanpan report --month 2026-01

# 指定输出目录
suanpan report --month 2026-01 --output ./reports

# 只生成图表（不生成 HTML）
suanpan report --month 2026-01 --charts-only
```

### 账户管理

```bash
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
```

### 标签管理

```bash
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
```

### 分类管理

```bash
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

## 6.3 List 输出格式

```
+---------------------------+--------+--------+------+----------+----------+-----------+----------+--------------+
| 时间                      | 类型   | 金额   | 货币 | 账户     | 去向     | 分类      | 备注     | ID           |
+================================================================================================================+
| 2026-04-11T12:30:00+08:00 | 支出   | 35.00  | CNY  | 支付宝   | -        | 餐饮/午餐 | 工作午餐 | a1b2c3d4e5f6 |
|---------------------------+--------+--------+------+----------+----------+-----------+----------+--------------|
| 2026-04-10T19:15:22+08:00 | 支出   | 128.50 | CNY  | 招行卡   | -        | 餐饮/晚餐 | 朋友聚餐 | g7h8i9j0k1l2 |
|---------------------------+--------+--------+------+----------+----------+-----------+----------+--------------|
| 2026-04-10T08:45:00+08:00 | 支出   | 6.00   | CNY  | 交通卡   | -        | 交通/地铁 | 通勤     | m3n4o5p6q7r8 |
|---------------------------+--------+--------+------+----------+----------+-----------+----------+--------------|
| 2026-04-09T18:00:00+08:00 | 收入   | 8500.00| CNY  | 公司     | 工资卡   | 收入/工资 | 三月工资 | s9t0u1v2w3x4 |
|---------------------------+--------+--------+------+----------+----------+-----------+----------+--------------|
| 2026-04-09T14:20:10+08:00 | 转账   | 5000.00| CNY  | 工资卡   | 余额宝   | 转账      | 理财     | y5z6a7b8c9d0 |
|---------------------------+--------+--------+------+----------+----------+-----------+----------+--------------|
| 2026-04-08T20:00:00+08:00 | 支出   | 299.00 | CNY  | 京东白条 | -        | 购物/数码 | 耳机     | e1f2g3h4i5j6 |
|---------------------------+--------+--------+------+----------+----------+-----------+----------+--------------|
| 2026-04-08T09:30:00+08:00 | 债务变动| 2000.00| CNY  | 朋友小明 | 现金     | 借贷/借入 | 临时周转 | k7l8m9n0o1p2 |
+---------------------------+--------+--------+------+----------+----------+-----------+----------+--------------+
共 7 条记录
```

**说明：**
- **时间格式**：ISO 8601 格式带时区 `YYYY-MM-DDTHH:MM:SS+HH:MM`
- **类型**：支出、收入、转账、债务变动、债权变动
- **金额**：正数表示资金流入，负数表示资金流出（部分交易类型）
- **去向**：支出类型显示 `-`，转账类型显示目标账户
- **ID 位置**：表格最后一列，便于复制使用
- **短 ID**：显示 Record ID 的前 12 位，平衡可读性与唯一性
- **移除/更新**：使用短 ID 即可，命令内部使用完整 ID 匹配
- **空值显示**：`-` 表示该字段为空

## 6.4 模糊搜索使用说明

模糊搜索支持对交易备注（description）字段进行不区分大小写的子串匹配。

### 基本用法

```bash
# 搜索包含"午餐"的交易
suanpan list --search "午餐"

# 搜索包含"鸡蛋"的交易（支持部分匹配）
suanpan list --search "鸡蛋"

# 组合时间范围和搜索
suanpan list --from 2026-01-01 --to 2026-01-31 --search "地铁"
```

### ⚠️ 注意事项

**1. --limit 参数与模糊搜索的交互**

模糊搜索是在数据库查询完成后，在内存中对结果进行过滤。因此 `--limit` 参数可能会在过滤前截断数据，导致搜索结果为空或不全。

```bash
# ❌ 可能返回空结果（如果前10条没有匹配项）
suanpan list --search "鸡蛋" --limit 10

# ✅ 正确做法：增大 limit 或使用时间范围缩小查询范围
suanpan list --search "鸡蛋" --limit 500
suanpan list --from 2026-01-01 --to 2026-01-31 --search "鸡蛋"
```

**建议**：使用模糊搜索时，建议设置较大的 `--limit` 值（如 500 或 1000），或配合 `--from/--to` 时间范围使用。

**2. 搜索范围**

模糊搜索目前仅支持搜索**备注（description）**字段，不支持搜索分类名称或账户名称。如需按分类筛选，请使用 `--category` 参数。

**3. 大小写不敏感**

搜索自动忽略大小写，以下命令效果相同：

```bash
suanpan list --search "XBOX"
suanpan list --search "xbox"
suanpan list --search "Xbox"
```

**4. 部分匹配**

搜索支持子串匹配，无需输入完整内容：

```bash
# 备注为"XBOX手柄摇杆头更换"，以下搜索都能匹配：
suanpan list --search "XBOX"
suanpan list --search "手柄"
suanpan list --search "摇杆"
```
