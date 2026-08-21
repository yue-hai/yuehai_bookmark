//! 用户数据访问层

use sqlx::{PgConnection, PgPool};
use crate::modules::auth::model::user::User;

/// 用户注册
/// 
/// # Arguments
/// * `connection`：共享 PostgreSQL 连接池，事务持有的连接
/// * `email`：Service 已规范化的邮箱字符串切片
/// * `password_hash`：Service 已哈希的密码字符串切片
/// * `display_name`：Service 已规范化的显示名称字符串切片
/// 
/// # Returns
/// * `Result<User, sqlx::Error>`：成功时返回新创建的 User 结构，失败时返回 SQLx 错误
pub async fn insert_user(connection: &mut PgConnection, email: &str, password_hash: &str, display_name: &str) -> Result<User, sqlx::Error> {
    // 插入新用户记录，并返回完整的 User 结构
    sqlx::query_as::<_, User>(
        r#"
        INSERT INTO users (email, password_hash, display_name)
        VALUES ($1, $2, $3)
        RETURNING id, email, password_hash, display_name , system_role, status, email_verified_at, last_login_at, created_at, updated_at, deleted_at
        "#,
    )
        .bind(email) // 将邮箱安全绑定为 PostgreSQL 的第一个参数，避免 SQL 注入
        .bind(password_hash) // 将密码哈希安全绑定为 PostgreSQL 的第二个参数，避免 SQL 注入
        .bind(display_name) // 将显示名称安全绑定为 PostgreSQL 的第三个参数，避免 SQL 注入
        .fetch_one(connection) // 查询并返回新插入的用户记录
        .await
}

/// 按邮箱查询可登录用户
/// 
/// # Arguments
/// * `pool`：共享 PostgreSQL 连接池
/// * `email`：Service 已规范化的邮箱字符串切片
/// 
/// # Returns
/// * `Result<Option<User>, sqlx::Error>`：成功时返回 Some(User) 或 None，失败时返回 SQLx 错误
pub async fn find_active_user_by_email(pool: &PgPool, email: &str) -> Result<Option<User>, sqlx::Error> {
    // 根据邮箱查询未删除且状态为 active 的用户记录
    sqlx::query_as::<_, User>(
        r#"
        SELECT id, email, password_hash, display_name , system_role, status, email_verified_at, last_login_at, created_at, updated_at, deleted_at
        FROM users
        WHERE lower(btrim(email)) = $1
            AND status = 'active'
            AND deleted_at IS NULL
        "#,
    )
        .bind(email) // 将邮箱安全绑定为 PostgreSQL 的第一个参数，避免 SQL 注入
        .fetch_optional(pool) // 查询零行或一行用户记录
        .await
}

/// 按用户 ID 查询可以使用系统的用户
///
/// # Arguments
/// * `pool`：共享 PostgreSQL 连接池
/// * `user_id`：用户 ID
///
/// # Returns
/// * `Result<Option<User>, sqlx::Error>`：成功时返回 Some(User) 或 None，失败时返回 SQLx 错误
pub async fn find_active_user_by_id(pool: &PgPool, user_id: i64, ) -> Result<Option<User>, sqlx::Error> {
    // Token 有效不代表用户永远有效，所以这里再次检查用户状态
    sqlx::query_as::<_, User>(
        r#"
        SELECT id, email, password_hash, display_name, system_role, status, email_verified_at, last_login_at, created_at, updated_at, deleted_at
        FROM users
        WHERE id = $1
            AND status = 'active'
            AND deleted_at IS NULL
        "#,
    )
        .bind(user_id) // 将用户 ID 安全绑定为 PostgreSQL 的第一个参数，避免 SQL 注入
        .fetch_optional(pool)// 查询零行或一行用户记录
        .await
}