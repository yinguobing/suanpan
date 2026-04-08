use clap::{Args, Subcommand};

use crate::db::surreal::Database;
use crate::error::Result;
use crate::models::{CategoryRecord, category_utils};

/// 分类管理子命令
#[derive(Subcommand)]
pub enum CategoryCommands {
    /// 查看分类树
    Tree(TreeArgs),
    /// 添加分类
    Add(CategoryAddArgs),
    /// 重命名分类
    Rename(CategoryRenameArgs),
    /// 删除分类
    Delete(CategoryDeleteArgs),
}

/// 查看分类树参数
#[derive(Args)]
pub struct TreeArgs {
    /// 只显示前N层
    #[arg(short, long)]
    pub depth: Option<u32>,
}

/// 添加分类参数
#[derive(Args)]
pub struct CategoryAddArgs {
    /// 分类路径（如 "餐饮/午餐"）
    pub path: String,
}

/// 重命名分类参数
#[derive(Args)]
pub struct CategoryRenameArgs {
    /// 原分类路径或ID
    pub path_or_id: String,
    /// 新名称
    pub new_name: String,
}

/// 删除分类参数
#[derive(Args)]
pub struct CategoryDeleteArgs {
    /// 分类路径或ID
    pub path_or_id: String,
}

pub async fn execute(db: &Database, command: CategoryCommands) -> Result<()> {
    match command {
        CategoryCommands::Tree(args) => show_tree(db, args).await,
        CategoryCommands::Add(args) => add_category(db, args).await,
        CategoryCommands::Rename(args) => rename_category(db, args).await,
        CategoryCommands::Delete(args) => delete_category(db, args).await,
    }
}

async fn show_tree(db: &Database, args: TreeArgs) -> Result<()> {
    let categories = db.list_categories().await?;

    if categories.is_empty() {
        println!("暂无分类");
        return Ok(());
    }

    println!("\n📂 分类树\n");

    // 构建树结构
    let mut roots: Vec<&CategoryRecord> = categories.iter().filter(|c| c.parent_id.is_none()).collect();
    roots.sort_by(|a, b| a.name.cmp(&b.name));

    for root in roots {
        print_category_node(db, root, &categories, "", args.depth, 0).await?;
    }
    println!();

    Ok(())
}

async fn print_category_node(
    _db: &Database,
    category: &CategoryRecord,
    all_categories: &[CategoryRecord],
    prefix: &str,
    max_depth: Option<u32>,
    current_depth: u32,
) -> Result<()> {
    // 检查深度限制
    if let Some(max) = max_depth {
        if current_depth > max {
            return Ok(());
        }
    }

    let connector = if current_depth == 0 { "" } else { "├── " };
    println!("{}{}{}", prefix, connector, category.name);

    // 获取子分类
    let mut children: Vec<&CategoryRecord> = all_categories
        .iter()
        .filter(|c| c.parent_id.as_ref() == Some(&category.id))
        .collect();
    children.sort_by(|a, b| a.name.cmp(&b.name));

    let new_prefix = if current_depth == 0 {
        ""
    } else if prefix.is_empty() {
        "│   "
    } else {
        &format!("{}    ", prefix)
    };

    for (i, child) in children.iter().enumerate() {
        let is_last = i == children.len() - 1;
        let child_prefix = if is_last && current_depth > 0 {
            prefix.to_string() + "    "
        } else {
            new_prefix.to_string()
        };
        Box::pin(print_category_node(
            _db,
            child,
            all_categories,
            &child_prefix,
            max_depth,
            current_depth + 1,
        ))
        .await?;
    }

    Ok(())
}

async fn add_category(db: &Database, args: CategoryAddArgs) -> Result<()> {
    let segments = category_utils::parse_path(&args.path);

    if segments.is_empty() {
        println!("❌ 无效的分类路径");
        return Ok(());
    }

    // 逐级创建分类
    let mut parent_id: Option<String> = None;
    let mut current_path = String::new();

    for (i, segment) in segments.iter().enumerate() {
        // 构建当前路径
        if current_path.is_empty() {
            current_path = segment.to_string();
        } else {
            current_path = format!("{}/{}", current_path, segment);
        }

        // 检查是否已存在
        if let Some(existing) = db.get_category_by_path(&current_path).await? {
            println!("  ✓ 已存在: {}", current_path);
            parent_id = Some(existing.id);
            continue;
        }

        // 创建新分类
        let id = format!("cat_{}", nanoid::nanoid!(8));
        let level = i as u32;
        let category = CategoryRecord {
            id,
            name: segment.to_string(),
            parent_id: parent_id.clone(),
            full_path: current_path.clone(),
            level,
            created_at: surrealdb::Datetime::from(chrono::Utc::now()),
        };

        let created = db.create_category(category).await?;
        println!("  ✓ 创建: {} ({})", current_path, created.id);
        parent_id = Some(created.id);
    }

    println!("\n✅ 分类路径已确保存在: {}", args.path);
    Ok(())
}

async fn rename_category(db: &Database, args: CategoryRenameArgs) -> Result<()> {
    // 尝试按路径或ID查找
    let category = if let Some(cat) = db.get_category_by_path(&args.path_or_id).await? {
        cat
    } else if let Some(cat) = db.get_category(&args.path_or_id).await? {
        cat
    } else {
        println!("❌ 分类不存在: {}", args.path_or_id);
        return Ok(());
    };

    let updated = db.update_category(&category.id, &args.new_name).await?;
    if let Some(cat) = updated {
        println!("✅ 分类已重命名:");
        println!("   原路径: {}", category.full_path);
        println!("   新路径: {}", cat.full_path);
    }

    Ok(())
}

async fn delete_category(db: &Database, args: CategoryDeleteArgs) -> Result<()> {
    // 尝试按ID或路径查找
    let category = db
        .get_category_by_path(&args.path_or_id)
        .await?;

    let category = match category {
        Some(c) => c,
        None => {
            // 尝试按ID查找
            if let Some(c) = db.get_category(&args.path_or_id).await? {
                c
            } else {
                println!("❌ 分类不存在: {}", args.path_or_id);
                return Ok(());
            }
        }
    };

    // 检查是否有子分类
    let children = db.list_child_categories(&category.id).await?;
    if !children.is_empty() {
        println!("⚠️  该分类有 {} 个子分类，将一并删除", children.len());
    }

    // 检查是否有关联的交易记录
    let tx_count = db.count_transactions_by_category(&category.id).await?;
    if tx_count > 0 {
        println!("❌ 无法删除：该分类被 {} 条交易记录引用", tx_count);
        println!("   请先删除或修改相关交易记录后再试");
        return Ok(());
    }

    let deleted = db.delete_category(&category.id).await?;
    if deleted {
        println!("✅ 分类已删除: {}", category.full_path);
    }

    Ok(())
}
