use serde::{Deserialize, Serialize};
use surrealdb::Datetime;

/// 标签
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Tag {
    /// 永久唯一ID（如 "tag_q1"）
    pub id: String,
    /// 显示名称（可修改）
    pub name: String,
    /// 显示颜色（可选，如 "#FF0000"）
    #[serde(skip_serializing_if = "Option::is_none")]
    pub color: Option<String>,
    /// 创建时间
    pub created_at: Datetime,
}

impl Tag {
    /// 创建新标签
    pub fn new(id: impl Into<String>, name: impl Into<String>) -> Self {
        let now = Datetime::from(chrono::Utc::now());
        Self {
            id: id.into(),
            name: name.into(),
            color: None,
            created_at: now,
        }
    }

    /// 设置颜色
    pub fn with_color(mut self, color: impl Into<String>) -> Self {
        self.color = Some(color.into());
        self
    }

    /// 验证颜色格式（简单的十六进制验证）
    pub fn is_valid_color(color: &str) -> bool {
        // 支持 #RGB 或 #RRGGBB 格式
        if !color.starts_with('#') {
            return false;
        }
        let hex = &color[1..];
        matches!(hex.len(), 3 | 6) && hex.chars().all(|c| c.is_ascii_hexdigit())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_tag_new() {
        let tag = Tag::new("tag_q1", "2026-Q1");
        assert_eq!(tag.id, "tag_q1");
        assert_eq!(tag.name, "2026-Q1");
        assert!(tag.color.is_none());
    }

    #[test]
    fn test_tag_with_color() {
        let tag = Tag::new("tag_trip", "旅游").with_color("#00FF00");
        assert_eq!(tag.color, Some("#00FF00".to_string()));
    }

    #[test]
    fn test_is_valid_color() {
        assert!(Tag::is_valid_color("#FF0000")); // RRGGBB
        assert!(Tag::is_valid_color("#F00")); // RGB
        assert!(Tag::is_valid_color("#ff0000")); // 小写
        assert!(!Tag::is_valid_color("FF0000")); // 缺少#
        assert!(!Tag::is_valid_color("#GG0000")); // 非法字符
        assert!(!Tag::is_valid_color("#FF00")); // 长度不对
    }
}
