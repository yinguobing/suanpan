use clap::Args;
use rust_decimal::Decimal;

use crate::db::surreal::Database;
use crate::error::Result;
use crate::models::{Transaction, TxType};

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

pub async fn execute(db: &Database, args: AddArgs) -> Result<()> {
    let tx_type: TxType = args.tx_type.into();

    let transaction = Transaction::new(
        args.amount,
        args.currency,
        tx_type,
        args.from,
        args.to,
        args.category,
        args.description,
    )
    .with_tags(args.tag);

    let created = db.create_transaction(transaction).await?;

    println!("✅ 交易记录已创建:");
    println!("   ID: {:?}", created.id);
    println!("   类型: {}", created.tx_type);
    println!("   金额: {} {}", created.amount, created.currency);
    println!("   账户: {} -> {}", created.account_from, created.account_to.as_deref().unwrap_or("-"));
    println!("   分类: {}", created.category);
    if let Some(desc) = &created.description {
        println!("   描述: {}", desc);
    }
    if !created.tags.is_empty() {
        println!("   标签: {}", created.tags.join(", "));
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::str::FromStr;

    #[test]
    fn test_tx_type_arg_from_str() {
        // 英文
        assert!(matches!(TxTypeArg::from_str("expense").unwrap(), TxTypeArg::Expense));
        assert!(matches!(TxTypeArg::from_str("income").unwrap(), TxTypeArg::Income));
        assert!(matches!(TxTypeArg::from_str("transfer").unwrap(), TxTypeArg::Transfer));
        assert!(matches!(TxTypeArg::from_str("debt").unwrap(), TxTypeArg::DebtChange));
        assert!(matches!(TxTypeArg::from_str("credit").unwrap(), TxTypeArg::CreditChange));

        // 中文
        assert!(matches!(TxTypeArg::from_str("支出").unwrap(), TxTypeArg::Expense));
        assert!(matches!(TxTypeArg::from_str("收入").unwrap(), TxTypeArg::Income));
        assert!(matches!(TxTypeArg::from_str("转账").unwrap(), TxTypeArg::Transfer));
        assert!(matches!(TxTypeArg::from_str("债务").unwrap(), TxTypeArg::DebtChange));
        assert!(matches!(TxTypeArg::from_str("债权").unwrap(), TxTypeArg::CreditChange));

        // 缩写
        assert!(matches!(TxTypeArg::from_str("e").unwrap(), TxTypeArg::Expense));
        assert!(matches!(TxTypeArg::from_str("i").unwrap(), TxTypeArg::Income));
        assert!(matches!(TxTypeArg::from_str("t").unwrap(), TxTypeArg::Transfer));
        assert!(matches!(TxTypeArg::from_str("d").unwrap(), TxTypeArg::DebtChange));
        assert!(matches!(TxTypeArg::from_str("c").unwrap(), TxTypeArg::CreditChange));

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
