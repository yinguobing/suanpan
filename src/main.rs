use anyhow::Result;
use clap::Parser;
use dirs::data_dir;
use std::path::PathBuf;

use finance_cli::commands::{add, list, stats, Cli, Commands};
use finance_cli::db::surreal::Database;

#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();

    // 获取数据目录
    let data_dir = get_data_dir()?;
    std::fs::create_dir_all(&data_dir)?;

    // 初始化数据库
    let db_path = data_dir.join("data.db");
    let db = Database::new(&db_path).await?;

    // 执行命令
    match cli.command {
        Commands::Add(args) => add::execute(&db, args).await?,
        Commands::List(args) => list::execute(&db, args).await?,
        Commands::Stats(args) => stats::execute(&db, args).await?,
    }

    Ok(())
}

fn get_data_dir() -> Result<PathBuf> {
    let base_dir = data_dir().ok_or_else(|| {
        anyhow::anyhow!("无法获取数据目录")
    })?;
    Ok(base_dir.join("finance-cli"))
}
