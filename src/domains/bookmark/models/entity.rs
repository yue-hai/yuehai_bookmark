//! 书签领域实体定义
//!
//! `Bookmark` 表示一条已持久化的记录；读取接口实现后可将其作为响应数据模型

// 导入 Serialize 派生宏，使实体可被编码为 JSON
use serde::Serialize;

/// 表示数据库中的一条书签记录
#[derive(Serialize)]
pub struct Bookmark {
    /// 书签 id
    pub id: i64,
    /// 书签标题
    pub title: String,
    /// 书签 URL
    pub url: String,
}
