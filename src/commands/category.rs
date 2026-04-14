use clap::{Args, Subcommand};

use crate::db::surreal::Database;
use crate::error::Result;
use crate::models::{category_utils, CategoryRecord};
use crate::output::{
    print_empty_line, print_error, print_success, print_title, OutputFormat, OutputTable,
};

/// 分类管理子命令
#[derive(Subcommand)]
pub enum CategoryCommands {
    /// 列出所有分类
    List(ListArgs),
    /// 查看分类树
    Tree(TreeArgs),
    /// 添加分类
    Add(CategoryAddArgs),
    /// 重命名分类
    Rename(CategoryRenameArgs),
    /// 移除分类
    Remove(CategoryRemoveArgs),
}

/// 列出分类参数
#[derive(Args)]
pub struct ListArgs {
    /// 显示 ID 而非名称
    #[arg(long)]
    pub show_ids: bool,
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

/// 移除分类参数
#[derive(Args)]
pub struct CategoryRemoveArgs {
    /// 分类路径或ID
    pub path_or_id: String,
}

pub async fn execute(
    db: &Database,
    command: CategoryCommands,
    output_format: OutputFormat,
) -> Result<()> {
    match command {
        CategoryCommands::List(args) => list_categories(db, args, output_format).await,
        CategoryCommands::Tree(args) => show_tree(db, args, output_format).await,
        CategoryCommands::Add(args) => add_category(db, args, output_format).await,
        CategoryCommands::Rename(args) => rename_category(db, args, output_format).await,
        CategoryCommands::Remove(args) => remove_category(db, args, output_format).await,
    }
}

async fn list_categories(
    db: &Database,
    _args: ListArgs,
    output_format: OutputFormat,
) -> Result<()> {
    let categories = db.list_categories().await?;

    if categories.is_empty() {
        match output_format {
            OutputFormat::Machine => println!("NO_DATA"),
            OutputFormat::Human => println!("暂无分类"),
        }
        return Ok(());
    }

    // 按完整路径排序
    let mut sorted: Vec<&CategoryRecord> = categories.iter().collect();
    sorted.sort_by(|a, b| a.full_path.cmp(&b.full_path));

    match output_format {
        OutputFormat::Machine => {
            println!("ID|层级|父分类|完整路径");
            for cat in sorted {
                let parent = cat.parent_id.as_deref().unwrap_or("-");
                println!("{}|{}|{}|{}", cat.id, cat.level, parent, cat.full_path);
            }
        }
        OutputFormat::Human => {
            print_title("分类列表", output_format);
            print_empty_line();

            let mut table = OutputTable::new(output_format);
            table.set_header(vec!["ID", "层级", "父分类", "完整路径"]);

            for cat in sorted {
                let parent = cat.parent_id.as_deref().unwrap_or("-");
                table.add_row(vec![
                    &cat.id,
                    &cat.level.to_string(),
                    parent,
                    &cat.full_path,
                ]);
            }
            table.print();
            print_empty_line();
        }
    }

    Ok(())
}

async fn show_tree(db: &Database, args: TreeArgs, output_format: OutputFormat) -> Result<()> {
    let categories = db.list_categories().await?;

    if categories.is_empty() {
        match output_format {
            OutputFormat::Machine => println!("NO_DATA"),
            OutputFormat::Human => println!("暂无分类"),
        }
        return Ok(());
    }

    match output_format {
        OutputFormat::Machine => {
            // 机器可读格式：使用缩进路径表示层级
            // 构建树结构
            let mut roots: Vec<&CategoryRecord> = categories
                .iter()
                .filter(|c| c.parent_id.is_none())
                .collect();
            roots.sort_by(|a, b| a.name.cmp(&b.name));

            for root in roots {
                print_category_node_machine(root, &categories, args.depth, 0).await?;
            }
        }
        OutputFormat::Human => {
            print_title("分类树", output_format);
            print_empty_line();

            // 构建树结构
            let mut roots: Vec<&CategoryRecord> = categories
                .iter()
                .filter(|c| c.parent_id.is_none())
                .collect();
            roots.sort_by(|a, b| a.name.cmp(&b.name));

            for root in roots {
                print_category_node_human(root, &categories, "", args.depth, 0).await?;
            }
            print_empty_line();
        }
    }

    Ok(())
}

async fn print_category_node_machine(
    category: &CategoryRecord,
    all_categories: &[CategoryRecord],
    max_depth: Option<u32>,
    current_depth: u32,
) -> Result<()> {
    // 检查深度限制
    if let Some(max) = max_depth {
        if current_depth > max {
            return Ok(());
        }
    }

    // 机器可读格式：使用路径前缀表示层级
    let prefix = "/".repeat(current_depth as usize);
    println!("{}{}", prefix, category.name);

    // 获取子分类
    let mut children: Vec<&CategoryRecord> = all_categories
        .iter()
        .filter(|c| c.parent_id.as_ref() == Some(&category.id))
        .collect();
    children.sort_by(|a, b| a.name.cmp(&b.name));

    for child in children {
        Box::pin(print_category_node_machine(
            child,
            all_categories,
            max_depth,
            current_depth + 1,
        ))
        .await?;
    }

    Ok(())
}

async fn print_category_node_human(
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
        Box::pin(print_category_node_human(
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

async fn add_category(
    db: &Database,
    args: CategoryAddArgs,
    output_format: OutputFormat,
) -> Result<()> {
    let segments = category_utils::parse_path(&args.path);

    if segments.is_empty() {
        print_error("无效的分类路径", output_format);
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
            match output_format {
                OutputFormat::Machine => println!("EXISTING:{}:{}", current_path, existing.id),
                OutputFormat::Human => println!("  ✓ 已存在: {}", current_path),
            }
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
        match output_format {
            OutputFormat::Machine => println!("CREATED:{}:{}", current_path, created.id),
            OutputFormat::Human => println!("  ✓ 创建: {} ({})", current_path, created.id),
        }
        parent_id = Some(created.id);
    }

    print_empty_line();
    match output_format {
        OutputFormat::Machine => println!("DONE:{}", args.path),
        OutputFormat::Human => {
            print_success(&format!("分类路径已确保存在: {}", args.path), output_format)
        }
    }
    Ok(())
}

async fn rename_category(
    db: &Database,
    args: CategoryRenameArgs,
    output_format: OutputFormat,
) -> Result<()> {
    // 尝试按路径或ID查找
    let category = if let Some(cat) = db.get_category_by_path(&args.path_or_id).await? {
        cat
    } else if let Some(cat) = db.get_category(&args.path_or_id).await? {
        cat
    } else {
        print_error(&format!("分类不存在: {}", args.path_or_id), output_format);
        return Ok(());
    };

    let updated = db.update_category(&category.id, &args.new_name).await?;
    if let Some(cat) = updated {
        match output_format {
            OutputFormat::Machine => {
                println!(
                    "RENAMED:{}:{}:{}:{}",
                    category.id, category.full_path, cat.full_path, cat.id
                );
            }
            OutputFormat::Human => {
                print_success("分类已重命名:", output_format);
                println!("   原路径: {}", category.full_path);
                println!("   新路径: {}", cat.full_path);
            }
        }
    }

    Ok(())
}

async fn remove_category(
    db: &Database,
    args: CategoryRemoveArgs,
    output_format: OutputFormat,
) -> Result<()> {
    // 尝试按ID或路径查找
    let category = db.get_category_by_path(&args.path_or_id).await?;

    let category = match category {
        Some(c) => c,
        None => {
            // 尝试按ID查找
            if let Some(c) = db.get_category(&args.path_or_id).await? {
                c
            } else {
                print_error(&format!("分类不存在: {}", args.path_or_id), output_format);
                return Ok(());
            }
        }
    };

    // 检查是否有子分类
    let children = db.list_child_categories(&category.id).await?;
    if !children.is_empty() {
        match output_format {
            OutputFormat::Machine => {
                println!("WARN:HAS_CHILDREN:{}", children.len());
                for child in children {
                    println!("CHILD:{}:{}", child.name, child.id);
                }
            }
            OutputFormat::Human => {
                print_error(
                    &format!("该分类有 {} 个子分类，将一并移除", children.len()),
                    output_format,
                );
            }
        }
    }

    // 检查是否有关联的交易记录
    let tx_count = db.count_transactions_by_category(&category.id).await?;
    if tx_count > 0 {
        print_error(
            &format!("无法移除：该分类被 {} 条交易记录引用", tx_count),
            output_format,
        );
        match output_format {
            OutputFormat::Machine => println!("ERROR:HAS_TRANSACTIONS:{}", tx_count),
            OutputFormat::Human => println!("   请先移除或修改相关交易记录后再试"),
        }
        return Ok(());
    }

    let removed = db.delete_category(&category.id).await?;
    if removed {
        match output_format {
            OutputFormat::Machine => println!("REMOVED:{}:{}", category.full_path, category.id),
            OutputFormat::Human => print_success(
                &format!("分类已移除: {}", category.full_path),
                output_format,
            ),
        }
    }

    Ok(())
}
