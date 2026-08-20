//! 登录相关接口

use crate::app::state::AppState;
use crate::common::error::AppError;
use crate::common::http::response::ApiResponse;
use axum::{Json, extract::State, response::IntoResponse};
use crate::modules::auth::dto::login::LoginRequest;
use crate::modules::auth::service;

/// 定义 POST /api/auth/login 的异步 Handler
/// 
/// # Arguments
/// * `State(state)`：从顶层 Router 注入的共享状态中取得 PgPool
/// * `Json(payload)`：将 HTTP JSON 请求体反序列化为 LoginRequest
/// 
/// # Returns
/// * `Result<impl IntoResponse, AppError>`：成功时返回 HTTP 200 和 JSON，失败时返回 HTTP 错误
pub async fn login(State(state): State<AppState>, Json(payload): Json<LoginRequest>) -> Result<impl IntoResponse, AppError> {
    // 调用 Service 完成校验、查库、验密和创建登录会话，成功时返回 JSON，失败时交给 AppError 转换 HTTP 错误
    let response = service::auth::login(&state, payload).await?;
    // 使用统一成功响应包装 LoginResponse，并返回 HTTP 200
    Ok(ApiResponse::success(response))
}
