//! 全局状态
//!
//! 定义由 Axum `State` 提取器在请求生命周期中动态注入给业务 Handler 的应用上下文

use sqlx::PgPool;

/// derive(Clone) 是 Auxum 框架要求的约定，主要是为了在请求生命周期中实现状态的共享和传递
/// 每当一个新的 HTTP 请求进入时，框架会为当前协程任务（Task）克隆一份极其轻量的该状态实例
#[derive(Clone)]
/// 应用全局状态上下文容器
pub struct AppState {
    /// 异步数据库连接池句柄，所有的 Handler 均通过该句柄派发 SQL 操作，实现单例级别的高并发网络复用
    pub database_url: PgPool,
}
