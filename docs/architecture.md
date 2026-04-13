# 技术架构

## 4.1 技术栈

| 层级 | 选型 | 理由 |
|------|------|------|
| 编程语言 | Rust | 类型安全、性能优异、Decimal 精度保障 |
| 数据库 | SurrealDB | 嵌入式模式、现代 API、无需手写 SQL、支持多模型查询 |
| CLI 框架 | clap | Rust 生态标准 |
| 序列化 | serde | 与 SurrealDB 原生集成 |
| 时间处理 | chrono | Rust 标准 |
| 金额计算 | rust_decimal | 避免浮点误差 |

## 4.2 项目结构

```
suanpan/
├── Cargo.toml
├── src/
│   ├── main.rs              # CLI 入口
│   ├── lib.rs
│   ├── models/              # 数据模型
│   │   ├── mod.rs
│   │   ├── transaction.rs   # 交易记录
│   │   ├── account.rs       # 账户
│   │   ├── category.rs      # 分类
│   │   ├── tag.rs           # 标签
│   │   └── types.rs         # 枚举类型
│   ├── db/                  # 数据库层
│   │   ├── mod.rs
│   │   └── surreal.rs       # SurrealDB 封装
│   ├── commands/            # CLI 子命令
│   │   ├── mod.rs
│   │   ├── add.rs           # 添加交易
│   │   ├── remove.rs        # 移除交易
│   │   ├── update.rs        # 更新交易
│   │   ├── list.rs          # 查询交易（支持组合筛选）
│   │   ├── stats.rs         # 统计分析
│   │   ├── trend.rs         # 趋势分析
│   │   ├── compare.rs       # 对比分析（环比/同比）
│   │   ├── report.rs        # HTML 数据报表
│   │   ├── migrate.rs       # 数据迁移
│   │   ├── import.rs        # 数据导入
│   │   ├── account.rs       # 账户管理
│   │   ├── category.rs      # 分类管理
│   │   └── tag.rs           # 标签管理
│   └── error.rs             # 错误处理
```

## 4.3 安全与隐私

### 本地优先
- 所有数据存储在本地文件系统
- 无需网络连接
- 数据文件可由用户完全控制

### 备份策略
- 数据库文件天然适合 Git 版本控制（SurrealDB 单文件存储）
- 定期导出为可读的 JSON/CSV 备份

## 4.4 术语表

| 术语 | 定义 |
|------|------|
| Transaction | 交易记录，资金流动的一次记录 |
| TxType | 交易类型（支出/收入/转账/债务/债权） |
| Account | 账户，资金的容器（银行卡、支付宝、现金等） |
| Category | 分类，标准化的财务类别 |
| Tag | 标签，自由输入的标记，可多选 |
| Metadata | 元数据，机器生成的扩展信息 |
| Decimal | 定点数，用于精确表示金额 |
