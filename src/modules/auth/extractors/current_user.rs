//! 当前登录用户提取器

use axum::extract::FromRequestParts;
use axum::http::request::Parts;
use crate::app::state::AppState;
use crate::common::error::AppError;
use crate::modules::auth::model::user::User;
use crate::modules::auth::service::auth::authenticate;

/// 已经通过 Token 验证的当前用户
pub struct CurrentUser(pub User);

/// 从 Authorization 请求头中提取并验证当前用户
/// 使用 FromRequestParts 是因为这里只读取请求头，不读取请求体，因此它可以和 AppJson<T> 同时用于同一个 Handler
/// 
/// # Errors
/// - 如果请求头中没有 Authorization 字段，返回 AppError::Unauthorized
/// - 如果 Authorization 字段不是 Bearer Token 格式，返回 AppError::Unauthorized
impl FromRequestParts<AppState> for CurrentUser {
    // 如果提取失败，抛出 AppError
    type Rejection = AppError;

    /// 提取器的核心异步方法，从请求头中提取并验证当前用户
    /// 
    /// # Arguments
    /// * `parts`：HTTP 请求的头部部分
    /// * `state`：应用状态对象
    /// 
    /// # Returns
    /// * `Result<Self, Self::Rejection>`：成功时返回 CurrentUser 包裹的 User 对象，失败时返回 AppError
    async fn from_request_parts(parts: &mut Parts, state: &AppState, ) -> Result<Self, Self::Rejection> {
        // 读取 Authorization 请求头
        let authorization = parts
            .headers // 获取请求头
            .get("Authorization") // 获取 Authorization 字段
            .and_then(|value| value.to_str().ok()) // 将 HeaderValue 转换为 &str
            .ok_or(AppError::Unauthorized)?; // 如果没有 Authorization 字段，返回 Unauthorized 错误

        // 检查 Authorization 字段是否以 "Bearer " 开头，并提取 Token，否则返回 Unauthorized 错误
        let access_token = authorization.strip_prefix("Bearer ").ok_or(AppError::Unauthorized)?;
        // 如果 Token 为空，返回 Unauthorized 错误
        if access_token.is_empty() { return Err(AppError::Unauthorized); }

        // 验证 Token
        let user = authenticate(state, access_token).await?;

        // 将已验证的用户交给 Handler
        Ok(CurrentUser(user))
    }
}


