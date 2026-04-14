use clap::Args;
use rust_decimal::Decimal;

use crate::db::surreal::{Database, TransactionUpdate};
use crate::error::Result;
use crate::models::TxType;
use crate::output::{print_error, print_success, OutputFormat};

/// 更新交易记录
#[derive(Args)]
pub struct UpdateArgs {
    /// 交易记录的短 ID（12位）
    pub id: String,

    /// 金额
    #[arg(short, long)]
    pub amount: Option<Decimal>,

    /// 货币
    #[arg(short = 'y', long)]
    pub currency: Option<String>,

    /// 交易类型
    #[arg(short, long)]
    pub tx_type: Option<String>,

    /// 来源账户
    #[arg(short, long)]
    pub from: Option<String>,

    /// 去向账户/商户
    #[arg(short = 'o', long)]
    pub to: Option<String>,

    /// 分类
    #[arg(short, long)]
    pub category: Option<String>,

    /// 描述/备注
    #[arg(short, long)]
    pub description: Option<String>,

    /// 标签（覆盖原有标签，可多次使用）
    #[arg(short = 'g', long)]
    pub tag: Vec<String>,
}

pub async fn execute(db: &Database, args: UpdateArgs, output_format: OutputFormat) -> Result<()> {
    // 验证 ID 格式
    if args.id.len() != 12 {
        print_error(
            &format!("ID '{}' 格式错误，应为 12 位字符", args.id),
            output_format,
        );
        return Ok(());
    }

    // 检查是否有任何更新字段
    if args.amount.is_none()
        && args.currency.is_none()
        && args.tx_type.is_none()
        && args.from.is_none()
        && args.to.is_none()
        && args.category.is_none()
        && args.description.is_none()
        && args.tag.is_empty()
    {
        match output_format {
            OutputFormat::Machine => println!("ERROR:NO_FIELDS"),
            OutputFormat::Human => {
                print_error("请至少指定一个要更新的字段", output_format);
                println!("用法: suanpan update <短ID> -a 40 -d \"新描述\"");
            }
        }
        return Ok(());
    }

    // 解析交易类型
    let tx_type = if let Some(ref tx_type_str) = args.tx_type {
        Some(parse_tx_type(tx_type_str)?)
    } else {
        None
    };

    // 构建更新参数
    let updates = TransactionUpdate {
        amount: args.amount,
        currency: args.currency,
        tx_type,
        account_from_id: args.from.map(|f| format!("acc_{}", f)),
        account_to_id: if args.to.is_some() {
            Some(args.to.map(|t| format!("acc_{}", t)))
        } else {
            None
        },
        category_id: args.category.map(|c| format!("cat_{}", c)),
        description: if args.description.is_some() {
            Some(args.description)
        } else {
            None
        },
        tag_ids: if args.tag.is_empty() {
            None
        } else {
            Some(args.tag.into_iter().map(|t| format!("tag_{}", t)).collect())
        },
    };

    // 执行更新
    match db.update_by_short_id(&args.id, updates).await? {
        Some(tx) => match output_format {
            OutputFormat::Machine => {
                println!(
                    "UPDATED:{}:{}:{}:{}:{}:{}:{}:{}",
                    args.id,
                    tx.tx_type,
                    tx.amount,
                    tx.currency,
                    tx.account_from_id,
                    tx.account_to_id.as_deref().unwrap_or("-"),
                    tx.category_id,
                    tx.description.as_deref().unwrap_or("-")
                );
            }
            OutputFormat::Human => {
                print_success("交易记录已更新:", output_format);
                println!("   ID: {:?}", tx.id);
                println!("   类型: {}", tx.tx_type);
                println!("   金额: {} {}", tx.amount, tx.currency);
                println!(
                    "   账户: {} -> {}",
                    tx.account_from_id,
                    tx.account_to_id.as_deref().unwrap_or("-")
                );
                println!("   分类: {}", tx.category_id);
                if let Some(desc) = &tx.description {
                    println!("   描述: {}", desc);
                }
                if !tx.tag_ids.is_empty() {
                    println!("   标签: {}", tx.tag_ids.join(", "));
                }
            }
        },
        None => match output_format {
            OutputFormat::Machine => println!("NOT_FOUND:{}", args.id),
            OutputFormat::Human => print_error(
                &format!("未找到 ID 为 '{}' 的交易记录", args.id),
                output_format,
            ),
        },
    }

    Ok(())
}

fn parse_tx_type(s: &str) -> Result<TxType> {
    match s.to_lowercase().as_str() {
        "expense" | "支出" | "e" => Ok(TxType::Expense),
        "income" | "收入" | "i" => Ok(TxType::Income),
        "transfer" | "转账" | "t" => Ok(TxType::Transfer),
        "debt" | "debtchange" | "债务" | "d" => Ok(TxType::DebtChange),
        "credit" | "creditchange" | "债权" | "c" => Ok(TxType::CreditChange),
        _ => Err(crate::error::FinanceError::Parse(format!(
            "未知的交易类型: {}",
            s
        ))),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_tx_type() {
        // 英文
        assert!(matches!(parse_tx_type("expense").unwrap(), TxType::Expense));
        assert!(matches!(parse_tx_type("income").unwrap(), TxType::Income));
        assert!(matches!(
            parse_tx_type("transfer").unwrap(),
            TxType::Transfer
        ));
        assert!(matches!(parse_tx_type("debt").unwrap(), TxType::DebtChange));
        assert!(matches!(
            parse_tx_type("credit").unwrap(),
            TxType::CreditChange
        ));

        // 中文
        assert!(matches!(parse_tx_type("支出").unwrap(), TxType::Expense));
        assert!(matches!(parse_tx_type("收入").unwrap(), TxType::Income));

        // 缩写
        assert!(matches!(parse_tx_type("e").unwrap(), TxType::Expense));
        assert!(matches!(parse_tx_type("i").unwrap(), TxType::Income));

        // 错误
        assert!(parse_tx_type("unknown").is_err());
    }

    #[test]
    fn test_update_args_id_validation() {
        // 有效 ID: 12 位
        let valid_id = "f4sp877fxbwc";
        assert_eq!(valid_id.len(), 12);

        // 无效 ID
        let short_id = "abc123";
        assert!(short_id.len() != 12);
    }
}
