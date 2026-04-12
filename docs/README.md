# 算盘 (Suanpan) 产品文档

> 智能体时代的个人财务管理 CLI 工具

## 文档索引

| 文档 | 内容 |
|------|------|
| [overview.md](./overview.md) | 产品概述、核心理念、目标用户 |
| [architecture.md](./architecture.md) | 技术架构、技术栈、项目结构 |
| [database.md](./database.md) | 数据模型设计、数据库存储 |
| [commands.md](./commands.md) | CLI 交互设计、命令参考 |
| [phases/phase-01-core.md](./phases/phase-01-core.md) | 阶段一：MVP 基础功能 |
| [phases/phase-02-import.md](./phases/phase-02-import.md) | 阶段二：数据整合与导入 |
| [phases/phase-03-analysis.md](./phases/phase-03-analysis.md) | 阶段三：数据分析 |

## 快速导航

### 核心理念
- **CLI 优先**：通过结构化命令行参数进行交互，简洁高效
- **资金流动为核心**：所有财务活动抽象为资金的流动，简化数据模型
- **渐进式演进**：分阶段迭代，MVP 聚焦核心记账和查询功能

### 主要功能
- 📊 **记账**：快速录入收支、转账、债务变动
- 📁 **导入**：支持随手记 CSV/XLSX 导入，自动去重
- 📈 **统计**：月度汇总、分类占比、趋势分析
- 🔍 **查询**：模糊搜索、组合筛选、导出 CSV
- 📊 **报表**：HTML 可视化报表（饼图、折线图、柱状图）

### 技术栈
- **语言**: Rust
- **数据库**: SurrealDB（嵌入式）
- **CLI 框架**: clap

---

*详见各分文档了解完整内容*
