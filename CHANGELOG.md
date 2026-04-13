# Changelog

所有重要变更都将记录在此文件中。

格式基于 [Keep a Changelog](https://keepachangelog.com/zh-CN/1.0.0/)，
并且本项目遵循 [Semantic Versioning](https://semver.org/lang/zh-CN/)。

## [Unreleased]

### Added
- 初始版本开发中

## [0.1.0] - 2026-04-12

### Added
- 核心记账功能：支出、收入、转账、债务、债权交易类型
- 账户管理：添加、列出、重命名、移除账户
- 分类管理：多级分类体系，支持层级汇总
- 标签管理：为交易添加标签
- 交易查询：模糊搜索、日期范围筛选、导出 CSV
- 统计分析：月度统计、分类统计、账户统计
- 趋势分析：按日/周/月/季度/年查看趋势
- 对比分析：环比/同比分析
- 可视化报表：生成 HTML 报表（饼图、折线图、柱状图）
- 数据导入：支持随手记 XLS/XLSX/CSV 导入，自动去重
- 嵌入式数据库：使用 SurrealKV 存储，单文件便于备份
