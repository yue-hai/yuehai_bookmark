//! 注册相关 DTO

use serde::Deserialize;
use crate::common::error::AppError;

// 注册请求体结构体
#[derive(Deserialize)]
pub struct RegisterRequest {
    // 邮箱
    pub email: String,
    // 密码
    pub password: String,
    // 显示名称
    pub display_name: String,
}

/// RegisterRequest 的实现块
impl RegisterRequest {
    /// 检查注册请求，并返回清理后的注册数据
    /// 
    /// # Arguments
    /// * `self`：注册请求实例
    /// 
    /// # Returns
    /// * `Result<Self, AppError>`：成功时返回清理后的注册请求，失败时返回应用错误
    pub fn validate(self) -> Result<Self, AppError> {
        // 去除邮箱首尾空白并统一转小写
        let email = self.email.trim().to_lowercase();
        // 邮箱不可为空
        if email.is_empty() { return Err(AppError::BadRequest("邮箱不能为空")); }
        // 邮箱格式必须合法
        if !is_valid_email(&email) { return Err(AppError::BadRequest("邮箱格式不正确")); }
        
        // 密码不可为空
        if self.password.is_empty() { return Err(AppError::BadRequest("密码不能为空")); }
        // 密码长度必须至少为 8 个字符
        if self.password.chars().count() < 8 { return Err(AppError::BadRequest("密码长度不能少于 8 个字符")); }
        
        // 去除显示名称首尾空白
        let display_name = self.display_name.trim().to_owned();
        // 显示名称不可为空
        if display_name.is_empty() { return Err(AppError::BadRequest("显示名称不能为空")); }
        // 显示名称长度不能超过 100 个字符
        if display_name.chars().count() > 100 { return Err(AppError::BadRequest("显示名称长度不能超过 100 个字符")); }
        
        // 返回清理后的注册请求
        Ok(Self { email, password: self.password, display_name})
    }
}

/// 简单的邮箱检查
/// 
/// # Arguments
/// * `email`：待检查的邮箱字符串切片
/// 
/// # Returns
/// * `bool`：邮箱格式是否合法
fn is_valid_email(email: &str) -> bool {
    // 邮箱必须包含 '@' 且分为本地部分和域名部分
    let Some((local, domain)) = email.split_once('@') else { return false; };
    // 本地部分和域名部分都不能为空
    if local.is_empty() || domain.is_empty() { return false; }
    // 域名部分不能以 '.' 开头或结尾
    if domain.starts_with('.') || domain.ends_with('.') { return false; }
    
    // 域名部分必须包含至少一个 '.'，确保有顶级域名
    domain.contains('.')
}