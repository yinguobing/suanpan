# 阶段一：MVP 基础功能

> **状态**: ✅ 已完成

## 目标

建立基础记账能力，验证核心交互流程。

## 功能范围

### ✅ 已实现功能

| 功能 | 命令 | 说明 |
|------|------|------|
| 结构化参数录入 | `suanpan add` | CLI 参数直接录入 |
| 流水记录存储 | - | 自动持久化到 SurrealDB |
| 流水查询 | `suanpan list` | 支持时间、分类筛选 |
| 流水移除 | `suanpan remove` | 通过短 ID 移除 |
| 流水更新 | `suanpan update` | 通过短 ID 更新指定字段 |
| 基础统计 | `suanpan stats` | 月度汇总、分类占比 |
| CLI 界面 | - | 完整的命令行交互 |

## 核心命令

```bash
# 添加支出
suanpan add -a 35 -f 支付宝 -t expense -c "餐饮/午餐" -d "午餐"

# 添加收入
suanpan add -a 8500 -t income -f 公司 -c "收入/工资" -d "三月工资"

# 添加转账
suanpan add -a 1000 -t transfer -f 招行卡 -o 支付宝 -c "转账"

# 列出最近流水
suanpan list --limit 20

# 按月统计
suanpan stats --month 2025-04
```

## 技术要点

- 使用 `rust_decimal::Decimal` 确保金额精度
- SurrealDB 嵌入式存储，单文件数据
- 交易记录使用 `account_from_id` 和 `account_to_id` 实现复式记账
- 短 ID 显示（前12位）便于用户操作
