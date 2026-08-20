//! 用户相关接口

use axum::extract::State;
use axum::Json;
use axum::response::IntoResponse;
use crate::app::state::AppState;
use crate::common::error::AppError;
use crate::common::http::response::ApiResponse;
use crate::modules::auth::dto::register::RegisterRequest;
use crate::modules::auth::service;

/// 定义 POST /api/auth/register 的异步 Handler
/// 
/// # Arguments
/// * `State(state)`：从顶层 Router 注入的共享状态中取得 PgPool
/// * `Json(payload)`：将 HTTP JSON 请求体反序列化为 RegisterRequest
/// 
/// # Returns
/// * `Result<impl IntoResponse, AppError>`：成功时返回 HTTP 200 和 JSON，失败时返回 HTTP 错误
pub async fn register(State(state): State<AppState>, Json(payload): Json<RegisterRequest>) -> Result<impl IntoResponse, AppError> {
    // 调用 Service 完成注册逻辑，包括验证、哈希密码、插入数据库和签发 Token，成功时返回 JSON，失败时交给 AppError 转换 HTTP 错误
    let response = service::user::register(&state, payload).await?;
    // 使用统一成功响应包装 RegisterResponse，并返回 HTTP 200
    Ok(ApiResponse::success(response))
}
