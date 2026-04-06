use chrono::{Datelike, Local};
use clap::Args;
use comfy_table::Table;
use rust_decimal::Decimal;

use crate::db::surreal::Database;
use crate::error::Result;

/// 统计报表
#[derive(Args)]
pub struct StatsArgs {
    /// 月份（格式：YYYY-MM）
    #[arg(short, long)]
    pub month: Option<String>,

    /// 按分类统计
    #[arg(long)]
    pub by_category: bool,
}

pub async fn execute(db: &Database, args: StatsArgs) -> Result<()> {
    let now = Local::now();
    let (year, month) = if let Some(month_str) = args.month {
        parse_month(&month_str)?
    } else {
        (now.year(), now.month())
    };

    let stats = db.get_monthly_stats(year, month).await?;

    println!("\n📊 {}年{}月 财务统计\n", year, month);

    // 基本统计
    let mut table = Table::new();
    table.set_header(vec!["项目", "金额"]);
    table.add_row(vec!["总收入", &format!("¥{}", stats.total_income)]);
    table.add_row(vec!["总支出", &format!("¥{}", stats.total_expense)]);
    
    let net_color = if stats.net >= Decimal::ZERO { "+" } else { "" };
    table.add_row(vec!["净收支", &format!("{}{}¥{}", net_color, if stats.net >= Decimal::ZERO { "" } else { "" }, stats.net)]);
    table.add_row(vec!["交易笔数", &stats.transaction_count.to_string()]);
    println!("{}", table);

    // 分类统计
    if args.by_category && !stats.category_breakdown.is_empty() {
        println!("\n📈 支出分类占比\n");
        let mut cat_table = Table::new();
        cat_table.set_header(vec!["分类", "金额", "占比"]);

        let mut categories: Vec<_> = stats.category_breakdown.iter().collect();
        categories.sort_by(|a, b| b.1.cmp(a.1));

        for (category, amount) in categories {
            let percentage = if stats.total_expense > Decimal::ZERO {
                (*amount / stats.total_expense * Decimal::from(100)).round_dp(1)
            } else {
                Decimal::ZERO
            };
            cat_table.add_row(vec![
                category,
                &format!("¥{}", amount),
                &format!("{}%", percentage),
            ]);
        }
        println!("{}", cat_table);
    }

    Ok(())
}

fn parse_month(month_str: &str) -> Result<(i32, u32)> {
    let parts: Vec<&str> = month_str.split('-').collect();
    if parts.len() != 2 {
        return Err(crate::error::FinanceError::Parse(
            "月份格式错误，应为 YYYY-MM".to_string(),
        ));
    }

    let year: i32 = parts[0]
        .parse()
        .map_err(|_| crate::error::FinanceError::Parse("年份格式错误".to_string()))?;
    let month: u32 = parts[1]
        .parse()
        .map_err(|_| crate::error::FinanceError::Parse("月份格式错误".to_string()))?;

    if month < 1 || month > 12 {
        return Err(crate::error::FinanceError::Parse(
            "月份应在 1-12 之间".to_string(),
        ));
    }

    Ok((year, month))
}
