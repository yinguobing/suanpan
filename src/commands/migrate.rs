use clap::Args;

use crate::db::surreal::Database;
use crate::error::Result;
use crate::output::{print_empty_line, print_warning, OutputFormat};

/// 数据迁移参数
#[derive(Args)]
pub struct MigrateArgs {
    /// 预览模式（不实际执行）
    #[arg(long)]
    pub dry_run: bool,

    /// 跳过备份提示
    #[arg(long)]
    pub skip_backup: bool,
}

pub async fn execute(db: &Database, args: MigrateArgs, output_format: OutputFormat) -> Result<()> {
    if !args.skip_backup {
        match output_format {
            OutputFormat::Machine => print_warning("BACKUP_RECOMMENDED", output_format),
            OutputFormat::Human => {
                println!("[WARN] 警告：数据迁移会修改数据库结构");
                println!("   建议先备份数据文件: ~/.local/share/suanpan/data.db");
                println!("   使用 --skip-backup 跳过此提示");
                print_empty_line();
            }
        }
    }

    if args.dry_run {
        match output_format {
            OutputFormat::Machine => println!("DRY_RUN"),
            OutputFormat::Human => {
                println!("[预览] 预览模式：将显示将要执行的更改，但不会实际修改数据\n")
            }
        }
    } else {
        match output_format {
            OutputFormat::Machine => println!("MIGRATING"),
            OutputFormat::Human => println!("[开始] 开始数据迁移...\n"),
        }
    }

    // 执行迁移
    let stats = db.migrate_data(args.dry_run).await?;

    if stats.transactions_migrated == 0 && stats.accounts_created == 0 {
        match output_format {
            OutputFormat::Machine => println!("NO_CHANGES"),
            OutputFormat::Human => println!("✓ 没有找到需要迁移的数据，可能已迁移完成"),
        }
        return Ok(());
    }

    match output_format {
        OutputFormat::Machine => {
            println!("TRANSACTIONS:{}", stats.transactions_migrated);
            println!("ACCOUNTS:{}", stats.accounts_created);
            println!("CATEGORIES:{}", stats.categories_created);
            println!("TAGS:{}", stats.tags_created);
        }
        OutputFormat::Human => {
            println!("\n[结果] 迁移结果：");
            println!("   交易记录迁移: {}", stats.transactions_migrated);
            println!("   账户创建: {}", stats.accounts_created);
            println!("   分类创建: {}", stats.categories_created);
            println!("   标签创建: {}", stats.tags_created);
        }
    }

    if args.dry_run {
        match output_format {
            OutputFormat::Machine => println!("DRY_RUN_COMPLETE"),
            OutputFormat::Human => {
                println!("\n[信息] 这是预览模式，实际数据未修改");
                println!("   移除 --dry-run 参数执行实际迁移");
            }
        }
    } else {
        match output_format {
            OutputFormat::Machine => println!("MIGRATION_COMPLETE"),
            OutputFormat::Human => {
                println!("\n[OK] 数据迁移完成！");
                println!("   现在可以使用新的 account/category/tag 管理命令了");
            }
        }
    }

    Ok(())
}
