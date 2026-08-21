//! 当前用户相关 DTO

use chrono::{DateTime, Utc};
use serde::Serialize;

use crate::modules::auth::model::user::{SystemRole, User, UserStatus};

/// 当前用户公开信息
#[derive(Serialize)]
pub struct MeResponse {
    /// 用户 ID
    pub id: i64,
    /// 登录邮箱
    pub email: String,
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
}

/// MeResponse 的实现块
impl MeResponse {
    /// 将数据库用户模型转换为公开响应 DTO
    pub fn from_user(user: User) -> Self {
        Self {
            id: user.id,
            email: user.email,
            display_name: user.display_name,
            system_role: user.system_role,
            status: user.status,
            email_verified_at: user.email_verified_at,
            last_login_at: user.last_login_at,
            created_at: user.created_at,
            updated_at: user.updated_at,
        }
    }
}
