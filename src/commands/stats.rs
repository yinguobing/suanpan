use chrono::{Datelike, Local, NaiveDate};
use clap::Args;
use comfy_table::Table;
use rust_decimal::Decimal;

use crate::db::surreal::{Database, HierarchicalCategoryStats, PeriodStats};
use crate::db::MonthlyStats;
use crate::error::Result;

/// 统计报表
#[derive(Args)]
pub struct StatsArgs {
    /// 月份（格式：YYYY-MM）
    #[arg(short, long, group = "time_range")]
    pub month: Option<String>,

    /// 起始日期（格式：YYYY-MM-DD）
    #[arg(long, group = "time_range")]
    pub from: Option<String>,

    /// 结束日期（格式：YYYY-MM-DD，需与 --from 同时使用）
    #[arg(long, requires = "from")]
    pub to: Option<String>,

    /// 按分类统计
    #[arg(long)]
    pub by_category: bool,

    /// 按账户统计
    #[arg(long)]
    pub by_account: bool,

    /// 指定账户统计（账户ID或名称）
    #[arg(long)]
    pub account: Option<String>,

    /// 显示分类 ID 而非名称
    #[arg(long)]
    pub show_ids: bool,
}

pub async fn execute(db: &Database, args: StatsArgs) -> Result<()> {
    // 处理账户统计
    if args.by_account || args.account.is_some() {
        let (from_dt, to_dt) = if let Some(from) = &args.from {
            // 自定义日期范围
            let from_date = parse_date(from)?;
            let to_date = match &args.to {
                Some(to_str) => parse_date(to_str)?,
                None => Local::now().date_naive(),
            };

            if from_date > to_date {
                return Err(crate::error::FinanceError::Validation(
                    "起始日期不能晚于结束日期".to_string(),
                ));
            }

            let from_dt = from_date.and_hms_opt(0, 0, 0).unwrap().and_utc();
            let to_dt = to_date.and_hms_opt(23, 59, 59).unwrap().and_utc();
            (Some(from_dt), Some(to_dt))
        } else if let Some(month_str) = &args.month {
            // 指定月份
            let (year, month) = parse_month(month_str)?;
            let start = NaiveDate::from_ymd_opt(year, month, 1)
                .ok_or_else(|| crate::error::FinanceError::Validation("无效的日期".to_string()))?;
            let end = if month == 12 {
                NaiveDate::from_ymd_opt(year + 1, 1, 1)
            } else {
                NaiveDate::from_ymd_opt(year, month + 1, 1)
            }
            .ok_or_else(|| crate::error::FinanceError::Validation("无效的日期".to_string()))?;

            let from_dt = start.and_hms_opt(0, 0, 0).unwrap().and_utc();
            let to_dt = end.and_hms_opt(0, 0, 0).unwrap().and_utc();
            (Some(from_dt), Some(to_dt))
        } else {
            // 不限定日期范围
            (None, None)
        };

        // 解析账户参数（可能是ID或名称）
        let account_filter = if let Some(acc) = &args.account {
            // 先尝试作为ID查找，如果找不到则作为名称查找
            if let Some(account) = db.get_account(acc).await? {
                Some(account.id)
            } else if let Some(account) = db.find_account_by_name(acc).await? {
                Some(account.id)
            } else {
                return Err(crate::error::FinanceError::Validation(format!(
                    "未找到账户: {}",
                    acc
                )));
            }
        } else {
            None
        };

        let account_stats = db
            .get_stats_by_account(account_filter.as_deref(), from_dt, to_dt)
            .await?;

        // 如果指定了特定账户，过滤只显示该账户
        let filtered_stats: Vec<_> = if args.account.is_some() {
            account_stats
                .into_iter()
                .filter(|s| Some(&s.account_id) == account_filter.as_ref())
                .collect()
        } else {
            account_stats
        };

        print_account_stats(&filtered_stats, args.show_ids, args.account.is_some());
        return Ok(());
    }

    // 处理按分类层级统计
    if args.by_category {
        let (from_dt, to_dt, title) = if let Some(from) = &args.from {
            // 自定义日期范围
            let from_date = parse_date(from)?;
            let to_date = match &args.to {
                Some(to_str) => parse_date(to_str)?,
                None => Local::now().date_naive(),
            };

            if from_date > to_date {
                return Err(crate::error::FinanceError::Validation(
                    "起始日期不能晚于结束日期".to_string(),
                ));
            }

            let from_dt = from_date.and_hms_opt(0, 0, 0).unwrap().and_utc();
            let to_dt = to_date.and_hms_opt(23, 59, 59).unwrap().and_utc();

            let title = if from_date == to_date {
                format!("{} 分类统计", from_date)
            } else {
                format!("{} 至 {} 分类统计", from_date, to_date)
            };
            (Some(from_dt), Some(to_dt), title)
        } else if let Some(month_str) = &args.month {
            // 指定月份
            let (year, month) = parse_month(month_str)?;
            let start = NaiveDate::from_ymd_opt(year, month, 1)
                .ok_or_else(|| crate::error::FinanceError::Validation("无效的日期".to_string()))?;
            let end = if month == 12 {
                NaiveDate::from_ymd_opt(year + 1, 1, 1)
            } else {
                NaiveDate::from_ymd_opt(year, month + 1, 1)
            }
            .ok_or_else(|| crate::error::FinanceError::Validation("无效的日期".to_string()))?;

            let from_dt = start.and_hms_opt(0, 0, 0).unwrap().and_utc();
            let to_dt = end.and_hms_opt(0, 0, 0).unwrap().and_utc();

            let title = format!("{}年{}月 分类统计", year, month);
            (Some(from_dt), Some(to_dt), title)
        } else {
            // 全部时间
            (None, None, "全部分类统计".to_string())
        };

        let stats = db.get_hierarchical_category_stats(from_dt, to_dt).await?;
        print_hierarchical_category_stats(&stats, &title, args.show_ids);
        return Ok(());
    }

    // 判断使用自定义日期范围还是月度统计
    let use_date_range = args.from.is_some();

    if use_date_range {
        // 自定义日期范围统计
        let from_date = parse_date(args.from.as_ref().unwrap())?;
        let to_date = match &args.to {
            Some(to_str) => parse_date(to_str)?,
            None => Local::now().date_naive(),
        };

        // 验证日期范围
        if from_date > to_date {
            return Err(crate::error::FinanceError::Validation(
                "起始日期不能晚于结束日期".to_string(),
            ));
        }

        let from_dt = from_date.and_hms_opt(0, 0, 0).unwrap().and_utc();
        let to_dt = to_date.and_hms_opt(23, 59, 59).unwrap().and_utc();

        let stats = db.get_stats_by_date_range(from_dt, to_dt).await?;

        print_period_stats(&stats, args.by_category, args.show_ids);
    } else {
        // 月度统计
        let now = Local::now();
        let (year, month) = if let Some(month_str) = args.month {
            parse_month(&month_str)?
        } else {
            (now.year(), now.month())
        };

        let stats = db.get_monthly_stats(year, month).await?;

        print_monthly_stats(&stats, args.by_category, args.show_ids);
    }

    Ok(())
}

/// 打印月度统计结果
fn print_monthly_stats(stats: &MonthlyStats, by_category: bool, show_ids: bool) {
    println!("\n[报表] {}年{}月 财务统计\n", stats.year, stats.month);

    // 基本统计
    let mut table = Table::new();
    table.set_header(vec!["项目", "金额"]);
    table.add_row(vec!["总收入", &format!("¥{}", stats.total_income)]);
    table.add_row(vec!["总支出", &format!("¥{}", stats.total_expense)]);

    let net_color = if stats.net >= Decimal::ZERO { "+" } else { "" };
    table.add_row(vec!["净收支", &format!("{}¥{}", net_color, stats.net)]);
    table.add_row(vec!["交易笔数", &stats.transaction_count.to_string()]);
    println!("{}", table);

    // 分类统计
    if by_category {
        if !stats.category_breakdown.is_empty() {
            print_category_breakdown(&stats.category_breakdown, stats.total_expense, show_ids);
        } else {
            println!("\n[信息] 暂无分类数据");
        }
    }
}

/// 打印自定义日期范围统计结果
fn print_period_stats(stats: &PeriodStats, by_category: bool, show_ids: bool) {
    let from_str = stats.from.format("%Y-%m-%d").to_string();
    let to_str = stats.to.format("%Y-%m-%d").to_string();

    if from_str == to_str {
        println!("\n[报表] {} 财务统计\n", from_str);
    } else {
        println!("\n[报表] {} 至 {} 财务统计\n", from_str, to_str);
    }

    // 基本统计
    let mut table = Table::new();
    table.set_header(vec!["项目", "金额"]);
    table.add_row(vec!["总收入", &format!("¥{}", stats.total_income)]);
    table.add_row(vec!["总支出", &format!("¥{}", stats.total_expense)]);

    let net_color = if stats.net >= Decimal::ZERO { "+" } else { "" };
    table.add_row(vec!["净收支", &format!("{}¥{}", net_color, stats.net)]);
    table.add_row(vec!["交易笔数", &stats.transaction_count.to_string()]);
    println!("{}", table);

    // 分类统计
    if by_category && !stats.category_breakdown.is_empty() {
        print_category_breakdown(&stats.category_breakdown, stats.total_expense, show_ids);
    }
}

/// 打印分类统计表格
fn print_category_breakdown(
    category_breakdown: &std::collections::HashMap<String, (String, Decimal)>,
    total_expense: Decimal,
    show_ids: bool,
) {
    println!("\n[图表] 支出分类占比\n");
    let mut cat_table = Table::new();
    cat_table.set_header(vec!["分类", "金额", "占比"]);

    let mut categories: Vec<_> = category_breakdown.iter().collect();
    // 按金额降序排序
    categories.sort_by(|a, b| b.1 .1.cmp(&a.1 .1));

    for (category_id, (category_name, amount)) in categories {
        let percentage = if total_expense > Decimal::ZERO {
            (*amount / total_expense * Decimal::from(100)).round_dp(1)
        } else {
            Decimal::ZERO
        };
        // 根据 show_ids 参数决定显示 ID 还是名称
        let display_name = if show_ids { category_id } else { category_name };
        cat_table.add_row(vec![
            display_name,
            &format!("¥{}", amount),
            &format!("{}%", percentage),
        ]);
    }
    println!("{}", cat_table);
}

/// 打印层级分类统计
fn print_hierarchical_category_stats(
    stats: &[HierarchicalCategoryStats],
    title: &str,
    show_ids: bool,
) {
    println!("\n[报表] {}\n", title);

    // 检查是否有有效数据（非零金额）
    let has_data = stats.iter().any(|s| s.total_amount != Decimal::ZERO);
    if !has_data {
        println!("暂无支出数据");
        return;
    }

    let mut table = Table::new();
    table.set_header(vec!["分类", "直接金额", "汇总金额", "占比"]);

    // 递归添加行
    fn add_rows(
        table: &mut Table,
        stats: &[HierarchicalCategoryStats],
        show_ids: bool,
        indent: usize,
    ) {
        for stat in stats {
            // 跳过金额为0的分类
            if stat.total_amount == Decimal::ZERO && stat.direct_amount == Decimal::ZERO {
                continue;
            }

            // 缩进前缀
            let prefix = "  ".repeat(indent);
            let display_name = if show_ids {
                format!("{}{}", prefix, stat.category_id)
            } else {
                format!("{}{}", prefix, stat.category_name)
            };

            // 直接金额（不为0时显示，负数表示退款）
            let direct_str = if stat.direct_amount != Decimal::ZERO {
                format!("¥{}", stat.direct_amount)
            } else {
                "-".to_string()
            };

            // 汇总金额（与直接金额不同或有子分类时显示）
            let total_str = if stat.total_amount != stat.direct_amount || !stat.children.is_empty()
            {
                format!("¥{}", stat.total_amount)
            } else {
                "-".to_string()
            };

            // 占比显示（正数为支出，负数为退款）
            let percentage_str = format!("{}%", stat.percentage);

            table.add_row(vec![display_name, direct_str, total_str, percentage_str]);

            // 递归添加子分类
            if !stat.children.is_empty() {
                add_rows(table, &stat.children, show_ids, indent + 1);
            }
        }
    }

    add_rows(&mut table, stats, show_ids, 0);
    println!("{}", table);

    // 打印总计（净支出，已扣除退款）
    let total: Decimal = stats.iter().map(|s| s.total_amount).sum();
    println!("\n总支出: ¥{}", total);
}

/// 打印账户统计结果
fn print_account_stats(
    account_stats: &[crate::db::surreal::AccountStats],
    show_ids: bool,
    single_account: bool,
) {
    if account_stats.is_empty() {
        println!("\n[报表] 没有找到账户统计数据\n");
        return;
    }

    // 如果是单账户统计，简化标题
    if single_account && account_stats.len() == 1 {
        let stats = &account_stats[0];
        let display_name = if show_ids {
            &stats.account_id
        } else {
            &stats.account_name
        };
        println!("\n[报表] 账户「{}」统计\n", display_name);
    } else {
        println!("\n[报表] 账户统计\n");
    }

    let mut table = Table::new();
    table.set_header(vec!["账户", "总收入", "总支出", "净流入", "交易笔数"]);

    // 计算总计
    let mut total_income = Decimal::ZERO;
    let mut total_expense = Decimal::ZERO;
    let mut total_count = 0usize;

    for stats in account_stats {
        total_income += stats.total_income;
        total_expense += stats.total_expense;
        total_count += stats.transaction_count;

        let net_color = if stats.net_flow >= Decimal::ZERO {
            "+"
        } else {
            ""
        };
        let display_name = if show_ids {
            &stats.account_id
        } else {
            &stats.account_name
        };

        table.add_row(vec![
            display_name,
            &format!("¥{}", stats.total_income),
            &format!("¥{}", stats.total_expense),
            &format!("{}{}¥{}", net_color, "", stats.net_flow),
            &stats.transaction_count.to_string(),
        ]);
    }

    // 添加总计行（只在多账户时显示）
    if !single_account {
        let net_total = total_income - total_expense;
        let net_color = if net_total >= Decimal::ZERO { "+" } else { "" };
        table.add_row(vec![
            "─────────",
            "─────────",
            "─────────",
            "─────────",
            "─────────",
        ]);
        table.add_row(vec![
            "总计",
            &format!("¥{}", total_income),
            &format!("¥{}", total_expense),
            &format!("{}{}¥{}", net_color, "", net_total),
            &total_count.to_string(),
        ]);
    }

    println!("{}", table);
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

    if !(1..=12).contains(&month) {
        return Err(crate::error::FinanceError::Parse(
            "月份应在 1-12 之间".to_string(),
        ));
    }

    Ok((year, month))
}

/// 解析日期字符串（YYYY-MM-DD）
fn parse_date(date_str: &str) -> Result<NaiveDate> {
    NaiveDate::parse_from_str(date_str, "%Y-%m-%d").map_err(|_| {
        crate::error::FinanceError::Parse(format!("日期格式错误：'{}'，应为 YYYY-MM-DD", date_str))
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_month_valid() {
        assert_eq!(parse_month("2025-04").unwrap(), (2025, 4));
        assert_eq!(parse_month("2024-12").unwrap(), (2024, 12));
        assert_eq!(parse_month("2023-01").unwrap(), (2023, 1));
    }

    #[test]
    fn test_parse_month_invalid_format() {
        assert!(parse_month("2025/04").is_err());
        assert!(parse_month("2025").is_err());
        assert!(parse_month("04-2025").is_err());
        assert!(parse_month("invalid").is_err());
    }

    #[test]
    fn test_parse_month_invalid_month() {
        assert!(parse_month("2025-00").is_err());
        assert!(parse_month("2025-13").is_err());
        assert!(parse_month("2025-99").is_err());
    }

    #[test]
    fn test_parse_month_invalid_year() {
        assert!(parse_month("invalid-04").is_err());
    }

    #[test]
    fn test_parse_date_valid() {
        let date = parse_date("2025-04-15").unwrap();
        assert_eq!(date.year(), 2025);
        assert_eq!(date.month(), 4);
        assert_eq!(date.day(), 15);

        assert_eq!(parse_date("2024-12-31").unwrap().to_string(), "2024-12-31");
        assert_eq!(parse_date("2023-01-01").unwrap().to_string(), "2023-01-01");
    }

    #[test]
    fn test_parse_date_invalid_format() {
        assert!(parse_date("2025/04/15").is_err());
        assert!(parse_date("2025-04").is_err()); // 缺少日
        assert!(parse_date("invalid").is_err());
    }

    #[test]
    fn test_parse_date_invalid_date() {
        assert!(parse_date("2025-02-30").is_err()); // 2月没有30日
        assert!(parse_date("2025-13-01").is_err()); // 没有13月
        assert!(parse_date("2025-00-15").is_err()); // 没有0月
    }
}
