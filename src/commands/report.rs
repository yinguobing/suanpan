use chrono::{Datelike, Local, NaiveDate, TimeZone, Utc};
use clap::Args;
use rust_decimal::Decimal;
use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;

use crate::db::surreal::Database;
use crate::error::Result;
use crate::models::transaction::Transaction;
use crate::models::types::TxType;
use crate::output::{print_empty_line, print_success, OutputFormat};

/// 生成 HTML 报表
#[derive(Args)]
pub struct ReportArgs {
    /// 报表月份（格式：YYYY-MM，默认为当前月）
    #[arg(short, long)]
    pub month: Option<String>,

    /// 输出目录（默认为当前目录）
    #[arg(short, long, default_value = ".")]
    pub output: PathBuf,
}

/// 分类统计数据
#[derive(Clone)]
struct CategoryStat {
    name: String,
    amount: Decimal,
}

/// 月度统计数据
struct MonthlyStat {
    month: String,
    income: Decimal,
    expense: Decimal,
}

/// 日期统计数据（用于趋势展示）
struct DailyStat {
    date: NaiveDate,
    income: Decimal,
    expense: Decimal,
}

pub async fn execute(db: &Database, args: ReportArgs, output_format: OutputFormat) -> Result<()> {
    let month = match args.month {
        Some(m) => m,
        None => Local::now().format("%Y-%m").to_string(),
    };

    // 解析月份范围
    let (year, mon) = parse_month(&month)?;
    let start_date = NaiveDate::from_ymd_opt(year, mon, 1).unwrap();
    let end_date = if mon == 12 {
        NaiveDate::from_ymd_opt(year + 1, 1, 1).unwrap()
    } else {
        NaiveDate::from_ymd_opt(year, mon + 1, 1).unwrap()
    };

    let start_dt = Utc.from_utc_datetime(&start_date.and_hms_opt(0, 0, 0).unwrap());
    let end_dt = Utc.from_utc_datetime(&end_date.and_hms_opt(0, 0, 0).unwrap());

    match output_format {
        OutputFormat::Machine => println!("GENERATING:{}", month),
        OutputFormat::Human => println!("[报表] 正在生成 {} 月报表...", month),
    }

    // 获取当月交易数据
    let transactions = db.query_by_date_range(start_dt, end_dt).await?;

    if transactions.is_empty() {
        match output_format {
            OutputFormat::Machine => println!("NO_DATA:{}", month),
            OutputFormat::Human => println!("[WARN] {} 月暂无交易记录", month),
        }
        return Ok(());
    }

    match output_format {
        OutputFormat::Machine => println!("FOUND:{}", transactions.len()),
        OutputFormat::Human => println!("   找到 {} 条交易记录", transactions.len()),
    }

    // 确保输出目录存在
    fs::create_dir_all(&args.output)?;

    // 获取分类名称映射
    let categories = db.list_categories().await?;
    let category_map: HashMap<_, _> = categories
        .into_iter()
        .map(|c| (c.id, c.full_path))
        .collect();

    // 获取最近12个月的统计数据
    let monthly_stats = get_monthly_stats(db, year, mon).await?;

    // 生成 HTML 报表
    let html_path = args.output.join(format!("report_{}.html", month));
    generate_html_report(
        &transactions,
        &monthly_stats,
        &category_map,
        &html_path,
        &month,
    )?;

    match output_format {
        OutputFormat::Machine => println!("HTML:{}", html_path.display()),
        OutputFormat::Human => println!("   [OK] HTML 报表: {}", html_path.display()),
    }

    print_empty_line();
    match output_format {
        OutputFormat::Machine => println!("DONE:{}", html_path.display()),
        OutputFormat::Human => {
            print_success("报表生成完成！", output_format);
            println!("   请用浏览器打开: {}", html_path.display());
        }
    }

    Ok(())
}

/// 解析月份字符串
fn parse_month(month: &str) -> Result<(i32, u32)> {
    let parts: Vec<&str> = month.split('-').collect();
    if parts.len() != 2 {
        return Err(crate::error::FinanceError::Validation(format!(
            "无效的月份格式: {}，应为 YYYY-MM",
            month
        )));
    }
    let year = parts[0]
        .parse::<i32>()
        .map_err(|e| crate::error::FinanceError::Validation(format!("无效的年份: {}", e)))?;
    let mon = parts[1]
        .parse::<u32>()
        .map_err(|e| crate::error::FinanceError::Validation(format!("无效的月份: {}", e)))?;
    Ok((year, mon))
}

/// 计算支出分类统计
fn calculate_expense_by_category(
    transactions: &[Transaction],
    category_map: &HashMap<String, String>,
) -> Vec<CategoryStat> {
    let mut stats: HashMap<String, Decimal> = HashMap::new();

    for tx in transactions {
        if tx.tx_type == TxType::Expense {
            let category_name = category_map
                .get(&tx.category_id)
                .cloned()
                .unwrap_or_else(|| "未分类".to_string());
            *stats.entry(category_name).or_insert_with(|| Decimal::ZERO) += tx.amount;
        }
    }

    let mut result: Vec<CategoryStat> = stats
        .into_iter()
        .map(|(name, amount)| CategoryStat { name, amount })
        .collect();

    // 按金额降序排序
    result.sort_by(|a, b| b.amount.cmp(&a.amount));
    result
}

/// 获取最近12个月的统计数据
async fn get_monthly_stats(
    db: &Database,
    current_year: i32,
    current_month: u32,
) -> Result<Vec<MonthlyStat>> {
    let mut stats = Vec::new();

    for i in 0..12 {
        let (year, month) = if current_month <= i {
            (current_year - 1, 12 + current_month - i)
        } else {
            (current_year, current_month - i)
        };

        let start_date = NaiveDate::from_ymd_opt(year, month, 1).unwrap();
        let end_date = if month == 12 {
            NaiveDate::from_ymd_opt(year + 1, 1, 1).unwrap()
        } else {
            NaiveDate::from_ymd_opt(year, month + 1, 1).unwrap()
        };

        let start_dt = Utc.from_utc_datetime(&start_date.and_hms_opt(0, 0, 0).unwrap());
        let end_dt = Utc.from_utc_datetime(&end_date.and_hms_opt(0, 0, 0).unwrap());

        let transactions = db.query_by_date_range(start_dt, end_dt).await?;

        let mut income = Decimal::ZERO;
        let mut expense = Decimal::ZERO;

        for tx in &transactions {
            match tx.tx_type {
                TxType::Income => income += tx.amount,
                TxType::Expense => expense += tx.amount,
                _ => {}
            }
        }

        stats.push(MonthlyStat {
            month: format!("{}-{:02}", year, month),
            income,
            expense,
        });
    }

    // 反转使其按时间正序排列
    stats.reverse();
    Ok(stats)
}

/// 计算每日统计数据
fn calculate_daily_stats(transactions: &[Transaction], year: i32, month: u32) -> Vec<DailyStat> {
    let mut stats: HashMap<NaiveDate, (Decimal, Decimal)> = HashMap::new();

    for tx in transactions {
        // 将 SurrealDB Datetime 转换为 chrono DateTime
        let sql_dt: surrealdb::sql::Datetime = tx.timestamp.to_owned().into_inner();
        let utc_dt: chrono::DateTime<chrono::Utc> = sql_dt.into();
        let local_dt: chrono::DateTime<Local> = utc_dt.into();
        let naive_date = local_dt.date_naive();

        if naive_date.year() == year && naive_date.month() == month {
            let entry = stats
                .entry(naive_date)
                .or_insert((Decimal::ZERO, Decimal::ZERO));
            match tx.tx_type {
                TxType::Income => entry.0 += tx.amount,
                TxType::Expense => entry.1 += tx.amount,
                _ => {}
            }
        }
    }

    let mut result: Vec<DailyStat> = stats
        .into_iter()
        .map(|(date, (income, expense))| DailyStat {
            date,
            income,
            expense,
        })
        .collect();

    result.sort_by(|a, b| a.date.cmp(&b.date));
    result
}

/// 生成 HTML 报表
fn generate_html_report(
    transactions: &[Transaction],
    monthly_stats: &[MonthlyStat],
    category_map: &HashMap<String, String>,
    output_path: &PathBuf,
    month: &str,
) -> Result<()> {
    // 计算当月汇总
    let mut total_income = Decimal::ZERO;
    let mut total_expense = Decimal::ZERO;

    for tx in transactions {
        match tx.tx_type {
            TxType::Income => total_income += tx.amount,
            TxType::Expense => total_expense += tx.amount,
            _ => {}
        }
    }

    let net = total_income - total_expense;

    // 获取支出分类 Top 10
    let expense_stats = calculate_expense_by_category(transactions, category_map);
    let top_expenses: Vec<_> = expense_stats.iter().take(10).collect();

    // 解析年月用于计算日统计
    let parts: Vec<&str> = month.split('-').collect();
    let year = parts[0].parse::<i32>().unwrap_or(2026);
    let mon = parts[1].parse::<u32>().unwrap_or(1);
    let daily_stats = calculate_daily_stats(transactions, year, mon);

    // 生成月度趋势表格
    let trend_table_rows = monthly_stats
        .iter()
        .map(|stat| {
            let net = stat.income - stat.expense;
            format!(
                r#"<tr>
                    <td>{}</td>
                    <td class="amount income">¥{:.2}</td>
                    <td class="amount expense">¥{:.2}</td>
                    <td class="amount {}">¥{:.2}</td>
                </tr>"#,
                stat.month,
                stat.income,
                stat.expense,
                if net >= Decimal::ZERO {
                    "income"
                } else {
                    "expense"
                },
                net
            )
        })
        .collect::<Vec<_>>()
        .join("\n");

    // 生成日收支表格
    let daily_table_rows = daily_stats
        .iter()
        .map(|stat| {
            let net = stat.income - stat.expense;
            format!(
                r#"<tr>
                    <td>{}</td>
                    <td class="amount income">¥{:.2}</td>
                    <td class="amount expense">¥{:.2}</td>
                    <td class="amount {}">¥{:.2}</td>
                </tr>"#,
                stat.date.format("%m-%d"),
                stat.income,
                stat.expense,
                if net >= Decimal::ZERO {
                    "income"
                } else {
                    "expense"
                },
                net
            )
        })
        .collect::<Vec<_>>()
        .join("\n");

    let html = format!(
        r#"<!DOCTYPE html>
<html lang="zh-CN">
<head>
    <meta charset="UTF-8">
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
    <title>财务报表 - {0}</title>
    <style>
        * {{
            margin: 0;
            padding: 0;
            box-sizing: border-box;
        }}
        body {{
            font-family: -apple-system, BlinkMacSystemFont, "Segoe UI", Roboto, "Helvetica Neue", Arial, sans-serif;
            background: #f5f5f5;
            color: #333;
            line-height: 1.6;
        }}
        .container {{
            max-width: 1200px;
            margin: 0 auto;
            padding: 20px;
        }}
        h1 {{
            text-align: center;
            color: #2c3e50;
            margin-bottom: 30px;
            font-size: 28px;
        }}
        .summary {{
            display: grid;
            grid-template-columns: repeat(auto-fit, minmax(200px, 1fr));
            gap: 20px;
            margin-bottom: 30px;
        }}
        .card {{
            background: white;
            border-radius: 12px;
            padding: 24px;
            box-shadow: 0 2px 8px rgba(0,0,0,0.1);
            text-align: center;
        }}
        .card h3 {{
            font-size: 14px;
            color: #666;
            margin-bottom: 8px;
            text-transform: uppercase;
        }}
        .card .value {{
            font-size: 32px;
            font-weight: bold;
            margin-top: 8px;
        }}
        .income {{ color: #27ae60; }}
        .expense {{ color: #e74c3c; }}
        .net {{ color: #2980b9; }}
        .section {{
            background: white;
            border-radius: 12px;
            padding: 24px;
            margin-bottom: 20px;
            box-shadow: 0 2px 8px rgba(0,0,0,0.1);
        }}
        .section h2 {{
            font-size: 20px;
            color: #2c3e50;
            margin-bottom: 20px;
            padding-bottom: 10px;
            border-bottom: 2px solid #3498db;
        }}
        table {{
            width: 100%;
            border-collapse: collapse;
            margin-top: 20px;
        }}
        th, td {{
            padding: 12px;
            text-align: left;
            border-bottom: 1px solid #eee;
        }}
        th {{
            background: #f8f9fa;
            font-weight: 600;
            color: #555;
        }}
        tr:hover {{
            background: #f8f9fa;
        }}
        .amount {{
            text-align: right;
            font-family: "SF Mono", Monaco, monospace;
        }}
        .footer {{
            text-align: center;
            margin-top: 40px;
            padding: 20px;
            color: #999;
            font-size: 14px;
        }}
        .notice {{
            background: #fff3cd;
            border: 1px solid #ffeaa7;
            border-radius: 8px;
            padding: 12px 16px;
            margin-bottom: 20px;
            color: #856404;
        }}
    </style>
</head>
<body>
    <div class="container">
        <h1>📊 {0} 财务月报</h1>
        
        <div class="notice">
            💡 图表功能暂不可用，当前报表以数据表格形式展示。后续版本将引入更优的可视化方案。
        </div>

        <div class="summary">
            <div class="card">
                <h3>总收入</h3>
                <div class="value income">¥{1:.2}</div>
            </div>
            <div class="card">
                <h3>总支出</h3>
                <div class="value expense">¥{2:.2}</div>
            </div>
            <div class="card">
                <h3>净收支</h3>
                <div class="value net">¥{3:.2}</div>
            </div>
            <div class="card">
                <h3>交易笔数</h3>
                <div class="value">{4}</div>
            </div>
        </div>

        <div class="section">
            <h2>📈 收支趋势（最近12个月）</h2>
            <table>
                <thead>
                    <tr>
                        <th>月份</th>
                        <th class="amount">收入</th>
                        <th class="amount">支出</th>
                        <th class="amount">净收支</th>
                    </tr>
                </thead>
                <tbody>
                    {5}
                </tbody>
            </table>
        </div>

        <div class="section">
            <h2>📅 每日收支明细</h2>
            <table>
                <thead>
                    <tr>
                        <th>日期</th>
                        <th class="amount">收入</th>
                        <th class="amount">支出</th>
                        <th class="amount">净收支</th>
                    </tr>
                </thead>
                <tbody>
                    {6}
                </tbody>
            </table>
        </div>

        <div class="section">
            <h2>🏷️ 支出分类 TOP 10</h2>
            <table>
                <thead>
                    <tr>
                        <th>排名</th>
                        <th>分类</th>
                        <th class="amount">金额</th>
                    </tr>
                </thead>
                <tbody>
                    {7}
                </tbody>
            </table>
        </div>

        <div class="footer">
            <p>生成时间: {8} | 算盘 · 个人财务管理</p>
        </div>
    </div>
</body>
</html>"#,
        month,
        total_income,
        total_expense,
        net,
        transactions.len(),
        trend_table_rows,
        daily_table_rows,
        top_expenses
            .iter()
            .enumerate()
            .map(|(i, stat)| format!(
                "<tr><td>{}</td><td>{}</td><td class='amount'>¥{:.2}</td></tr>",
                i + 1,
                stat.name,
                stat.amount
            ))
            .collect::<Vec<_>>()
            .join("\n"),
        Local::now().format("%Y-%m-%d %H:%M:%S")
    );

    fs::write(output_path, html)?;
    Ok(())
}
