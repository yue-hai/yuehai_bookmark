//! 注册 /api/auth 下的认证相关路由

use crate::app::state::AppState;
use crate::modules::auth::handlers;
use axum::Router;
use axum::routing::post;

/// 创建携带 AppState 类型约束子路由
/// 
/// # Returns
/// * `Router<AppState>`：返回一个携带 AppState 的路由器，用于注册认证相关的路由
pub fn router() -> Router<AppState> {
    // 创建一个空路由器作为当前领域的路由根
    Router::new()
        // 注册 /api/auth/register
        .route("/auth/register", post(handlers::user::register))
        // 登录 /api/auth/login
        .route("/auth/login", post(handlers::auth::login))
}

