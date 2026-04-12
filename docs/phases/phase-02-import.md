# 阶段二：数据整合

> **状态**: ✅ 已完成

## 目标

实现第三方数据导入，支持历史财务数据的迁移和整合。

## 功能范围

### ✅ 已实现功能

| 功能 | 命令 | 说明 |
|------|------|------|
| 随手记导入 | `suanpan import suishouji` | 支持 XLS/XLSX/CSV 格式 |
| 多 Sheet 支持 | - | 自动处理 Excel 多 Sheet |
| 自动创建账户 | - | 导入时自动创建不存在的账户 |
| 自动创建分类 | - | 导入时自动创建不存在的分类 |
| 重复检测 | - | 基于时间+金额+账户+描述 |
| 预览模式 | `--dry-run` | 预览导入结果，不实际写入 |
| 跳过去重 | `--skip-dedup` | 强制导入所有记录 |

## 核心命令

```bash
# 导入随手记 XLSX 文件
suanpan import suishouji ./path/to/suishouji.xlsx

# 预览导入（不实际写入）
suanpan import suishouji ./path/to/suishouji.xlsx --dry-run

# 跳过重复检测
suanpan import suishouji ./path/to/suishouji.xlsx --skip-dedup
```

## 重复检测机制

基于以下字段组合进行精确匹配：
- `timestamp` - 交易时间
- `amount` - 金额
- `account_from_id` - 来源账户
- `description` - 描述

如果所有字段都相同，则视为重复记录。

## 数据映射

随手记字段映射到 suanpan 数据模型：

| 随手记字段 | suanpan 字段 | 说明 |
|------------|--------------|------|
| 时间 | timestamp | 交易时间 |
| 金额 | amount | 金额（正数） |
| 类型 | tx_type | 支出/收入/转账 |
| 账户 | account_from_id | 来源账户 |
| 账户2 | account_to_id | 目标账户（转账） |
| 分类 | category_id | 分类（支持层级） |
| 项目 | tag_ids | 映射为标签 |
| 备注 | description | 描述 |

## 技术要点

- 使用 `calamine` 库读取 Excel 文件
- 自动识别文件格式（XLS/XLSX/CSV）
- 导入过程显示进度条
- 支持事务回滚，确保数据一致性
