//! 认证会话模型

use sqlx::FromRow;
use sqlx::types::chrono::{DateTime, Utc};
use std::net::IpAddr;

/// 认证会话模型
#[derive(FromRow, Debug)]
pub struct AuthSession {
    /// 会话主键
    pub id: i64,
    /// 所属用户 ID (外键)
    pub user_id: i64,
    /// Token 哈希值 (全局唯一)
    pub token_hash: String,
    /// 过期时间
    pub expires_at: DateTime<Utc>,
    /// 最近使用时间，允许为空
    pub last_used_at: Option<DateTime<Utc>>,
    /// 撤销时间，允许为空
    pub revoked_at: Option<DateTime<Utc>>,
    /// 客户端 IP，映射自 INET 列；允许为空
    pub ip_address: Option<IpAddr>,
    /// 客户端标识 (User-Agent)；TEXT 没有 NOT NULL，需用 Option 包裹
    pub user_agent: Option<String>,
    /// 创建时间
    pub created_at: DateTime<Utc>,
}