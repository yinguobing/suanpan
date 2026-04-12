use clap::{Parser, Subcommand};

pub mod account;
pub mod add;
pub mod category;
pub mod compare;
pub mod import;
pub mod list;
pub mod migrate;
pub mod remove;
pub mod report;
pub mod stats;
pub mod tag;
pub mod trend;
pub mod update;

use account::AccountCommands;
use add::AddArgs;
use category::CategoryCommands;
use compare::CompareArgs;
use import::ImportArgs;
use list::ListArgs;
use migrate::MigrateArgs;
use remove::RemoveArgs;
use report::ReportArgs;
use stats::StatsArgs;
use tag::TagCommands;
use trend::TrendArgs;
use update::UpdateArgs;

/// 国冰财务管理系统 CLI
#[derive(Parser)]
#[command(name = "finance")]
#[command(about = "个人财务管理工具")]
#[command(version)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Subcommand)]
pub enum Commands {
    /// 账户管理
    Account {
        #[command(subcommand)]
        command: AccountCommands,
    },
    /// 添加交易记录
    Add(AddArgs),
    /// 分类管理
    Category {
        #[command(subcommand)]
        command: CategoryCommands,
    },
    /// 列出交易记录
    List(ListArgs),
    /// 数据迁移
    Migrate(MigrateArgs),
    /// 移除交易记录
    Remove(RemoveArgs),
    /// 生成可视化报表
    Report(ReportArgs),
    /// 对比分析（环比/同比）
    Compare(CompareArgs),
    /// 统计报表
    Stats(StatsArgs),
    /// 标签管理
    Tag {
        #[command(subcommand)]
        command: TagCommands,
    },
    /// 趋势分析
    Trend(TrendArgs),
    /// 更新交易记录
    Update(UpdateArgs),
    /// 导入交易记录（支持 XLS/XLSX/CSV）
    Import(ImportArgs),
}
