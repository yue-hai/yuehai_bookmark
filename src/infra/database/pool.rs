//! PostgreSQL 连接池初始化
//!
//! 服务启动时异步创建连接池；连接失败会立即终止启动，避免应用在无法访问数据的状态下监听 HTTP 端口

use sqlx::{PgPool, postgres::PgPoolOptions};

/// 根据 DATABASE_URL 创建并返回 PostgreSQL 连接池
/// 
/// # Arguments
/// * `database_url`：数据库连接字符串，格式为 `postgres://username:password@host:port/database_name`
///
/// # Returns
/// * `Result<PgPool, sqlx::Error>`：成功时返回 PgPool，失败时返回 sqlx::Error
pub async fn init_pool(database_url: &str) -> Result<PgPool, sqlx::Error> {
    // 创建一个采用 sqlx 默认配置的连接池构建器
    PgPoolOptions::new()
        // 限制连接池中可同时建立的最大数据库连接数
        .max_connections(20)
        // 异步连接数据库并构造 PgPool
        .connect(database_url)
        // 等待连接操作完成，取得 Result<PgPool, sqlx::Error>
        .await
}
