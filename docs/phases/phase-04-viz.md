# 阶段四：可视化与预算

> **状态**: 📝 计划中

## 目标

提供更丰富的数据可视化能力，并引入预算管理功能。

## 规划功能

### 预算管理

| 功能 | 命令 | 说明 |
|------|------|------|
| 设置月度预算 | `suanpan budget set --month YYYY-MM --amount <金额>` | 设定月度总预算 |
| 设置分类预算 | `suanpan budget set --category <分类> --amount <金额>` | 设定分类预算 |
| 查看预算 | `suanpan budget show --month YYYY-MM` | 查看预算执行情况 |
| 预算提醒 | - | 预算超支提醒 |
| 预算分析 | `suanpan budget analyze` | 预算执行分析 |

### 高级可视化

| 功能 | 说明 |
|------|------|
| 热力图 | 支出热力图（日历视图） |
| 桑基图 | 资金流向图 |
| 预测曲线 | 基于历史数据的趋势预测 |
| 自定义报表 | 支持用户自定义报表模板 |

## 规划命令

```bash
# 设置月度预算
suanpan budget set --month 2026-04 --amount 5000

# 设置分类预算
suanpan budget set --category "餐饮" --amount 1500

# 查看预算执行情况
suanpan budget show --month 2026-04

# 预算分析
suanpan budget analyze

# 生成热力图报表
suanpan report --type heatmap --month 2026-04

# 生成资金流向图
suanpan report --type sankey --month 2026-04
```

## 待定实现

- [ ] 预算数据模型设计
- [ ] 预算执行追踪
- [ ] 超支提醒机制
- [ ] 热力图可视化
- [ ] 桑基图可视化
- [ ] 趋势预测算法
