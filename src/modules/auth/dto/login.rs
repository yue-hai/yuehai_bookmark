//! 登录相关 DTO

use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc};
use crate::common::error::AppError;
use crate::modules::auth::model::user::{SystemRole, User, UserStatus};

// 登录请求体结构体
#[derive(Deserialize)]
pub struct LoginRequest {
    // 邮箱
    pub email: String,
    // 密码
    pub password: String,
}

// 登录响应体结构体
#[derive(Serialize)]
pub struct LoginResponse {
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
    
    /// 用户 Session Token
    pub access_token: String,
    /// Token 类型，明确客户端应以 Bearer 方式使用该 Token
    pub token_type: &'static str,
}

/// LoginRequest 的实现块
impl LoginRequest {
    /// 验证登录请求
    /// 
    /// # Arguments
    /// * `self`：登录请求实例
    /// 
    /// # Returns
    /// * `Result<Self, AppError>`：成功时返回清理后的登录请求，失败时返回应用错误
    pub fn validate(self) -> Result<Self, AppError> {
        // 去除邮箱首尾空白并统一转小写
        let email = self.email.trim().to_lowercase();
        // 邮箱不可为空
        if email.is_empty() { return Err(AppError::BadRequest("邮箱不能为空")); }
        
        // 密码不可为空
        if self.password.is_empty() { return Err(AppError::BadRequest("密码不能为空")); }
        
        // 返回清理后的登录请求
        Ok(Self { email, password: self.password })
    }
}

/// LoginResponse 的实现块
impl LoginResponse {
    /// 从 User 模型、原始 Token 和 Token 类型构造 LoginResponse
    pub fn from_user(user: User, access_token: String, token_type: &'static str) -> Self {
        Self {
            email: user.email,
            display_name: user.display_name,
            system_role: user.system_role,
            status: user.status,
            email_verified_at: user.email_verified_at,
            last_login_at: user.last_login_at,
            created_at: user.created_at,
            updated_at: user.updated_at,
            access_token,
            token_type,
        }
    }
}