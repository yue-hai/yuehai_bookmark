//! 用户模型

use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use chrono::{DateTime, Utc};

/// 系统角色枚举
#[derive(sqlx::Type, Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[sqlx(type_name = "VARCHAR", rename_all = "lowercase")]
#[serde(rename_all = "lowercase")]
pub enum SystemRole {
    User,
    Admin,
}

/// 账号状态枚举
#[derive(sqlx::Type, Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[sqlx(type_name = "VARCHAR", rename_all = "lowercase")]
#[serde(rename_all = "lowercase")]
pub enum UserStatus {
    Pending,
    Active,
    Disabled,
}

/// 允许 SQLx 将查询结果映射为内部用户凭据结构
#[derive(FromRow)]
/// 用户模型
pub struct User {
    /// 用户 ID
    pub id: i64,
    /// 登录邮箱
    pub email: String,
    /// 密码哈希
    pub password_hash: String,
    /// 显示名称
    pub display_name: String,
    /// 系统角色
    pub system_role: SystemRole,
    /// 账号状态
    pub status: UserStatus,
    /// 邮箱验证时间
    pub email_verified_at: Option<DateTime<Utc>>,
    /// 最近登录时间
    pub last_login_at: Option<DateTime<Utc>>,
    /// 创建时间
    pub created_at: DateTime<Utc>,
    /// 更新时间
    pub updated_at: DateTime<Utc>,
    /// 软删除时间
    pub deleted_at: Option<DateTime<Utc>>,
}