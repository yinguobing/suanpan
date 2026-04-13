# 算盘 (suanpan)

智能体时代的个人财务管理 CLI 工具。算盘是中国传统计算工具，象征精确与效率。

基于 Rust + SurrealDB 构建，支持本地数据存储和多种交易类型管理。

将此页面发给你的智能体，让它协助你管理自己的财务。

## 特性

- **本地优先**：所有数据存储在本地，无需网络连接
- **嵌入式数据库**：使用 SurrealKV 存储，单文件便于备份
- **多种交易类型**：支持支出、收入、转账、债务、债权
- **统计报表**：月度收支统计、分类占比、趋势分析、对比分析
- **智能查询**：模糊搜索、组合筛选、导出 CSV
- **HTML 报表**：生成简洁的数据报表（表格形式展示）
- **数据导入**：支持随手记 XLS/XLSX/CSV 导入，自动去重
- **灵活分类**：多级分类体系，支持层级汇总
- **精确计算**：使用 Decimal 类型避免浮点精度问题

## 安装

```bash
# 克隆仓库
git clone https://github.com/yinguobing/suanpan.git
cd suanpan

# 编译发布版本
cargo build --release

# 安装到系统
sudo cp target/release/suanpan /usr/local/bin/
```

**依赖**: Rust 1.82+

## 快速开始

```bash
# 添加支出
suanpan add -a 35 -f 支付宝 -c "餐饮/午餐" -d "工作午餐"

# 添加收入
suanpan add -a 8500 -t income -f 公司 -c "收入/工资" -d "三月工资"

# 查看最近记录
suanpan list

# 查看月度统计
suanpan stats --month 2026-04

# 查看趋势分析
suanpan trend --period month

# 生成 HTML 报表
suanpan report --month 2026-04
```

## 常用命令

### 交易管理

```bash
# 添加交易
suanpan add -a <金额> -f <来源账户> -c <分类> [-t <类型>] [-d <备注>]

# 列出记录
suanpan list [--from <日期>] [--to <日期>] [--search <关键词>] [--output <文件>]

# 更新记录
suanpan update <ID> [-a <金额>] [-d <备注>]

# 移除记录
suanpan remove <ID>
```

### 统计分析

```bash
# 月度统计
suanpan stats [--month YYYY-MM] [--by-category] [--by-account]

# 趋势分析
suanpan trend --period <day|week|month|quarter|year> [--from <日期>] [--to <日期>]

# 对比分析（环比/同比）
suanpan compare --month YYYY-MM [--compare-type mom|yoy|both]

# 生成报表
suanpan report --month YYYY-MM [--output <目录>]
```

### 数据管理

```bash
# 导入随手记
suanpan import suishouji <文件路径> [--dry-run]

# 账户管理
suanpan account <list|add|rename|remove>

# 分类管理
suanpan category <tree|add|rename|remove>

# 标签管理
suanpan tag <list|add|rename|remove>
```

**完整命令参考** → [docs/commands.md](./docs/commands.md)

## 交易类型

| 类型 | 说明 | 示例 |
|------|------|------|
| `expense` | 支出（默认） | 购物、餐饮、交通 |
| `income` | 收入 | 工资、理财收益 |
| `transfer` | 转账 | 账户间资金转移 |
| `debtchange` | 债务变动 | 借入/偿还债务 |
| `creditchange` | 债权变动 | 借出/收回借款 |

## 数据存储

数据默认存储在：

- **Linux**: `~/.local/share/suanpan/data.db`
- **macOS**: `~/Library/Application Support/suanpan/data.db`
- **Windows**: `%APPDATA%\suanpan\data.db`

### 备份

```bash
# 备份
cp ~/.local/share/suanpan/data.db ~/suanpan-backup-$(date +%Y%m%d).db

# 恢复
cp ~/suanpan-backup-20260406.db ~/.local/share/suanpan/data.db
```

## 技术栈

Rust | SurrealDB | clap | chrono | rust_decimal

## 文档

- [docs/commands.md](./docs/commands.md) - 完整命令参考
- [docs/phases/](./docs/phases/) - 开发文档

## 许可证

GPL-3.0

## 作者

尹国冰
