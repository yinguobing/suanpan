use serde::{Deserialize, Serialize};
use surrealdb::Datetime;
use tree_ds::prelude::{Node, Tree, TraversalStrategy};

/// 分类节点数据（存储在 tree-ds Node 中的值）
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct CategoryData {
    /// 显示名称
    pub name: String,
    /// 创建时间
    pub created_at: Datetime,
}

impl CategoryData {
    /// 创建新的分类数据
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            created_at: Datetime::from(chrono::Utc::now()),
        }
    }
}

/// 分类树（使用 tree-ds 实现）
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CategoryTree {
    tree: Tree<String, CategoryData>,
}

impl CategoryTree {
    /// 创建空树
    pub fn new() -> Self {
        Self {
            tree: Tree::new(None),
        }
    }

    /// 添加根分类
    pub fn add_root(&mut self, id: impl Into<String>, name: impl Into<String>) -> anyhow::Result<String> {
        let id = id.into();
        let data = CategoryData::new(name);
        let node = Node::new(id.clone(), Some(data));
        self.tree.add_node(node, None)?;
        Ok(id)
    }

    /// 添加子分类
    pub fn add_child(
        &mut self,
        id: impl Into<String>,
        name: impl Into<String>,
        parent_id: impl AsRef<str>,
    ) -> anyhow::Result<String> {
        let id = id.into();
        let data = CategoryData::new(name);
        let node = Node::new(id.clone(), Some(data));
        let parent_id = parent_id.as_ref().to_string();
        self.tree.add_node(node, Some(&parent_id))?;
        Ok(id)
    }

    /// 获取节点
    pub fn get(&self, id: &str) -> Option<Node<String, CategoryData>> {
        self.tree.get_node_by_id(&id.to_string())
    }

    /// 获取节点数据
    pub fn get_data(&self, id: &str) -> Option<CategoryData> {
        self.tree.get_node_by_id(&id.to_string())
            .and_then(|n| n.get_value().map(|v| v.clone()))
    }

    /// 获取节点名称
    pub fn get_name(&self, id: &str) -> Option<String> {
        self.get_data(id).map(|d| d.name)
    }

    /// 获取父节点 ID
    pub fn get_parent_id(&self, id: &str) -> Option<String> {
        self.tree.get_ancestor_ids(&id.to_string()).ok()
            .and_then(|ancestors| ancestors.first().cloned())
    }

    /// 获取子节点 IDs
    pub fn get_children_ids(&self, id: &str) -> Vec<String> {
        self.tree.get_node_degree(&id.to_string()).ok()
            .map(|_degree| {
                // 获取所有后代，然后筛选直接子节点
                let descendants = self.tree.get_subtree(&id.to_string(), Some(1)).ok();
                descendants.map(|sub| {
                    sub.get_nodes().clone().into_iter()
                        .filter(|n| n.get_node_id() != id)
                        .map(|n| n.get_node_id())
                        .collect()
                }).unwrap_or_default()
            })
            .unwrap_or_default()
    }

    /// 获取节点深度（level）
    pub fn get_level(&self, id: &str) -> u32 {
        self.tree.get_node_depth(&id.to_string()).ok()
            .map(|d| d as u32)
            .unwrap_or(0)
    }

    /// 获取完整路径
    pub fn get_full_path(&self, id: &str) -> String {
        let mut ancestors = self.tree.get_ancestor_ids(&id.to_string()).ok()
            .unwrap_or_default();
        // ancestors 是从近到远，需要反转
        ancestors.reverse();
        
        let parts: Vec<String> = ancestors.iter()
            .filter_map(|aid| self.get_name(aid))
            .chain(self.get_name(id).into_iter())
            .collect();
        
        parts.join("/")
    }

    /// 根据路径查找分类 ID
    pub fn find_by_path(&self, path: &str) -> Option<String> {
        let parts: Vec<&str> = path.split('/').filter(|s| !s.is_empty()).collect();
        if parts.is_empty() {
            return None;
        }

        // 从根节点开始查找
        let roots = self.get_root_ids();
        let mut current_id = roots.iter()
            .find(|rid| self.get_name(rid).as_deref() == Some(parts[0]))
            .cloned()?;

        for part in &parts[1..] {
            let children = self.get_children_ids(&current_id);
            current_id = children.iter()
                .find(|cid| self.get_name(cid).as_deref() == Some(*part))
                .cloned()?;
        }

        Some(current_id)
    }

    /// 获取所有根节点 IDs
    pub fn get_root_ids(&self) -> Vec<String> {
        // tree-ds 可能有多个根，但我们只会有一个
        self.tree.get_nodes().clone().into_iter()
            .filter(|n| {
                // 检查是否为根节点（没有祖先）
                self.tree.get_ancestor_ids(&n.get_node_id()).ok()
                    .map(|a| a.is_empty())
                    .unwrap_or(true)
            })
            .map(|n| n.get_node_id())
            .collect()
    }

    /// 遍历所有节点（前序）
    pub fn traverse_pre_order(&self) -> Vec<(String, CategoryData, u32)> {
        let roots = self.get_root_ids();
        let mut result = Vec::new();
        
        for root_id in roots {
            if let Ok(ids) = self.tree.traverse(&root_id, TraversalStrategy::PreOrder) {
                for id in ids {
                    if let Some(data) = self.get_data(&id) {
                        let level = self.get_level(&id);
                        result.push((id, data, level));
                    }
                }
            }
        }
        
        result
    }

    /// 获取以指定节点为根的子树
    pub fn get_subtree(&self, id: &str) -> Option<CategoryTree> {
        let subtree = self.tree.get_subtree(&id.to_string(), None).ok()?;
        Some(CategoryTree { tree: subtree })
    }

    /// 删除节点
    pub fn remove(&mut self, id: &str) -> anyhow::Result<()> {
        use tree_ds::prelude::NodeRemovalStrategy;
        self.tree.remove_node(&id.to_string(), NodeRemovalStrategy::RemoveNodeAndChildren)?;
        Ok(())
    }

    /// 节点数量
    pub fn len(&self) -> usize {
        self.tree.get_nodes().len()
    }

    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    /// 从数据库记录列表构建树
    /// 
    /// 注意：tree-ds 库只支持单根节点，此方法仅供单一树结构使用
    pub fn from_records(records: Vec<CategoryRecord>) -> Self {
        let mut tree = Self::new();
        
        // 按 level 排序，确保父节点先添加
        let mut records = records;
        records.sort_by_key(|r| r.level);
        
        for record in records {
            if let Some(parent_id) = record.parent_id {
                // 子节点 - 先检查父节点是否存在
                if tree.get(&parent_id).is_some() {
                    let _ = tree.add_child(record.id, record.name, &parent_id);
                }
                // 如果父节点不存在，则跳过（数据不一致）
            } else {
                // 根节点 - 只添加第一个根节点（tree-ds 限制）
                if tree.get_root_ids().is_empty() {
                    let _ = tree.add_root(record.id, record.name);
                }
            }
        }
        
        tree
    }

    /// 转换为数据库记录列表
    pub fn to_records(&self) -> Vec<CategoryRecord> {
        self.traverse_pre_order()
            .into_iter()
            .map(|(id, data, level)| {
                let parent_id = self.get_parent_id(&id);
                let full_path = self.get_full_path(&id);
                CategoryRecord {
                    id,
                    name: data.name,
                    parent_id,
                    full_path,
                    level,
                    created_at: data.created_at,
                }
            })
            .collect()
    }
}

impl Default for CategoryTree {
    fn default() -> Self {
        Self::new()
    }
}

/// 数据库分类记录格式（用于与 SurrealDB 交互）
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CategoryRecord {
    pub id: String,
    pub name: String,
    pub parent_id: Option<String>,
    pub full_path: String,
    pub level: u32,
    pub created_at: Datetime,
}

/// 辅助方法
pub mod utils {
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

    /// 计算路径层级
    pub fn path_level(path: &str) -> u32 {
        parse_path(path).len() as u32 - 1
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_category_tree_add_root() {
        let mut tree = CategoryTree::new();
        let id = tree.add_root("cat_food", "餐饮").unwrap();
        assert_eq!(id, "cat_food");
        assert_eq!(tree.get_name(&id), Some("餐饮".to_string()));
        assert_eq!(tree.get_level(&id), 0);
        assert_eq!(tree.get_full_path(&id), "餐饮");
    }

    #[test]
    fn test_category_tree_add_child() {
        let mut tree = CategoryTree::new();
        let root = tree.add_root("cat_food", "餐饮").unwrap();
        let child = tree.add_child("cat_lunch", "午餐", &root).unwrap();
        
        assert_eq!(tree.get_name(&child), Some("午餐".to_string()));
        assert_eq!(tree.get_level(&child), 1);
        assert_eq!(tree.get_full_path(&child), "餐饮/午餐");
        assert_eq!(tree.get_parent_id(&child), Some(root));
    }

    #[test]
    fn test_category_tree_find_by_path() {
        let mut tree = CategoryTree::new();
        let food = tree.add_root("cat_food", "餐饮").unwrap();
        let lunch = tree.add_child("cat_lunch", "午餐", &food).unwrap();
        tree.add_child("cat_canteen", "食堂", &lunch).unwrap();

        assert_eq!(tree.find_by_path("餐饮"), Some("cat_food".to_string()));
        assert_eq!(tree.find_by_path("餐饮/午餐"), Some("cat_lunch".to_string()));
        assert_eq!(tree.find_by_path("餐饮/午餐/食堂"), Some("cat_canteen".to_string()));
        assert_eq!(tree.find_by_path("不存在"), None);
    }

    #[test]
    fn test_category_tree_traverse() {
        let mut tree = CategoryTree::new();
        let food = tree.add_root("cat_food", "餐饮").unwrap();
        let lunch = tree.add_child("cat_lunch", "午餐", &food).unwrap();
        let dinner = tree.add_child("cat_dinner", "晚餐", &food).unwrap();

        let result = tree.traverse_pre_order();
        assert_eq!(result.len(), 3);
        
        // 验证顺序（前序遍历：根 -> 左 -> 右）
        assert_eq!(result[0].0, "cat_food");
        assert_eq!(result[0].2, 0); // level
        assert!(result[1].0 == "cat_lunch" || result[1].0 == "cat_dinner");
        assert_eq!(result[1].2, 1); // level
    }

    #[test]
    fn test_utils_parse_path() {
        assert_eq!(
            utils::parse_path("餐饮/午餐/食堂"),
            vec!["餐饮", "午餐", "食堂"]
        );
        assert_eq!(utils::parse_path("餐饮"), vec!["餐饮"]);
    }

    #[test]
    fn test_utils_path_last_segment() {
        assert_eq!(utils::path_last_segment("餐饮/午餐"), "午餐");
        assert_eq!(utils::path_last_segment("餐饮"), "餐饮");
    }

    #[test]
    fn test_utils_parent_path() {
        assert_eq!(utils::parent_path("餐饮/午餐"), Some("餐饮"));
        assert_eq!(utils::parent_path("餐饮"), None);
    }
}
