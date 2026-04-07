use clap::{Parser, Subcommand};

pub mod account;
pub mod add;
pub mod category;
pub mod delete;
pub mod list;
pub mod stats;
pub mod tag;
pub mod update;

use account::AccountCommands;
use add::AddArgs;
use category::CategoryCommands;
use delete::DeleteArgs;
use list::ListArgs;
use stats::StatsArgs;
use tag::TagCommands;
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
    /// 删除交易记录
    Delete(DeleteArgs),
    /// 列出交易记录
    List(ListArgs),
    /// 统计报表
    Stats(StatsArgs),
    /// 标签管理
    Tag {
        #[command(subcommand)]
        command: TagCommands,
    },
    /// 更新交易记录
    Update(UpdateArgs),
}
