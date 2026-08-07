//! 书签领域的模块入口与路由装配
//!
//! 本模块中的相对路由会在 `main.rs` 里嵌套到 `/bookmarks`，
//! 因此此处的 `/` 对应实际的 `POST /bookmarks` 接口

/// 导出书签 HTTP Handler 子模块
pub mod handlers;
/// 导出书签请求 DTO 与实体模型子模块
pub mod models;
/// 导出书签数据库访问子模块
pub mod repository;

// 导入 Axum 的 POST 路由构造器和 Router 类型
use axum::{Router, routing::post};
// 导入本领域路由所需的共享应用状态类型
use crate::core::state::AppState;

/// 创建携带 AppState 类型约束的书签子路由
pub fn router() -> Router<AppState> {
    // 创建一个空路由器作为当前领域的路由根，将相对路径 `/` 的 POST 请求绑定到创建书签 Handler
    Router::new().route("/", post(handlers::create::handle))
}
