use serde::{Deserialize, Serialize};
use surrealdb::Datetime;

/// 分类（支持任意层级）
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Category {
    /// 永久唯一ID（如 "cat_food_lunch"）
    pub id: String,
    /// 显示名称（当前节点名称）
    pub name: String,
    /// 父分类ID（None表示根分类）
    #[serde(skip_serializing_if = "Option::is_none")]
    pub parent_id: Option<String>,
    /// 预计算完整路径（如 "餐饮/午餐"）
    pub full_path: String,
    /// 层级深度（0=根，1=一级，以此类推）
    pub level: u32,
    /// 创建时间
    pub created_at: Datetime,
}

impl Category {
    /// 创建根分类
    pub fn new_root(id: impl Into<String>, name: impl Into<String>) -> Self {
        let id = id.into();
        let name = name.into();
        let now = Datetime::from(chrono::Utc::now());
        Self {
            id: id.clone(),
            name: name.clone(),
            parent_id: None,
            full_path: name.clone(),
            level: 0,
            created_at: now,
        }
    }

    /// 创建子分类
    pub fn new_child(
        id: impl Into<String>,
        name: impl Into<String>,
        parent_id: impl Into<String>,
        parent_path: &str,
        parent_level: u32,
    ) -> Self {
        let id = id.into();
        let name = name.into();
        let now = Datetime::from(chrono::Utc::now());
        let full_path = format!("{}/{}", parent_path, name);
        Self {
            id: id.clone(),
            name: name.clone(),
            parent_id: Some(parent_id.into()),
            full_path,
            level: parent_level + 1,
            created_at: now,
        }
    }

    /// 从路径解析分类名列表
    pub fn parse_path(path: &str) -> Vec<&str> {
        path.split('/').filter(|s| !s.is_empty()).collect()
    }

    /// 获取路径的最后一部分（当前分类名）
    pub fn path_last_segment(path: &str) -> &str {
        path.rfind('/')
            .map(|i| &path[i + 1..])
            .unwrap_or(path)
    }

    /// 获取父路径
    pub fn parent_path(path: &str) -> Option<&str> {
        path.rfind('/').map(|i| &path[..i])
    }
}

/// 分类树节点（用于递归显示）
#[derive(Debug, Clone)]
pub struct CategoryTreeNode {
    pub category: Category,
    pub children: Vec<CategoryTreeNode>,
}

impl CategoryTreeNode {
    pub fn new(category: Category) -> Self {
        Self {
            category,
            children: Vec::new(),
        }
    }

    /// 递归添加子节点
    pub fn add_child(&mut self, child: CategoryTreeNode) {
        self.children.push(child);
    }

    /// 递归查找节点
    pub fn find_node(&self, id: &str) -> Option<&CategoryTreeNode> {
        if self.category.id == id {
            return Some(self);
        }
        for child in &self.children {
            if let Some(node) = child.find_node(id) {
                return Some(node);
            }
        }
        None
    }
}

/// 打印分类树的辅助函数
pub fn print_category_tree(nodes: &[CategoryTreeNode], prefix: &str) {
    for (i, node) in nodes.iter().enumerate() {
        let is_last = i == nodes.len() - 1;
        let connector = if is_last { "└── " } else { "├── " };
        println!("{}{}{}", prefix, connector, node.category.name);

        let new_prefix = if is_last {
            format!("{}    ", prefix)
        } else {
            format!("{}│   ", prefix)
        };
        print_category_tree(&node.children, &new_prefix);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_category_new_root() {
        let cat = Category::new_root("cat_food", "餐饮");
        assert_eq!(cat.id, "cat_food");
        assert_eq!(cat.name, "餐饮");
        assert_eq!(cat.full_path, "餐饮");
        assert_eq!(cat.level, 0);
        assert!(cat.parent_id.is_none());
    }

    #[test]
    fn test_category_new_child() {
        let child = Category::new_child(
            "cat_lunch",
            "午餐",
            "cat_food",
            "餐饮",
            0,
        );
        assert_eq!(child.id, "cat_lunch");
        assert_eq!(child.name, "午餐");
        assert_eq!(child.full_path, "餐饮/午餐");
        assert_eq!(child.level, 1);
        assert_eq!(child.parent_id, Some("cat_food".to_string()));
    }

    #[test]
    fn test_parse_path() {
        assert_eq!(
            Category::parse_path("餐饮/午餐/食堂"),
            vec!["餐饮", "午餐", "食堂"]
        );
        assert_eq!(Category::parse_path("餐饮"), vec!["餐饮"]);
    }

    #[test]
    fn test_path_last_segment() {
        assert_eq!(Category::path_last_segment("餐饮/午餐"), "午餐");
        assert_eq!(Category::path_last_segment("餐饮"), "餐饮");
    }

    #[test]
    fn test_parent_path() {
        assert_eq!(Category::parent_path("餐饮/午餐"), Some("餐饮"));
        assert_eq!(Category::parent_path("餐饮"), None);
    }
}
