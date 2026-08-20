//! 顶层路由
//!
//! 只负责 API 版本、中间件和业务模块路由的组合

use axum::Router;

use crate::app::state::AppState;
use crate::modules::{auth};

/// 创建应用顶层 Router
/// 
/// # Arguments
/// * `state`：应用状态，包含数据库连接池等共享资源
/// 
/// # Returns
/// * `Router`：顶层路由，包含所有业务模块的路由
pub fn build(state: AppState) -> Router {
    // 创建尚未注入 AppState 的 API 子 Router
    let api = Router::new()
        // 合并认证模块的 /auth/login 等相对路径
        .merge(auth::routes::router());

    // 创建应用最外层 Router
    Router::new()
        // 统一为全部业务接口添加 /api 前缀
        .nest("/api", api)
        // 最后注入 AppState，使所有 Handler 可提取数据库连接池
        .with_state(state)
}