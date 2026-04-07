use clap::{Args, Subcommand};

use crate::db::surreal::Database;
use crate::error::Result;
use crate::models::Tag;

/// 标签管理子命令
#[derive(Subcommand)]
pub enum TagCommands {
    /// 列出所有标签
    List,
    /// 添加标签
    Add(TagAddArgs),
    /// 重命名标签
    Rename(TagRenameArgs),
    /// 删除标签
    Delete(TagDeleteArgs),
}

/// 添加标签参数
#[derive(Args)]
pub struct TagAddArgs {
    /// 标签名称
    pub name: String,

    /// 标签颜色（如 "#FF0000"）
    #[arg(long)]
    pub color: Option<String>,
}

/// 重命名标签参数
#[derive(Args)]
pub struct TagRenameArgs {
    /// 标签ID或名称
    pub id_or_name: String,
    /// 新名称
    pub new_name: String,
}

/// 删除标签参数
#[derive(Args)]
pub struct TagDeleteArgs {
    /// 标签ID或名称
    pub id_or_name: String,
}

pub async fn execute(db: &Database, command: TagCommands) -> Result<()> {
    match command {
        TagCommands::List => list_tags(db).await,
        TagCommands::Add(args) => add_tag(db, args).await,
        TagCommands::Rename(args) => rename_tag(db, args).await,
        TagCommands::Delete(args) => delete_tag(db, args).await,
    }
}

async fn list_tags(db: &Database) -> Result<()> {
    let tags = db.list_tags().await?;

    if tags.is_empty() {
        println!("暂无标签");
        return Ok(());
    }

    println!("\n🏷️ 标签列表\n");
    println!("{:<20} {:<10} {}", "ID", "颜色", "名称");
    println!("{}", "-".repeat(50));

    for tag in tags {
        let color = tag.color.as_deref().unwrap_or("-");
        println!("{:<20} {:<10} {}", tag.id, color, tag.name);
    }
    println!();

    Ok(())
}

async fn add_tag(db: &Database, args: TagAddArgs) -> Result<()> {
    // 检查是否已存在同名标签
    if let Some(existing) = db.find_tag_by_name(&args.name).await? {
        println!("❌ 已存在同名标签: {} (ID: {})", existing.name, existing.id);
        return Ok(());
    }

    // 验证颜色格式
    if let Some(ref color) = args.color {
        if !Tag::is_valid_color(color) {
            println!("❌ 无效的颜色格式: {} (应为 #RGB 或 #RRGGBB)", color);
            return Ok(());
        }
    }

    // 生成ID
    let id = format!("tag_{}", nanoid::nanoid!(8));
    let mut tag = Tag::new(&id, &args.name);

    if let Some(color) = args.color {
        tag = tag.with_color(color);
    }

    let created = db.create_tag(tag).await?;
    println!("✅ 标签已创建:");
    println!("   ID: {}", created.id);
    println!("   名称: {}", created.name);
    if let Some(color) = &created.color {
        println!("   颜色: {}", color);
    }

    Ok(())
}

async fn rename_tag(db: &Database, args: TagRenameArgs) -> Result<()> {
    // 尝试查找标签
    let tag = if let Some(tag) = db.find_tag_by_name(&args.id_or_name).await? {
        tag
    } else if let Some(tag) = db.get_tag(&args.id_or_name).await? {
        tag
    } else {
        println!("❌ 标签不存在: {}", args.id_or_name);
        return Ok(());
    };

    // 检查新名称是否已被使用
    if let Some(existing) = db.find_tag_by_name(&args.new_name).await? {
        if existing.id != tag.id {
            println!("❌ 名称 '{}' 已被标签 {} 使用", args.new_name, existing.id);
            return Ok(());
        }
    }

    let updated = db.update_tag(&tag.id, &args.new_name).await?;
    if let Some(t) = updated {
        println!("✅ 标签已重命名: {} -> {}", tag.name, t.name);
    }

    Ok(())
}

async fn delete_tag(db: &Database, args: TagDeleteArgs) -> Result<()> {
    // 尝试查找标签
    let tag = if let Some(tag) = db.find_tag_by_name(&args.id_or_name).await? {
        tag
    } else if let Some(tag) = db.get_tag(&args.id_or_name).await? {
        tag
    } else {
        println!("❌ 标签不存在: {}", args.id_or_name);
        return Ok(());
    };

    // TODO: 从所有Transaction中移除该标签ID
    // 这需要先查询所有包含该标签的交易，然后更新它们

    let deleted = db.delete_tag(&tag.id).await?;
    if deleted {
        println!("✅ 标签已删除: {} ({})", tag.name, tag.id);
    }

    Ok(())
}
