//! 认证会话数据访问层

use sqlx::PgPool;

/// 为已成功验证密码的用户创建服务端登录会
/// 
/// # Arguments
/// * `pool`：共享 PostgreSQL 连接池
/// * `user_id`：已认证用户的数据库主键
/// * `token_hash`：仅可验证、不可还原的 Session Token 哈希
/// 
/// # Returns
/// * `Result<(), sqlx::Error>`：成功时返回 Ok，失败时返回 SQLx 错误
pub async fn insert_session(pool: &PgPool, user_id: i64, token_hash: &str) -> Result<(), sqlx::Error> {
    // 将会话信息写入 auth_sessions 表
    sqlx::query(
        r#"
        INSERT INTO auth_sessions (user_id, token_hash, expires_at)
        VALUES ($1, $2, CURRENT_TIMESTAMP + INTERVAL '30 days')
        "#,
    )
        .bind(user_id) // 安全绑定当前登录用户的主键
        .bind(token_hash) // 安全绑定不可逆的 Session Token 哈希
        .execute(pool) // 在连接池中异步执行会话插入
        .await?; // 等待写入完成，并将 SQLx 错误向上传播
    Ok(()) // 明确表示会话创建成功
}
