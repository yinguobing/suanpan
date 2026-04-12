use chrono::NaiveDate;
use clap::Args;
use comfy_table::Table;
use rust_decimal::Decimal;

use crate::db::surreal::Database;
use crate::error::Result;

/// 时间段对比分析（环比/同比）
#[derive(Args)]
pub struct CompareArgs {
    /// 对比月份（格式：YYYY-MM）
    #[arg(short, long)]
    pub month: String,

    /// 对比类型（mom: 环比, yoy: 同比, both: 两者）
    #[arg(short, long, default_value = "both")]
    pub compare_type: String,
}

pub async fn execute(db: &Database, args: CompareArgs) -> Result<()> {
    // 解析目标月份
    let (year, month) = parse_month(&args.month)?;
    
    // 解析对比类型
    let compare_type = parse_compare_type(&args.compare_type)?;

    // 计算目标月份的起止时间
    let target_start = NaiveDate::from_ymd_opt(year, month, 1)
        .ok_or_else(|| crate::error::FinanceError::Validation("无效的日期".to_string()))?;
    let target_end = if month == 12 {
        NaiveDate::from_ymd_opt(year + 1, 1, 1)
    } else {
        NaiveDate::from_ymd_opt(year, month + 1, 1)
    }
    .ok_or_else(|| crate::error::FinanceError::Validation("无效的日期".to_string()))?;

    // 获取目标月份数据
    let target_stats = db
        .get_stats_by_date_range(
            target_start.and_hms_opt(0, 0, 0).unwrap().and_utc(),
            target_end.and_hms_opt(0, 0, 0).unwrap().and_utc(),
        )
        .await?;

    println!("\n[报表] {}年{}月 财务对比分析\n", year, month);

    // 打印目标月份概览
    print_overview(&target_stats);

    // 根据对比类型执行对比
    match compare_type {
        CompareType::Mom => {
            print_mom_comparison(db, year, month, &target_stats).await?;
        }
        CompareType::Yoy => {
            print_yoy_comparison(db, year, month, &target_stats).await?;
        }
        CompareType::Both => {
            print_mom_comparison(db, year, month, &target_stats).await?;
            print_yoy_comparison(db, year, month, &target_stats).await?;
        }
    }

    Ok(())
}

/// 对比类型
enum CompareType {
    Mom,   // 环比（与上月对比）
    Yoy,   // 同比（与去年同月对比）
    Both,  // 两者
}

/// 打印概览
fn print_overview(stats: &crate::db::surreal::PeriodStats) {
    let mut table = Table::new();
    table.set_header(vec!["项目", "金额", "笔数"]);
    table.add_row(vec!["总收入", &format!("¥{}", stats.total_income), &stats.transaction_count.to_string()]);
    table.add_row(vec!["总支出", &format!("¥{}", stats.total_expense), ""]);
    
    let net = stats.total_income - stats.total_expense;
    let net_str = if net >= Decimal::ZERO {
        format!("+¥{}", net)
    } else {
        format!("-¥{}", net.abs())
    };
    table.add_row(vec!["净收支", &net_str, ""]);
    
    println!("{}", table);
    println!();
}

/// 打印环比对比
async fn print_mom_comparison(
    db: &Database,
    year: i32,
    month: u32,
    target_stats: &crate::db::surreal::PeriodStats,
) -> Result<()> {
    // 计算上月
    let (prev_year, prev_month) = if month == 1 {
        (year - 1, 12)
    } else {
        (year, month - 1)
    };

    let prev_start = NaiveDate::from_ymd_opt(prev_year, prev_month, 1)
        .ok_or_else(|| crate::error::FinanceError::Validation("无效的日期".to_string()))?;
    let prev_end = NaiveDate::from_ymd_opt(year, month, 1)
        .ok_or_else(|| crate::error::FinanceError::Validation("无效的日期".to_string()))?;

    let prev_stats = db
        .get_stats_by_date_range(
            prev_start.and_hms_opt(0, 0, 0).unwrap().and_utc(),
            prev_end.and_hms_opt(0, 0, 0).unwrap().and_utc(),
        )
        .await?;

    println!("[对比] 环比对比（vs {}年{}月）", prev_year, prev_month);
    println!();

    let mut table = Table::new();
    table.set_header(vec!["项目", "本月", "上月", "变化", "变化率"]);

    // 收入对比
    let income_change = target_stats.total_income - prev_stats.total_income;
    let income_change_pct = if prev_stats.total_income > Decimal::ZERO {
        (income_change / prev_stats.total_income * Decimal::from(100)).round_dp(1)
    } else {
        Decimal::ZERO
    };
    table.add_row(vec![
        "收入",
        &format!("¥{}", target_stats.total_income),
        &format!("¥{}", prev_stats.total_income),
        &format_change(income_change),
        &format_change_pct(income_change_pct),
    ]);

    // 支出对比
    let expense_change = target_stats.total_expense - prev_stats.total_expense;
    let expense_change_pct = if prev_stats.total_expense > Decimal::ZERO {
        (expense_change / prev_stats.total_expense * Decimal::from(100)).round_dp(1)
    } else {
        Decimal::ZERO
    };
    table.add_row(vec![
        "支出",
        &format!("¥{}", target_stats.total_expense),
        &format!("¥{}", prev_stats.total_expense),
        &format_change(expense_change),
        &format_change_pct(expense_change_pct),
    ]);

    // 净收支对比
    let target_net = target_stats.total_income - target_stats.total_expense;
    let prev_net = prev_stats.total_income - prev_stats.total_expense;
    let net_change = target_net - prev_net;
    let net_change_pct = if prev_net != Decimal::ZERO {
        (net_change / prev_net.abs() * Decimal::from(100)).round_dp(1)
    } else {
        Decimal::ZERO
    };
    table.add_row(vec![
        "净收支",
        &format_net(target_net),
        &format_net(prev_net),
        &format_change(net_change),
        &format_change_pct(net_change_pct),
    ]);

    // 交易笔数对比
    let count_change = target_stats.transaction_count as i64 - prev_stats.transaction_count as i64;
    let count_change_pct = if prev_stats.transaction_count > 0 {
        (Decimal::from(count_change) / Decimal::from(prev_stats.transaction_count) * Decimal::from(100)).round_dp(1)
    } else {
        Decimal::ZERO
    };
    table.add_row(vec![
        "交易笔数",
        &target_stats.transaction_count.to_string(),
        &prev_stats.transaction_count.to_string(),
        &format!("{:+}", count_change),
        &format_change_pct(count_change_pct),
    ]);

    println!("{}", table);
    println!();

    Ok(())
}

/// 打印同比对比
async fn print_yoy_comparison(
    db: &Database,
    year: i32,
    month: u32,
    target_stats: &crate::db::surreal::PeriodStats,
) -> Result<()> {
    // 计算去年同期
    let last_year = year - 1;

    let yoy_start = NaiveDate::from_ymd_opt(last_year, month, 1)
        .ok_or_else(|| crate::error::FinanceError::Validation("无效的日期".to_string()))?;
    let yoy_end = if month == 12 {
        NaiveDate::from_ymd_opt(last_year + 1, 1, 1)
    } else {
        NaiveDate::from_ymd_opt(last_year, month + 1, 1)
    }
    .ok_or_else(|| crate::error::FinanceError::Validation("无效的日期".to_string()))?;

    let yoy_stats = db
        .get_stats_by_date_range(
            yoy_start.and_hms_opt(0, 0, 0).unwrap().and_utc(),
            yoy_end.and_hms_opt(0, 0, 0).unwrap().and_utc(),
        )
        .await?;

    println!("[对比] 同比对比（vs {}年{}月）", last_year, month);
    println!();

    let mut table = Table::new();
    table.set_header(vec!["项目", "本月", "去年同期", "变化", "变化率"]);

    // 收入对比
    let income_change = target_stats.total_income - yoy_stats.total_income;
    let income_change_pct = if yoy_stats.total_income > Decimal::ZERO {
        (income_change / yoy_stats.total_income * Decimal::from(100)).round_dp(1)
    } else {
        Decimal::ZERO
    };
    table.add_row(vec![
        "收入",
        &format!("¥{}", target_stats.total_income),
        &format!("¥{}", yoy_stats.total_income),
        &format_change(income_change),
        &format_change_pct(income_change_pct),
    ]);

    // 支出对比
    let expense_change = target_stats.total_expense - yoy_stats.total_expense;
    let expense_change_pct = if yoy_stats.total_expense > Decimal::ZERO {
        (expense_change / yoy_stats.total_expense * Decimal::from(100)).round_dp(1)
    } else {
        Decimal::ZERO
    };
    table.add_row(vec![
        "支出",
        &format!("¥{}", target_stats.total_expense),
        &format!("¥{}", yoy_stats.total_expense),
        &format_change(expense_change),
        &format_change_pct(expense_change_pct),
    ]);

    // 净收支对比
    let target_net = target_stats.total_income - target_stats.total_expense;
    let yoy_net = yoy_stats.total_income - yoy_stats.total_expense;
    let net_change = target_net - yoy_net;
    let net_change_pct = if yoy_net != Decimal::ZERO {
        (net_change / yoy_net.abs() * Decimal::from(100)).round_dp(1)
    } else {
        Decimal::ZERO
    };
    table.add_row(vec![
        "净收支",
        &format_net(target_net),
        &format_net(yoy_net),
        &format_change(net_change),
        &format_change_pct(net_change_pct),
    ]);

    // 交易笔数对比
    let count_change = target_stats.transaction_count as i64 - yoy_stats.transaction_count as i64;
    let count_change_pct = if yoy_stats.transaction_count > 0 {
        (Decimal::from(count_change) / Decimal::from(yoy_stats.transaction_count) * Decimal::from(100)).round_dp(1)
    } else {
        Decimal::ZERO
    };
    table.add_row(vec![
        "交易笔数",
        &target_stats.transaction_count.to_string(),
        &yoy_stats.transaction_count.to_string(),
        &format!("{:+}", count_change),
        &format_change_pct(count_change_pct),
    ]);

    println!("{}", table);
    println!();

    Ok(())
}

/// 格式化变化（带符号）
fn format_change(change: Decimal) -> String {
    if change >= Decimal::ZERO {
        format!("+¥{}", change)
    } else {
        format!("-¥{}", change.abs())
    }
}

/// 格式化变化率（带符号和百分比）
fn format_change_pct(change_pct: Decimal) -> String {
    if change_pct >= Decimal::ZERO {
        format!("+{}%", change_pct)
    } else {
        format!("{}%", change_pct)
    }
}

/// 格式化净收支（带符号）
fn format_net(net: Decimal) -> String {
    if net >= Decimal::ZERO {
        format!("+¥{}", net)
    } else {
        format!("-¥{}", net.abs())
    }
}

/// 解析月份
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

/// 解析对比类型
fn parse_compare_type(compare_type: &str) -> Result<CompareType> {
    match compare_type.to_lowercase().as_str() {
        "mom" | "m" | "环比" => Ok(CompareType::Mom),
        "yoy" | "y" | "同比" => Ok(CompareType::Yoy),
        "both" | "b" | "all" | "全部" => Ok(CompareType::Both),
        _ => Err(crate::error::FinanceError::Validation(format!(
            "无效的对比类型: {}，支持 mom/yoy/both",
            compare_type
        ))),
    }
}
