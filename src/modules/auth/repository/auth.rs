//! 认证会话数据访问层

use sqlx::{PgConnection, PgPool};
use crate::modules::auth::model::auth_session::AuthSession;

/// 将会话信息写入数据库
/// 
/// # Arguments
/// * `connection`：共享 PostgreSQL 连接池，事物持有的连接
/// * `user_id`：已认证用户的数据库主键
/// * `token_hash`：仅可验证、不可还原的 Session Token 哈希
/// * `token_expire_days`：会话过期天数
/// 
/// # Returns
/// * `Result<(), sqlx::Error>`：成功时返回 Ok，失败时返回 SQLx 错误
pub async fn insert_session(connection: &mut PgConnection, user_id: i64, token_hash: &str, token_expire_days: i32) -> Result<(), sqlx::Error> {
    // 将会话信息写入 auth_sessions 表
    sqlx::query(
        r#"
        INSERT INTO auth_sessions (user_id, token_hash, expires_at)
        VALUES ($1, $2, CURRENT_TIMESTAMP + make_interval(days => $3))
        "#,
    )
        .bind(user_id) // 安全绑定当前登录用户的主键
        .bind(token_hash) // 安全绑定不可逆的 Session Token 哈希
        .bind(token_expire_days) // 安全绑定会话过期天数
        .execute(connection) // 在连接池中异步执行会话插入
        .await?; // 等待写入完成，并将 SQLx 错误向上传播
    Ok(()) // 明确表示会话创建成功
}

/// 根据 HMAC 后的 Token 哈希查询有效 Session
///
/// # Arguments
/// * `pool`：共享 PostgreSQL 连接池
/// * `token_hash`：仅可验证、不可还原的 Session Token 哈希
/// 
/// # Returns
/// * `Result<Option<AuthSession>, sqlx::Error>`：成功时返回 Some(AuthSession) 或 None，失败时返回 SQLx 错误
pub async fn find_active_session(pool: &PgPool, token_hash: &str, ) -> Result<Option<AuthSession>, sqlx::Error> {
    // 只查询未过期且未撤销的 Session
    sqlx::query_as::<_, AuthSession>(
        r#"
        SELECT id, user_id, token_hash, expires_at, last_used_at, revoked_at, ip_address, user_agent, created_at
        FROM auth_sessions
        WHERE token_hash = $1
            AND expires_at > CURRENT_TIMESTAMP
            AND revoked_at IS NULL
        "#,
    )
        .bind(token_hash)
        .fetch_optional(pool)
        .await
}
