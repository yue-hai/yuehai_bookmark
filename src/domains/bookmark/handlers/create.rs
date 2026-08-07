//! `POST /bookmarks` 的请求处理逻辑
//!
//! Axum 在调用 Handler 前会提取共享状态并反序列化 JSON 请求体
//! 处理函数随后调用仓储层写入数据，底层 `sqlx::Error` 会通过 `?` 自动转换为 `AppError`

// 导入从请求中提取共享状态和 JSON 载荷的 Axum 类型
use axum::{Json, extract::State};
// 导入本 Handler 需要的错误、响应、状态、DTO 和仓储别名
use crate::{
    // 导入统一错误、成功响应和共享状态
    core::{error::AppError, response::ApiResponse, state::AppState},
    // 导入书签领域的创建请求 DTO 和 postgres 仓储模块
    domains::bookmark::{models::dto::create::CreateBookmarkReq, repository::postgres},
};

/// 创建一条书签记录并返回其数据库 ID
pub async fn handle(
    // 从 Router 注入的 AppState 中提取连接池等共享资源
    State(state): State<AppState>,
    // 将 HTTP 请求体 JSON 自动反序列化为创建书签请求 DTO
    Json(payload): Json<CreateBookmarkReq>,
    // 成功时返回统一 JSON，失败时返回可被 Axum 转换的 AppError
) -> Result<impl axum::response::IntoResponse, AppError> {
    // 调用仓储层执行持久化；`?` 会自动把 sqlx::Error 转为 AppError
    let id = postgres::insert(&state.db_pool, &payload).await?;

    // 使用统一成功响应格式返回新建记录的 ID
    Ok(ApiResponse::success(id))
}
