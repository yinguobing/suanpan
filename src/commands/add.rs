use clap::Args;
use rust_decimal::Decimal;

use crate::db::surreal::Database;
use crate::error::Result;
use crate::models::{Transaction, TxType};
use crate::output::{print_success, OutputFormat};

/// 添加交易记录
#[derive(Args)]
pub struct AddArgs {
    /// 金额
    #[arg(short, long)]
    pub amount: Decimal,

    /// 交易类型
    #[arg(short, long, value_enum, default_value = "expense")]
    pub tx_type: TxTypeArg,

    /// 来源账户
    #[arg(short, long)]
    pub from: String,

    /// 去向账户（可选）
    #[arg(short = 'o', long)]
    pub to: Option<String>,

    /// 分类
    #[arg(short, long, default_value = "其他")]
    pub category: String,

    /// 描述/备注
    #[arg(short, long)]
    pub description: Option<String>,

    /// 货币（默认 CNY）
    #[arg(short = 'y', long, default_value = "CNY")]
    pub currency: String,

    /// 标签（可多次使用）
    #[arg(short = 'g', long)]
    pub tag: Vec<String>,
}

#[derive(Clone, Debug)]
pub enum TxTypeArg {
    Expense,
    Income,
    Transfer,
    DebtChange,
    CreditChange,
}

impl std::str::FromStr for TxTypeArg {
    type Err = String;

    fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "expense" | "支出" | "e" => Ok(TxTypeArg::Expense),
            "income" | "收入" | "i" => Ok(TxTypeArg::Income),
            "transfer" | "转账" | "t" => Ok(TxTypeArg::Transfer),
            "debt" | "debtchange" | "债务" | "d" => Ok(TxTypeArg::DebtChange),
            "credit" | "creditchange" | "债权" | "c" => Ok(TxTypeArg::CreditChange),
            _ => Err(format!("未知的交易类型: {}", s)),
        }
    }
}

impl From<TxTypeArg> for TxType {
    fn from(arg: TxTypeArg) -> Self {
        match arg {
            TxTypeArg::Expense => TxType::Expense,
            TxTypeArg::Income => TxType::Income,
            TxTypeArg::Transfer => TxType::Transfer,
            TxTypeArg::DebtChange => TxType::DebtChange,
            TxTypeArg::CreditChange => TxType::CreditChange,
        }
    }
}

pub async fn execute(db: &Database, args: AddArgs, output_format: OutputFormat) -> Result<()> {
    let tx_type: TxType = args.tx_type.into();

    // 查找或创建来源账户
    let account_from = db.find_or_create_account_by_name(&args.from).await?;

    // 查找或创建去向账户（可选）
    let account_to_id = if let Some(to_name) = args.to {
        let account_to = db.find_or_create_account_by_name(&to_name).await?;
        Some(account_to.id)
    } else {
        None
    };

    // 查找或创建分类
    let category = db.find_or_create_category_by_path(&args.category).await?;

    // 查找或创建标签
    let mut tag_ids = Vec::new();
    for tag_name in args.tag {
        let tag = db.find_or_create_tag_by_name(&tag_name).await?;
        tag_ids.push(tag.id);
    }

    let transaction = Transaction::new(
        args.amount,
        args.currency,
        tx_type,
        account_from.id,
        account_to_id,
        category.id,
        args.description,
    )
    .with_tag_ids(tag_ids);

    let created = db.create_transaction(transaction).await?;

    match output_format {
        OutputFormat::Machine => {
            println!(
                "CREATED:{}:{}:{}:{}:{}:{}:{}:{}",
                format_short_id(&created.id),
                created.tx_type,
                created.amount,
                created.currency,
                created.account_from_id,
                created.account_to_id.as_deref().unwrap_or("-"),
                created.category_id,
                created.description.as_deref().unwrap_or("-")
            );
        }
        OutputFormat::Human => {
            print_success("交易记录已创建:", output_format);
            println!("   ID: {:?}", created.id);
            println!("   类型: {}", created.tx_type);
            println!("   金额: {} {}", created.amount, created.currency);
            println!(
                "   账户: {} -> {}",
                created.account_from_id,
                created.account_to_id.as_deref().unwrap_or("-")
            );
            println!("   分类: {}", created.category_id);
            if let Some(desc) = &created.description {
                println!("   描述: {}", desc);
            }
            if !created.tag_ids.is_empty() {
                println!("   标签: {}", created.tag_ids.join(", "));
            }
        }
    }

    Ok(())
}

/// 从 RecordId 提取短 ID（前12位）
fn format_short_id(id: &Option<surrealdb::RecordId>) -> String {
    match id {
        Some(rid) => {
            let full_id = rid.to_string();
            full_id
                .split(':')
                .nth(1)
                .map(|s| s.chars().take(12).collect())
                .unwrap_or_else(|| "unknown".to_string())
        }
        None => "unknown".to_string(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::str::FromStr;

    #[test]
    fn test_tx_type_arg_from_str() {
        // 英文
        assert!(matches!(
            TxTypeArg::from_str("expense").unwrap(),
            TxTypeArg::Expense
        ));
        assert!(matches!(
            TxTypeArg::from_str("income").unwrap(),
            TxTypeArg::Income
        ));
        assert!(matches!(
            TxTypeArg::from_str("transfer").unwrap(),
            TxTypeArg::Transfer
        ));
        assert!(matches!(
            TxTypeArg::from_str("debt").unwrap(),
            TxTypeArg::DebtChange
        ));
        assert!(matches!(
            TxTypeArg::from_str("credit").unwrap(),
            TxTypeArg::CreditChange
        ));

        // 中文
        assert!(matches!(
            TxTypeArg::from_str("支出").unwrap(),
            TxTypeArg::Expense
        ));
        assert!(matches!(
            TxTypeArg::from_str("收入").unwrap(),
            TxTypeArg::Income
        ));
        assert!(matches!(
            TxTypeArg::from_str("转账").unwrap(),
            TxTypeArg::Transfer
        ));
        assert!(matches!(
            TxTypeArg::from_str("债务").unwrap(),
            TxTypeArg::DebtChange
        ));
        assert!(matches!(
            TxTypeArg::from_str("债权").unwrap(),
            TxTypeArg::CreditChange
        ));

        // 缩写
        assert!(matches!(
            TxTypeArg::from_str("e").unwrap(),
            TxTypeArg::Expense
        ));
        assert!(matches!(
            TxTypeArg::from_str("i").unwrap(),
            TxTypeArg::Income
        ));
        assert!(matches!(
            TxTypeArg::from_str("t").unwrap(),
            TxTypeArg::Transfer
        ));
        assert!(matches!(
            TxTypeArg::from_str("d").unwrap(),
            TxTypeArg::DebtChange
        ));
        assert!(matches!(
            TxTypeArg::from_str("c").unwrap(),
            TxTypeArg::CreditChange
        ));

        // 错误情况
        assert!(TxTypeArg::from_str("unknown").is_err());
    }

    #[test]
    fn test_tx_type_arg_into_tx_type() {
        let tx_type: TxType = TxTypeArg::Expense.into();
        assert!(matches!(tx_type, TxType::Expense));

        let tx_type: TxType = TxTypeArg::Income.into();
        assert!(matches!(tx_type, TxType::Income));

        let tx_type: TxType = TxTypeArg::Transfer.into();
        assert!(matches!(tx_type, TxType::Transfer));

        let tx_type: TxType = TxTypeArg::DebtChange.into();
        assert!(matches!(tx_type, TxType::DebtChange));

        let tx_type: TxType = TxTypeArg::CreditChange.into();
        assert!(matches!(tx_type, TxType::CreditChange));
    }
}
