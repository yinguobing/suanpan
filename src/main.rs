use anyhow::Result;
use clap::Parser;
use dirs::data_dir;
use std::path::PathBuf;

use suanpan::commands::{account, add, category, compare, import, list, migrate, remove, report, stats, tag, trend, update, Cli, Commands};
use suanpan::db::surreal::Database;

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
        Commands::Account { command } => account::execute(&db, command).await?,
        Commands::Add(args) => add::execute(&db, args).await?,
        Commands::Category { command } => category::execute(&db, command).await?,
        Commands::Compare(args) => compare::execute(&db, args).await?,
        Commands::Import(args) => import::execute(&db, args).await?,
        Commands::List(args) => list::execute(&db, args).await?,
        Commands::Migrate(args) => migrate::execute(&db, args).await?,
        Commands::Remove(args) => remove::execute(&db, args).await?,
        Commands::Report(args) => report::execute(&db, args).await?,
        Commands::Stats(args) => stats::execute(&db, args).await?,
        Commands::Tag { command } => tag::execute(&db, command).await?,
        Commands::Trend(args) => trend::execute(&db, args).await?,
        Commands::Update(args) => update::execute(&db, args).await?,
    }

    Ok(())
}

fn get_data_dir() -> Result<PathBuf> {
    let base_dir = data_dir().ok_or_else(|| {
        anyhow::anyhow!("无法获取数据目录")
    })?;
    let new_dir = base_dir.join("suanpan");
    let old_dir = base_dir.join("finance-cli");
    
    // 如果新目录不存在且旧目录存在，执行迁移
    if !new_dir.exists() && old_dir.exists() {
        println!("📦 检测到旧版本数据，正在迁移...");
        std::fs::create_dir_all(&new_dir)?;
        
        // 复制所有文件
        for entry in std::fs::read_dir(&old_dir)? {
            let entry = entry?;
            let src = entry.path();
            let dst = new_dir.join(entry.file_name());
            if src.is_file() {
                std::fs::copy(&src, &dst)?;
            }
        }
        println!("✅ 数据迁移完成！旧数据保留在: {}", old_dir.display());
        println!();
    }
    
    Ok(new_dir)
}
