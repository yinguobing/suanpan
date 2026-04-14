use clap::{Args, Subcommand};

use crate::db::surreal::Database;
use crate::error::Result;
use crate::models::{Account, AccountType};
use crate::output::{
    print_empty_line, print_error, print_success, print_title, OutputFormat, OutputTable,
};

/// 账户管理子命令
#[derive(Subcommand)]
pub enum AccountCommands {
    /// 列出所有账户
    List,
    /// 添加账户
    Add(AccountAddArgs),
    /// 重命名账户
    Rename(AccountRenameArgs),
    /// 移除账户
    Remove(AccountRemoveArgs),
}

/// 添加账户参数
#[derive(Args)]
pub struct AccountAddArgs {
    /// 账户名称
    pub name: String,

    /// 账户类型
    #[arg(short, long, value_enum)]
    pub account_type: AccountTypeArg,

    /// 父账户ID（用于子账户）
    #[arg(long)]
    pub parent: Option<String>,
}

/// 重命名账户参数
#[derive(Args)]
pub struct AccountRenameArgs {
    /// 账户ID
    pub id: String,
    /// 新名称
    pub new_name: String,
}

/// 移除账户参数
#[derive(Args)]
pub struct AccountRemoveArgs {
    /// 账户ID
    pub id: String,
}

#[derive(Clone, Debug, clap::ValueEnum)]
pub enum AccountTypeArg {
    BankCard,
    EWallet,
    Cash,
    Investment,
    Credit,
    Other,
}

impl From<AccountTypeArg> for AccountType {
    fn from(arg: AccountTypeArg) -> Self {
        match arg {
            AccountTypeArg::BankCard => AccountType::BankCard,
            AccountTypeArg::EWallet => AccountType::EWallet,
            AccountTypeArg::Cash => AccountType::Cash,
            AccountTypeArg::Investment => AccountType::Investment,
            AccountTypeArg::Credit => AccountType::Credit,
            AccountTypeArg::Other => AccountType::Other,
        }
    }
}

pub async fn execute(
    db: &Database,
    command: AccountCommands,
    output_format: OutputFormat,
) -> Result<()> {
    match command {
        AccountCommands::List => list_accounts(db, output_format).await,
        AccountCommands::Add(args) => add_account(db, args, output_format).await,
        AccountCommands::Rename(args) => rename_account(db, args, output_format).await,
        AccountCommands::Remove(args) => remove_account(db, args, output_format).await,
    }
}

async fn list_accounts(db: &Database, output_format: OutputFormat) -> Result<()> {
    let accounts = db.list_accounts().await?;

    if accounts.is_empty() {
        match output_format {
            OutputFormat::Machine => println!("NO_DATA"),
            OutputFormat::Human => println!("暂无账户"),
        }
        return Ok(());
    }

    match output_format {
        OutputFormat::Machine => {
            println!("ID|类型|父账户|名称");
            for account in accounts {
                let parent = account.parent_id.as_deref().unwrap_or("-");
                println!(
                    "{}|{}|{}|{}",
                    account.id, account.account_type, parent, account.name
                );
            }
        }
        OutputFormat::Human => {
            print_title("账户列表", output_format);
            print_empty_line();

            let mut table = OutputTable::new(output_format);
            table.set_header(vec!["ID", "类型", "父账户", "名称"]);

            for account in accounts {
                let parent = account.parent_id.as_deref().unwrap_or("-");
                table.add_row(vec![
                    &account.id,
                    &account.account_type.to_string(),
                    parent,
                    &account.name,
                ]);
            }
            table.print();
            print_empty_line();
        }
    }

    Ok(())
}

async fn add_account(
    db: &Database,
    args: AccountAddArgs,
    output_format: OutputFormat,
) -> Result<()> {
    // 检查是否已存在同名账户
    if let Some(existing) = db.find_account_by_name(&args.name).await? {
        print_error(
            &format!("已存在同名账户: {} (ID: {})", existing.name, existing.id),
            output_format,
        );
        return Ok(());
    }

    // 生成ID
    let id = format!("acc_{}", nanoid::nanoid!(8));
    let account_type: AccountType = args.account_type.into();

    let mut account = Account::new(&id, &args.name, account_type);

    // 如果有父账户，验证父账户存在
    if let Some(parent_id) = args.parent {
        if db.get_account(&parent_id).await?.is_none() {
            print_error(&format!("父账户不存在: {}", parent_id), output_format);
            return Ok(());
        }
        account = account.with_parent(parent_id);
    }

    let created = db.create_account(account).await?;

    match output_format {
        OutputFormat::Machine => {
            println!(
                "CREATED:{}:{}:{}:{}",
                created.id,
                created.name,
                created.account_type,
                created.parent_id.as_deref().unwrap_or("-")
            );
        }
        OutputFormat::Human => {
            print_success("账户已创建:", output_format);
            println!("   ID: {}", created.id);
            println!("   名称: {}", created.name);
            println!("   类型: {}", created.account_type);
            if let Some(parent) = &created.parent_id {
                println!("   父账户: {}", parent);
            }
        }
    }

    Ok(())
}

async fn rename_account(
    db: &Database,
    args: AccountRenameArgs,
    output_format: OutputFormat,
) -> Result<()> {
    // 检查账户是否存在
    if db.get_account(&args.id).await?.is_none() {
        print_error(&format!("账户不存在: {}", args.id), output_format);
        return Ok(());
    }

    // 检查新名称是否已被使用
    if let Some(existing) = db.find_account_by_name(&args.new_name).await? {
        if existing.id != args.id {
            print_error(
                &format!("名称 '{}' 已被账户 {} 使用", args.new_name, existing.id),
                output_format,
            );
            return Ok(());
        }
    }

    let updated = db.update_account(&args.id, &args.new_name).await?;
    if let Some(account) = updated {
        match output_format {
            OutputFormat::Machine => {
                println!("RENAMED:{}:{}", args.id, account.name);
            }
            OutputFormat::Human => {
                print_success(
                    &format!("账户已重命名: {} -> {}", args.id, account.name),
                    output_format,
                );
            }
        }
    }

    Ok(())
}

async fn remove_account(
    db: &Database,
    args: AccountRemoveArgs,
    output_format: OutputFormat,
) -> Result<()> {
    // 检查账户是否存在
    if db.get_account(&args.id).await?.is_none() {
        print_error(&format!("账户不存在: {}", args.id), output_format);
        return Ok(());
    }

    // 检查是否有子账户
    let children = db.list_child_accounts(&args.id).await?;
    if !children.is_empty() {
        print_error(
            &format!("无法移除，该账户有 {} 个子账户", children.len()),
            output_format,
        );
        match output_format {
            OutputFormat::Machine => {
                for child in children {
                    println!("CHILD:{}:{}", child.name, child.id);
                }
            }
            OutputFormat::Human => {
                println!("   请先移除子账户:");
                for child in children {
                    println!("   - {} ({})", child.name, child.id);
                }
            }
        }
        return Ok(());
    }

    // 检查是否有关联的交易记录
    let tx_count = db.count_transactions_by_account(&args.id).await?;
    if tx_count > 0 {
        print_error(
            &format!("无法移除：该账户被 {} 条交易记录引用", tx_count),
            output_format,
        );
        match output_format {
            OutputFormat::Machine => println!("ERROR:HAS_TRANSACTIONS:{}", tx_count),
            OutputFormat::Human => println!("   请先移除或修改相关交易记录后再试"),
        }
        return Ok(());
    }

    let removed = db.delete_account(&args.id).await?;
    if removed {
        match output_format {
            OutputFormat::Machine => println!("REMOVED:{}", args.id),
            OutputFormat::Human => {
                print_success(&format!("账户已移除: {}", args.id), output_format)
            }
        }
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_account_type_arg_conversion() {
        assert!(matches!(
            AccountType::from(AccountTypeArg::BankCard),
            AccountType::BankCard
        ));
        assert!(matches!(
            AccountType::from(AccountTypeArg::EWallet),
            AccountType::EWallet
        ));
    }
}
