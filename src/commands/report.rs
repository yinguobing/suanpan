use chrono::{Datelike, Local, NaiveDate, TimeZone, Utc};
use clap::Args;
use plotters::prelude::*;
use rust_decimal::Decimal;
use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;

use crate::db::surreal::Database;
use crate::error::Result;
use crate::models::transaction::Transaction;
use crate::models::types::TxType;

/// 获取中文字体路径
fn get_chinese_font() -> Option<&'static str> {
    // 尝试常见的中文字体路径
    let font_paths = [
        "/usr/share/fonts/opentype/noto/NotoSansCJK-Regular.ttc",
        "/usr/share/fonts/truetype/noto/NotoSansCJK-Regular.ttc",
        "/usr/share/fonts/noto-cjk/NotoSansCJK-Regular.ttc",
    ];
    
    for path in &font_paths {
        if std::path::Path::new(path).exists() {
            return Some(path);
        }
    }
    None
}

/// 创建支持中文的字体描述
fn chinese_font(size: f64) -> FontDesc<'static> {
    if let Some(_font_path) = get_chinese_font() {
        FontDesc::new(FontFamily::Name("Noto Sans CJK SC"), size, FontStyle::Normal)
            .to_owned()
    } else {
        ("sans-serif", size).into_font()
    }
}

/// 生成可视化报表
#[derive(Args)]
pub struct ReportArgs {
    /// 报表月份（格式：YYYY-MM，默认为当前月）
    #[arg(short, long)]
    pub month: Option<String>,

    /// 输出目录（默认为当前目录）
    #[arg(short, long, default_value = ".")]
    pub output: PathBuf,

    /// 只生成图表，不生成 HTML
    #[arg(long)]
    pub charts_only: bool,

    /// 图表宽度（像素）
    #[arg(long, default_value = "800")]
    pub width: u32,

    /// 图表高度（像素）
    #[arg(long, default_value = "600")]
    pub height: u32,
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

/// 日期统计数据（用于趋势图）
struct DailyStat {
    date: NaiveDate,
    income: Decimal,
    expense: Decimal,
}

pub async fn execute(db: &Database, args: ReportArgs) -> Result<()> {
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

    println!("[报表] 正在生成 {} 月报表...", month);

    // 获取当月交易数据
    let transactions = db.query_by_date_range(start_dt, end_dt).await?;
    
    if transactions.is_empty() {
        println!("[WARN] {} 月暂无交易记录", month);
        return Ok(());
    }

    println!("   找到 {} 条交易记录", transactions.len());

    // 确保输出目录存在
    fs::create_dir_all(&args.output)?;

    // 获取分类名称映射
    let categories = db.list_categories().await?;
    let category_map: HashMap<_, _> = categories
        .into_iter()
        .map(|c| (c.id, c.full_path))
        .collect();

    // 1. 生成支出分类饼图
    let pie_chart_path = args.output.join(format!("expense_pie_{}.png", month));
    generate_expense_pie_chart(&transactions, &category_map, &pie_chart_path, args.width, args.height)?;
    println!("   ✅ 支出分类饼图: {}", pie_chart_path.display());

    // 2. 生成月度收支趋势图（最近12个月）
    let trend_chart_path = args.output.join(format!("trend_{}.png", month));
    let monthly_stats = get_monthly_stats(db, year, mon).await?;
    generate_trend_chart(&monthly_stats, &trend_chart_path, args.width, args.height)?;
    println!("   ✅ 收支趋势图: {}", trend_chart_path.display());

    // 3. 生成当月日收支趋势图
    let daily_chart_path = args.output.join(format!("daily_{}.png", month));
    let daily_stats = calculate_daily_stats(&transactions, year, mon);
    generate_daily_chart(&daily_stats, &daily_chart_path, args.width, args.height)?;
    println!("   ✅ 日收支趋势图: {}", daily_chart_path.display());

    // 4. 生成 HTML 报表（除非指定 --charts-only）
    if !args.charts_only {
        let html_path = args.output.join(format!("report_{}.html", month));
        generate_html_report(&transactions, &monthly_stats, &category_map, &html_path, &month, &args.output)?;
        println!("   [OK] HTML 报表: {}", html_path.display());
    }

    println!("\n[完成] 报表生成完成！");
    if !args.charts_only {
        println!("   请用浏览器打开: {}", args.output.join(format!("report_{}.html", month)).display());
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
    let year = parts[0].parse::<i32>().map_err(|e| {
        crate::error::FinanceError::Validation(format!("无效的年份: {}", e))
    })?;
    let mon = parts[1].parse::<u32>().map_err(|e| {
        crate::error::FinanceError::Validation(format!("无效的月份: {}", e))
    })?;
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

/// 将 plotters 错误转换为 FinanceError
fn plot_err<E: std::fmt::Display>(e: E) -> crate::error::FinanceError {
    crate::error::FinanceError::Io(std::io::Error::new(
        std::io::ErrorKind::Other,
        format!("图表生成错误: {}", e)
    ))
}

/// 生成支出分类饼图
fn generate_expense_pie_chart(
    transactions: &[Transaction],
    category_map: &HashMap<String, String>,
    output_path: &PathBuf,
    width: u32,
    height: u32,
) -> Result<()> {
    let stats = calculate_expense_by_category(transactions, category_map);
    
    if stats.is_empty() {
        return Ok(());
    }

    let root = BitMapBackend::new(output_path, (width, height)).into_drawing_area();
    root.fill(&WHITE).map_err(plot_err)?;

    let total: Decimal = stats.iter().map(|s| s.amount).sum();
    let total_f64: f64 = total.try_into().unwrap_or(0.0);

    // 只显示前8个分类，其余归为"其他"
    let (main_stats, other_amount) = if stats.len() > 8 {
        let main: Vec<_> = stats[..8].iter().cloned().collect();
        let other: Decimal = stats[8..].iter().map(|s| s.amount).sum();
        (main, other)
    } else {
        (stats, Decimal::ZERO)
    };

    let mut data: Vec<(String, f64)> = main_stats
        .iter()
        .map(|s| {
            let amount_f64: f64 = s.amount.try_into().unwrap_or(0.0);
            (s.name.clone(), amount_f64)
        })
        .collect();

    if other_amount > Decimal::ZERO {
        let other_f64: f64 = other_amount.try_into().unwrap_or(0.0);
        data.push(("其他".to_string(), other_f64));
    }

    // 颜色方案
    let colors: Vec<RGBColor> = vec![
        RGBColor(255, 99, 132),
        RGBColor(54, 162, 235),
        RGBColor(255, 206, 86),
        RGBColor(75, 192, 192),
        RGBColor(153, 102, 255),
        RGBColor(255, 159, 64),
        RGBColor(199, 199, 199),
        RGBColor(83, 102, 255),
        RGBColor(128, 128, 128),
    ];

    // 计算饼图参数
    let center = (width as i32 / 2, height as i32 / 2 + 20);
    let radius = (width.min(height) as i32 / 3) as i32;

    let mut current_angle: f64 = 0.0;

    for (i, (label, value)) in data.iter().enumerate() {
        let percentage = *value / total_f64;
        let sweep_angle = percentage * 2.0 * std::f64::consts::PI;

        let color = colors[i % colors.len()];
        
        // 绘制扇形 - 使用多边形近似圆弧
        let mut points: Vec<(i32, i32)> = vec![center];
        let steps = 20;
        for step in 0..=steps {
            let angle = current_angle + sweep_angle * (step as f64 / steps as f64);
            let x = center.0 + (angle.cos() * radius as f64) as i32;
            let y = center.1 + (angle.sin() * radius as f64) as i32;
            points.push((x, y));
        }
        points.push(center);
        
        root.draw(&Polygon::new(points, color.filled())).map_err(plot_err)?;

        // 绘制标签线
        let mid_angle = current_angle + sweep_angle / 2.0;
        let label_radius = radius + 30;
        let label_x = center.0 + (mid_angle.cos() * label_radius as f64) as i32;
        let label_y = center.1 + (mid_angle.sin() * label_radius as f64) as i32;

        let percentage_text = format!("{:.1}%", percentage * 100.0);
        let label_font = chinese_font(12.0);
        root.draw(&Text::new(
            format!("{} {}", label, percentage_text),
            (label_x, label_y),
            label_font.color(&BLACK),
        )).map_err(plot_err)?;

        current_angle += sweep_angle;
    }

    // 标题
    let title_font = chinese_font(16.0);
    root.draw(&Text::new(
        format!("支出分类分布 (总计: ¥{:.2})", total_f64),
        (width as i32 / 2 - 100, 30),
        title_font.color(&BLACK),
    )).map_err(plot_err)?;

    root.present().map_err(plot_err)?;
    Ok(())
}

/// 获取最近12个月的统计数据
async fn get_monthly_stats(db: &Database, current_year: i32, current_month: u32) -> Result<Vec<MonthlyStat>> {
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

/// 生成月度收支趋势图
fn generate_trend_chart(
    stats: &[MonthlyStat],
    output_path: &PathBuf,
    width: u32,
    height: u32,
) -> Result<()> {
    let root = BitMapBackend::new(output_path, (width, height)).into_drawing_area();
    root.fill(&WHITE).map_err(plot_err)?;

    let max_value: f64 = stats
        .iter()
        .map(|s| {
            let income: f64 = s.income.try_into().unwrap_or(0.0);
            let expense: f64 = s.expense.try_into().unwrap_or(0.0);
            income.max(expense)
        })
        .fold(0.0, f64::max)
        * 1.1;

    let caption_font = chinese_font(20.0);
    let mut chart = ChartBuilder::on(&root)
        .caption("收支趋势（最近12个月）", caption_font)
        .margin(40)
        .x_label_area_size(40)
        .y_label_area_size(60)
        .build_cartesian_2d(0..stats.len() - 1, 0.0..max_value)
        .map_err(plot_err)?;

    chart.configure_mesh()
        .x_labels(stats.len())
        .x_label_formatter(&|x| {
            if *x < stats.len() {
                stats[*x].month.clone()
            } else {
                String::new()
            }
        })
        .y_desc("金额")
        .draw()
        .map_err(plot_err)?;

    // 收入折线（绿色）
    let income_points: Vec<(usize, f64)> = stats
        .iter()
        .enumerate()
        .map(|(i, s)| {
            let val: f64 = s.income.try_into().unwrap_or(0.0);
            (i, val)
        })
        .collect();

    chart.draw_series(LineSeries::new(income_points, &GREEN))
        .map_err(plot_err)?
        .label("收入")
        .legend(|(x, y)| PathElement::new(vec![(x, y), (x + 10, y)], GREEN));

    // 支出折线（红色）
    let expense_points: Vec<(usize, f64)> = stats
        .iter()
        .enumerate()
        .map(|(i, s)| {
            let val: f64 = s.expense.try_into().unwrap_or(0.0);
            (i, val)
        })
        .collect();

    chart.draw_series(LineSeries::new(expense_points, &RED))
        .map_err(plot_err)?
        .label("支出")
        .legend(|(x, y)| PathElement::new(vec![(x, y), (x + 10, y)], RED));

    chart.configure_series_labels()
        .background_style(WHITE.mix(0.8))
        .border_style(BLACK)
        .draw()
        .map_err(plot_err)?;

    root.present().map_err(plot_err)?;
    Ok(())
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
            let entry = stats.entry(naive_date).or_insert((Decimal::ZERO, Decimal::ZERO));
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

/// 生成每日收支柱状图
fn generate_daily_chart(
    stats: &[DailyStat],
    output_path: &PathBuf,
    width: u32,
    height: u32,
) -> Result<()> {
    if stats.is_empty() {
        return Ok(());
    }

    let root = BitMapBackend::new(output_path, (width, height)).into_drawing_area();
    root.fill(&WHITE).map_err(plot_err)?;

    let max_value: f64 = stats
        .iter()
        .map(|s| {
            let income: f64 = s.income.try_into().unwrap_or(0.0);
            let expense: f64 = s.expense.try_into().unwrap_or(0.0);
            income.max(expense)
        })
        .fold(0.0, f64::max)
        * 1.1;

    let days: Vec<String> = stats
        .iter()
        .map(|s| s.date.format("%m-%d").to_string())
        .collect();

    // 使用 f64 类型的 X 轴
    let x_range = if stats.len() > 1 {
        0.0..(stats.len() - 1) as f64
    } else {
        0.0..1.0
    };

    let caption_font = chinese_font(20.0);
    let mut chart = ChartBuilder::on(&root)
        .caption("每日收支情况", caption_font)
        .margin(40)
        .x_label_area_size(40)
        .y_label_area_size(60)
        .build_cartesian_2d(x_range, 0.0..max_value)
        .map_err(plot_err)?;

    // 计算标签位置
    let label_step = if stats.len() <= 10 { 1 } else { stats.len() / 10 };

    chart.configure_mesh()
        .x_labels(stats.len().min(10))
        .x_label_formatter(&|x| {
            let idx = *x as usize;
            if idx < days.len() && idx % label_step == 0 {
                days[idx].clone()
            } else {
                String::new()
            }
        })
        .y_desc("金额")
        .draw()
        .map_err(plot_err)?;

    // 绘制柱状图
    let bar_width = 0.35;

    for (i, stat) in stats.iter().enumerate() {
        let income: f64 = stat.income.try_into().unwrap_or(0.0);
        let expense: f64 = stat.expense.try_into().unwrap_or(0.0);
        let x = i as f64;

        // 收入柱（绿色）
        if income > 0.0 {
            let x_start = x - bar_width;
            let x_end = x;
            chart.draw_series(std::iter::once(
                Rectangle::new([(x_start, 0.0), (x_end, income)], GREEN.filled())
            )).map_err(plot_err)?;
        }

        // 支出柱（红色）
        if expense > 0.0 {
            let x_start = x;
            let x_end = x + bar_width;
            chart.draw_series(std::iter::once(
                Rectangle::new([(x_start, 0.0), (x_end, expense)], RED.filled())
            )).map_err(plot_err)?;
        }
    }

    // 图例 - 使用 i32 坐标
    let legend_y: i32 = 20;
    root.draw(&Rectangle::new(
        [(width as i32 - 150, legend_y), (width as i32 - 130, legend_y + 15)],
        GREEN.filled(),
    )).map_err(plot_err)?;
    let legend_font = chinese_font(14.0);
    root.draw(&Text::new("收入", (width as i32 - 125, legend_y + 2), legend_font.clone()))
        .map_err(plot_err)?;

    root.draw(&Rectangle::new(
        [(width as i32 - 70, legend_y), (width as i32 - 50, legend_y + 15)],
        RED.filled(),
    )).map_err(plot_err)?;
    root.draw(&Text::new("支出", (width as i32 - 45, legend_y + 2), legend_font))
        .map_err(plot_err)?;

    root.present().map_err(plot_err)?;
    Ok(())
}

/// 生成 HTML 报表
fn generate_html_report(
    transactions: &[Transaction],
    _monthly_stats: &[MonthlyStat],
    category_map: &HashMap<String, String>,
    output_path: &PathBuf,
    month: &str,
    _output_dir: &PathBuf,
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

    // 获取图表文件名
    let pie_chart = format!("expense_pie_{}.png", month);
    let trend_chart = format!("trend_{}.png", month);
    let daily_chart = format!("daily_{}.png", month);

    let html = format!(
        r#"<!DOCTYPE html>
<html lang="zh-CN">
<head>
    <meta charset="UTF-8">
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
    <title>财务报表 - {}</title>
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
        .chart-section {{
            background: white;
            border-radius: 12px;
            padding: 24px;
            margin-bottom: 20px;
            box-shadow: 0 2px 8px rgba(0,0,0,0.1);
        }}
        .chart-section h2 {{
            font-size: 20px;
            color: #2c3e50;
            margin-bottom: 20px;
            padding-bottom: 10px;
            border-bottom: 2px solid #3498db;
        }}
        .chart-row {{
            display: grid;
            grid-template-columns: repeat(auto-fit, minmax(400px, 1fr));
            gap: 20px;
            margin-bottom: 20px;
        }}
        .chart-img {{
            max-width: 100%;
            height: auto;
            border-radius: 8px;
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
    </style>
</head>
<body>
    <div class="container">
        <h1>[报表] {} 财务月报</h1>
        
        <div class="summary">
            <div class="card">
                <h3>总收入</h3>
                <div class="value income">¥{:.2}</div>
            </div>
            <div class="card">
                <h3>总支出</h3>
                <div class="value expense">¥{:.2}</div>
            </div>
            <div class="card">
                <h3>净收支</h3>
                <div class="value net">¥{:.2}</div>
            </div>
            <div class="card">
                <h3>交易笔数</h3>
                <div class="value">{}</div>
            </div>
        </div>

        <div class="chart-row">
            <div class="chart-section">
                <h2>[图表] 支出分类分布</h2>
                <img src="{}" alt="支出分类饼图" class="chart-img">
            </div>
            <div class="chart-section">
                <h2>[图表] 收支趋势（12个月）</h2>
                <img src="{}" alt="收支趋势图" class="chart-img">
            </div>
        </div>

        <div class="chart-section">
            <h2>[图表] 每日收支情况</h2>
            <img src="{}" alt="每日收支图" class="chart-img">
        </div>

        <div class="chart-section">
            <h2>[排名] 支出分类 TOP 10</h2>
            <table>
                <thead>
                    <tr>
                        <th>排名</th>
                        <th>分类</th>
                        <th class="amount">金额</th>
                    </tr>
                </thead>
                <tbody>
                    {}
                </tbody>
            </table>
        </div>

        <div class="footer">
            <p>生成时间: {} | 财务管理系统</p>
        </div>
    </div>
</body>
</html>"#,
        month,
        month,
        total_income,
        total_expense,
        net,
        transactions.len(),
        pie_chart,
        trend_chart,
        daily_chart,
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
