//! 输出格式化模块
//!
//! 提供两种输出格式：
//! - Machine: 机器可读格式，简洁无表格线，适合LLM解析
//! - Human: 人类友好格式，带表格线和对齐，适合人类阅读

use comfy_table::{CellAlignment, Table};

/// 输出格式类型
#[derive(Clone, Copy, Debug, Default)]
pub enum OutputFormat {
    /// 机器可读格式（默认，适合LLM）
    #[default]
    Machine,
    /// 人类友好格式（带表格线）
    Human,
}

impl From<bool> for OutputFormat {
    fn from(human_readable: bool) -> Self {
        if human_readable {
            OutputFormat::Human
        } else {
            OutputFormat::Machine
        }
    }
}

/// 表格构建器
pub struct OutputTable {
    format: OutputFormat,
    headers: Vec<String>,
    rows: Vec<Vec<String>>,
    align_right_columns: Vec<usize>,
}

impl OutputTable {
    /// 创建新的表格
    pub fn new(format: OutputFormat) -> Self {
        Self {
            format,
            headers: Vec::new(),
            rows: Vec::new(),
            align_right_columns: Vec::new(),
        }
    }

    /// 设置表头
    pub fn set_header(&mut self, headers: Vec<&str>) {
        self.headers = headers.into_iter().map(|s| s.to_string()).collect();
    }

    /// 添加一行数据
    pub fn add_row(&mut self, row: Vec<&str>) {
        self.rows
            .push(row.into_iter().map(|s| s.to_string()).collect());
    }

    /// 设置右对齐的列（按索引）
    pub fn set_align_right(&mut self, columns: Vec<usize>) {
        self.align_right_columns = columns;
    }

    /// 打印表格
    pub fn print(&self) {
        match self.format {
            OutputFormat::Machine => self.print_machine(),
            OutputFormat::Human => self.print_human(),
        }
    }

    /// 转换为字符串
    pub fn as_string(&self) -> String {
        match self.format {
            OutputFormat::Machine => self.to_string_machine(),
            OutputFormat::Human => self.to_string_human(),
        }
    }

    /// 机器可读格式输出
    fn print_machine(&self) {
        if self.headers.is_empty() && self.rows.is_empty() {
            return;
        }

        // 打印表头
        if !self.headers.is_empty() {
            println!("{}", self.headers.join("|"));
        }

        // 打印数据行
        for row in &self.rows {
            println!("{}", row.join("|"));
        }
    }

    /// 人类可读格式输出（使用 comfy_table）
    fn print_human(&self) {
        if self.headers.is_empty() && self.rows.is_empty() {
            return;
        }

        let mut table = Table::new();

        if !self.headers.is_empty() {
            table.set_header(self.headers.clone());
        }

        for row in &self.rows {
            table.add_row(row.clone());
        }

        // 设置右对齐列
        for &col_idx in &self.align_right_columns {
            if let Some(col) = table.column_mut(col_idx) {
                col.set_cell_alignment(CellAlignment::Right);
            }
        }

        println!("{}", table);
    }

    /// 机器可读格式字符串
    fn to_string_machine(&self) -> String {
        if self.headers.is_empty() && self.rows.is_empty() {
            return String::new();
        }

        let mut result = String::new();

        // 表头
        if !self.headers.is_empty() {
            result.push_str(&self.headers.join("|"));
            result.push('\n');
        }

        // 数据行
        for row in &self.rows {
            result.push_str(&row.join("|"));
            result.push('\n');
        }

        result
    }

    /// 人类可读格式字符串
    fn to_string_human(&self) -> String {
        if self.headers.is_empty() && self.rows.is_empty() {
            return String::new();
        }

        let mut table = Table::new();

        if !self.headers.is_empty() {
            table.set_header(self.headers.clone());
        }

        for row in &self.rows {
            table.add_row(row.clone());
        }

        // 设置右对齐列
        for &col_idx in &self.align_right_columns {
            if let Some(col) = table.column_mut(col_idx) {
                col.set_cell_alignment(CellAlignment::Right);
            }
        }

        table.to_string()
    }
}

/// 打印标题
pub fn print_title(title: &str, format: OutputFormat) {
    match format {
        OutputFormat::Machine => println!("{}", title),
        OutputFormat::Human => println!("\n[{}]\n", title),
    }
}

/// 打印键值对
pub fn print_kv(key: &str, value: &str, format: OutputFormat) {
    match format {
        OutputFormat::Machine => println!("{}: {}", key, value),
        OutputFormat::Human => println!("{}: {}", key, value),
    }
}

/// 打印统计信息（键值对形式）
pub fn print_stats(stats: &[(&str, String)], format: OutputFormat) {
    match format {
        OutputFormat::Machine => {
            for (key, value) in stats {
                println!("{}: {}", key, value);
            }
        }
        OutputFormat::Human => {
            let mut table = Table::new();
            table.set_header(vec!["项目", "数值"]);
            for (key, value) in stats {
                table.add_row(vec![*key, value]);
            }
            println!("{}", table);
        }
    }
}

/// 打印信息消息
pub fn print_info(message: &str, format: OutputFormat) {
    match format {
        OutputFormat::Machine => println!("[INFO] {}", message),
        OutputFormat::Human => println!("[信息] {}", message),
    }
}

/// 打印成功消息
pub fn print_success(message: &str, format: OutputFormat) {
    match format {
        OutputFormat::Machine => println!("[OK] {}", message),
        OutputFormat::Human => println!("[OK] {}", message),
    }
}

/// 打印错误消息
pub fn print_error(message: &str, format: OutputFormat) {
    match format {
        OutputFormat::Machine => println!("[ERR] {}", message),
        OutputFormat::Human => println!("[ERR] {}", message),
    }
}

/// 打印警告消息
pub fn print_warning(message: &str, format: OutputFormat) {
    match format {
        OutputFormat::Machine => println!("[WARN] {}", message),
        OutputFormat::Human => println!("[WARN] {}", message),
    }
}

/// 打印空行
pub fn print_empty_line() {
    println!();
}

/// 打印分隔线（仅人类可读模式）
pub fn print_separator(format: OutputFormat) {
    if matches!(format, OutputFormat::Human) {
        println!("{}", "-".repeat(60));
    }
}

/// 打印记录数统计
pub fn print_count(count: usize, total: Option<usize>, format: OutputFormat) {
    match format {
        OutputFormat::Machine => {
            if let Some(t) = total {
                println!("COUNT: {}/{}", count, t);
            } else {
                println!("COUNT: {}", count);
            }
        }
        OutputFormat::Human => {
            if let Some(t) = total {
                println!("共 {} 条记录 (总计 {})", count, t);
            } else {
                println!("共 {} 条记录", count);
            }
        }
    }
}
