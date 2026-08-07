//! 全局共享状态模块
//!
//! 定义由 Axum `State` 提取器在请求生命周期中动态注入给业务 Handler 的应用上下文

// 导入 sqlx 提供的 PostgreSQL 连接池类型
use sqlx::PgPool;

/// 应用全局状态上下文容器 <br>
/// 宏 `#[derive(Clone)]` 是 Axum 运行时的硬性要求： <br>
/// 每当一个新的 HTTP 请求进入时，框架会为当前协程任务（Task）克隆一份该状态实例 <br>
/// 这里的 Clone 是极其轻量的操作，仅在底层执行原子级的引用计数 +1，绝不会复制物理 TCP 连接 <br>
#[derive(Clone)]
pub struct AppState {
    /// 异步数据库连接池句柄，所有的 Handler 均通过该句柄派发 SQL 操作，实现单例级别的高并发网络复用
    pub db_pool: PgPool,
}
