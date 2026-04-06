use clap::{Parser, Subcommand};

pub mod add;
pub mod list;
pub mod stats;

use add::AddArgs;
use list::ListArgs;
use stats::StatsArgs;

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
    /// 添加交易记录
    Add(AddArgs),
    /// 列出交易记录
    List(ListArgs),
    /// 统计报表
    Stats(StatsArgs),
}
