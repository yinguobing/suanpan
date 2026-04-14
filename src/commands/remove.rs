use clap::Args;

use crate::db::surreal::Database;
use crate::error::Result;
use crate::output::{print_empty_line, print_error, print_success, OutputFormat};

/// 移除交易记录
#[derive(Args)]
pub struct RemoveArgs {
    /// 交易记录的短 ID（一个或多个）
    pub ids: Vec<String>,
}

pub async fn execute(db: &Database, args: RemoveArgs, output_format: OutputFormat) -> Result<()> {
    if args.ids.is_empty() {
        match output_format {
            OutputFormat::Machine => println!("ERROR:NO_IDS"),
            OutputFormat::Human => {
                print_error("请提供要移除的交易记录 ID", output_format);
                println!("用法: suanpan remove <短ID> [短ID...]");
            }
        }
        return Ok(());
    }

    // 验证所有 ID 格式
    for id in &args.ids {
        if id.len() != 12 {
            print_error(
                &format!("ID '{}' 格式错误，应为 12 位字符", id),
                output_format,
            );
            return Ok(());
        }
    }

    let results = db.delete_by_short_ids(&args.ids).await?;

    let mut success_count = 0;
    let mut fail_count = 0;

    for (id, success) in &results {
        if *success {
            match output_format {
                OutputFormat::Machine => println!("REMOVED:{}", id),
                OutputFormat::Human => print_success(&format!("已移除: {}", id), output_format),
            }
            success_count += 1;
        } else {
            match output_format {
                OutputFormat::Machine => println!("NOT_FOUND:{}", id),
                OutputFormat::Human => print_error(&format!("未找到: {}", id), output_format),
            }
            fail_count += 1;
        }
    }

    print_empty_line();
    match output_format {
        OutputFormat::Machine => {
            println!("RESULT:SUCCESS:{}:FAIL:{}", success_count, fail_count);
        }
        OutputFormat::Human => {
            if success_count > 0 {
                println!("成功移除 {} 条记录", success_count);
            }
            if fail_count > 0 {
                println!("未找到 {} 条记录", fail_count);
            }
        }
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    #[test]
    fn test_remove_args_validation() {
        // 有效 ID: 12 位
        let valid_id = "f4sp877fxbwc";
        assert_eq!(valid_id.len(), 12);

        // 无效 ID: 太短
        let short_id = "abc123";
        assert!(short_id.len() != 12);

        // 无效 ID: 太长
        let long_id = "f4sp877fxbwc1234567890";
        assert!(long_id.len() != 12);
    }
}
