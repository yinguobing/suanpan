use chrono::{Local, NaiveDate};
use clap::Args;
use comfy_table::Table;
use rust_decimal::Decimal;

use crate::db::surreal::{Database, TrendPeriod, TrendStats};
use crate::error::Result;

/// 趋势分析
#[derive(Args)]
pub struct TrendArgs {
    /// 周期类型（day: 日, week: 周, month: 月, quarter: 季度, year: 年）
    #[arg(short, long, default_value = "month")]
    pub period: String,

    /// 起始日期（格式：YYYY-MM-DD）
    #[arg(long)]
    pub from: Option<String>,

    /// 结束日期（格式：YYYY-MM-DD，需与 --from 同时使用）
    #[arg(long, requires = "from")]
    pub to: Option<String>,

    /// 按分类展示趋势
    #[arg(long)]
    pub by_category: bool,

    /// 显示分类/账户 ID 而非名称
    #[arg(long)]
    pub show_ids: bool,
}

pub async fn execute(db: &Database, args: TrendArgs) -> Result<()> {
    // 解析周期类型
    let period = parse_period(&args.period)?;

    // 解析日期范围
    let (from_date, to_date) = if let Some(from_str) = &args.from {
        let from = parse_date(from_str)?;
        let to = match &args.to {
            Some(to_str) => parse_date(to_str)?,
            None => Local::now().date_naive(),
        };

        if from > to {
            return Err(crate::error::FinanceError::Validation(
                "起始日期不能晚于结束日期".to_string(),
            ));
        }

        (from, to)
    } else {
        // 默认最近6个月
        let today = Local::now().date_naive();
        let from = today
            .checked_sub_months(chrono::Months::new(6))
            .unwrap_or(today);
        (from, today)
    };

    let from_dt = from_date.and_hms_opt(0, 0, 0).unwrap().and_utc();
    let to_dt = to_date.and_hms_opt(23, 59, 59).unwrap().and_utc();

    // 获取趋势数据
    let stats = db.get_trend_stats(period, from_dt, to_dt).await?;

    // 打印趋势表
    print_trend_stats(&stats, &period, from_date, to_date, args.show_ids);

    // 如果按分类展示
    if args.by_category {
        let category_trends = db.get_category_trend_stats(period, from_dt, to_dt).await?;
        print_category_trends(&category_trends, &period, args.show_ids);
    }

    Ok(())
}

/// 打印趋势统计
fn print_trend_stats(
    stats: &[TrendStats],
    period: &TrendPeriod,
    from_date: NaiveDate,
    to_date: NaiveDate,
    _show_ids: bool,
) {
    let period_name = match period {
        TrendPeriod::Day => "日",
        TrendPeriod::Week => "周",
        TrendPeriod::Month => "月",
        TrendPeriod::Quarter => "季度",
        TrendPeriod::Year => "年",
    };

    println!(
        "\n[趋势] {}趋势分析 ({} 至 {})\n",
        period_name, from_date, to_date
    );

    if stats.is_empty() {
        println!("暂无数据");
        return;
    }

    let mut table = Table::new();
    table.set_header(vec!["时间周期", "收入", "支出", "净收支", "笔数"]);

    let mut total_income = Decimal::ZERO;
    let mut total_expense = Decimal::ZERO;
    let mut total_count = 0usize;

    for stat in stats {
        total_income += stat.income;
        total_expense += stat.expense;
        total_count += stat.transaction_count;

        let net = stat.income - stat.expense;
        let net_str = format_net(net);

        table.add_row(vec![
            &stat.period_label,
            &format!("¥{}", stat.income),
            &format!("¥{}", stat.expense),
            &net_str,
            &stat.transaction_count.to_string(),
        ]);
    }

    // 添加总计行
    let total_net = total_income - total_expense;
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
        &format_net(total_net),
        &total_count.to_string(),
    ]);

    println!("{}", table);

    // 计算平均值
    let avg_income = if !stats.is_empty() {
        total_income / Decimal::from(stats.len())
    } else {
        Decimal::ZERO
    };
    let avg_expense = if !stats.is_empty() {
        total_expense / Decimal::from(stats.len())
    } else {
        Decimal::ZERO
    };

    println!("\n平均值: 收入 ¥{} | 支出 ¥{}", avg_income, avg_expense);
}

/// 打印分类趋势
fn print_category_trends(
    trends: &[(String, Vec<(String, Decimal)>)],
    period: &TrendPeriod,
    show_ids: bool,
) {
    let period_name = match period {
        TrendPeriod::Day => "日",
        TrendPeriod::Week => "周",
        TrendPeriod::Month => "月",
        TrendPeriod::Quarter => "季度",
        TrendPeriod::Year => "年",
    };

    println!("\n[图表] 分类{}度支出趋势\n", period_name);

    if trends.is_empty() {
        println!("暂无分类数据");
        return;
    }

    // 获取所有时间周期（以第一个分类的时间序列为准）
    let periods: Vec<String> = trends
        .first()
        .map(|(_, data)| data.iter().map(|(p, _)| p.clone()).collect())
        .unwrap_or_default();

    if periods.is_empty() {
        println!("暂无数据");
        return;
    }

    // 创建表格
    let mut table = Table::new();
    let mut headers = vec!["分类".to_string()];
    headers.extend(periods.iter().cloned());
    table.set_header(headers);

    // 为每个分类添加一行
    for (category_id, data) in trends {
        let display_name = if show_ids {
            category_id.clone()
        } else {
            // 简化显示：只显示最后一部分
            category_id
                .split('/')
                .last()
                .unwrap_or(category_id)
                .to_string()
        };

        let mut row = vec![display_name];
        for (_, amount) in data {
            if *amount > Decimal::ZERO {
                row.push(format!("¥{}", amount));
            } else {
                row.push("-".to_string());
            }
        }
        table.add_row(row);
    }

    println!("{}", table);
}

/// 格式化净收支（带符号）
fn format_net(net: Decimal) -> String {
    if net >= Decimal::ZERO {
        format!("+¥{}", net)
    } else {
        format!("-¥{}", net.abs())
    }
}

/// 解析周期类型
fn parse_period(period: &str) -> Result<TrendPeriod> {
    match period.to_lowercase().as_str() {
        "day" | "d" | "日" => Ok(TrendPeriod::Day),
        "week" | "w" | "周" => Ok(TrendPeriod::Week),
        "month" | "m" | "月" => Ok(TrendPeriod::Month),
        "quarter" | "q" | "季度" => Ok(TrendPeriod::Quarter),
        "year" | "y" | "年" => Ok(TrendPeriod::Year),
        _ => Err(crate::error::FinanceError::Validation(format!(
            "无效的周期类型: {}，支持 day/week/month/quarter/year",
            period
        ))),
    }
}

/// 解析日期字符串（YYYY-MM-DD）
fn parse_date(date_str: &str) -> Result<NaiveDate> {
    NaiveDate::parse_from_str(date_str, "%Y-%m-%d").map_err(|_| {
        crate::error::FinanceError::Parse(format!("日期格式错误：'{}'，应为 YYYY-MM-DD", date_str))
    })
}
