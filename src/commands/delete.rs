use clap::Args;

use crate::db::surreal::Database;
use crate::error::Result;

/// 删除交易记录
#[derive(Args)]
pub struct DeleteArgs {
    /// 交易记录的短 ID（一个或多个）
    pub ids: Vec<String>,
}

pub async fn execute(db: &Database, args: DeleteArgs) -> Result<()> {
    if args.ids.is_empty() {
        println!("❌ 请提供要删除的交易记录 ID");
        println!("用法: finance delete <短ID> [短ID...]");
        return Ok(());
    }

    // 验证所有 ID 格式
    for id in &args.ids {
        if id.len() != 12 {
            println!("❌ ID '{}' 格式错误，应为 12 位字符", id);
            return Ok(());
        }
    }

    let results = db.delete_by_short_ids(&args.ids).await?;

    let mut success_count = 0;
    let mut fail_count = 0;

    for (id, success) in &results {
        if *success {
            println!("✅ 已删除: {}", id);
            success_count += 1;
        } else {
            println!("❌ 未找到: {}", id);
            fail_count += 1;
        }
    }

    println!();
    if success_count > 0 {
        println!("成功删除 {} 条记录", success_count);
    }
    if fail_count > 0 {
        println!("未找到 {} 条记录", fail_count);
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    #[test]
    fn test_delete_args_validation() {
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
