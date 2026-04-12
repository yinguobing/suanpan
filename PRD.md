# 算盘 (Suanpan) PRD

> 智能体时代的个人财务管理 CLI 工具

## 快速导航

完整产品文档已拆分到 `docs/` 目录：

| 文档 | 内容 |
|------|------|
| [docs/README.md](./docs/README.md) | 文档索引 |
| [docs/overview.md](./docs/overview.md) | 产品概述、核心理念、目标用户 |
| [docs/architecture.md](./docs/architecture.md) | 技术架构、技术栈、项目结构 |
| [docs/database.md](./docs/database.md) | 数据模型设计、数据库存储 |
| [docs/commands.md](./docs/commands.md) | CLI 交互设计、命令参考 |

## 阶段规划

| 阶段 | 名称 | 状态 | 文档 |
|------|------|------|------|
| 一 | MVP 基础功能 | ✅ 已完成 | [docs/phases/phase-01-core.md](./docs/phases/phase-01-core.md) |
| 二 | 数据整合 | ✅ 已完成 | [docs/phases/phase-02-import.md](./docs/phases/phase-02-import.md) |
| 三 | 数据分析 | ✅ 已完成 | [docs/phases/phase-03-analysis.md](./docs/phases/phase-03-analysis.md) |
| 四 | 可视化与预算 | 📝 计划中 | [docs/phases/phase-04-viz.md](./docs/phases/phase-04-viz.md) |
| 五 | 智能分析 | 📝 计划中 | [docs/phases/phase-05-ai.md](./docs/phases/phase-05-ai.md) |

## 核心理念

- **CLI 优先**：通过结构化命令行参数进行交互，简洁高效
- **资金流动为核心**：所有财务活动抽象为资金的流动，简化数据模型
- **渐进式演进**：分阶段迭代，MVP 聚焦核心记账和查询功能

## 快速开始

```bash
# 添加一笔支出
suanpan add -a 35 -f 支付宝 -t expense -c "餐饮/午餐" -d "午餐"

# 查看月度统计
suanpan stats --month 2026-04

# 查看趋势分析
suanpan trend --period month

# 生成可视化报表
suanpan report --month 2026-04
```

---

*详见 [docs/](./docs/) 目录了解完整产品文档*
