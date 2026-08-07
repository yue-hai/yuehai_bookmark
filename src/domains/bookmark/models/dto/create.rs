//! 创建书签接口的请求 DTO
//!
//! 客户端需提供 `title` 和 `url`；Axum 会利用 Serde 将 JSON 映射到该结构体

// 导入 Deserialize 派生宏，使 Axum 可以从 JSON 构造本结构体。
use serde::Deserialize;

/// `POST /bookmarks` 接收的请求数据
#[derive(Deserialize)]
pub struct CreateBookmarkReq {
    /// 书签标题
    pub title: String,
    /// 书签 URL
    pub url: String,
}
