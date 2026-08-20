//! 登录相关 service

use crate::app::state::AppState;
use crate::common::error::AppError;
use crate::modules::auth::dto::login::{LoginRequest, LoginResponse};
use argon2::password_hash::SaltString;
use argon2::{Argon2, PasswordHash, PasswordHasher, PasswordVerifier};
use rand_core::{OsRng, RngCore};
use crate::modules::auth::repository::{auth, user};

/// 登录 service，处理登录请求的业务逻辑，包括验证用户凭据、生成 Session Token 并返回给客户端
/// 
/// # Arguments
/// * `state`：应用状态，包含数据库连接池等共享资源
/// * `request`：登录请求，包含邮箱和密码
/// 
/// # Returns
/// * `Result<LoginResponse, AppError>`：成功时返回登录响应，包含用户信息和 Session Token，失败时返回应用错误
pub async fn login(state: &AppState, request: LoginRequest) -> Result<LoginResponse, AppError> {
    // 验证登录请求，确保邮箱和密码符合要求
    let request = request.validate()?;
    
    // 查询未删除且状态为 active 的用户凭据
    let user = user::find_active_user_by_email(&state.database_url, &request.email).await?;
    // 用户不存在或已禁用时返回统一凭据错误，不泄露具体原因
    let user = user.ok_or(AppError::InvalidCredentials)?;

    // 解析数据库中密码哈希
    let password_hash = PasswordHash::new(&user.password_hash).map_err(|_| AppError::Internal)?;
    // 创建默认的 Argon2id 密码校验器，使用客户端明文密码与数据库哈希进行安全比对，密码不匹配时统一返回凭据错误
    Argon2::default().verify_password(request.password.as_bytes(), &password_hash).map_err(|_| AppError::InvalidCredentials)?;

    // 生成原始 Token，并生成只保存到数据库的 Argon2 哈希值
    let (access_token, token_hash) = issue_session_token()?;
    // 持久化当前用户的 Session Token 哈希
    auth::insert_session(&state.database_url, user.id, &token_hash).await?;
    
    // 构造并返回登录成功响应
    Ok(LoginResponse::from_user(user, access_token, "Bearer"))
}

/// 生成原始 Token，并返回原始值与数据库哈希值
/// 
/// # Returns
/// * `Result<(String, String), AppError>`：成功时返回原始 Token和数据库哈希，失败时返回应用错误
pub(crate) fn issue_session_token() -> Result<(String, String), AppError> {
    // 创建由操作系统提供熵的安全随机数生成器
    let mut random = OsRng;
    // 准备 32 字节随机数据，即 256 位熵
    let mut token_bytes = [0_u8; 32];
    // 使用操作系统安全随机源填满 Token 字节数组。
    random.fill_bytes(&mut token_bytes);

    // 遍历随机字节，以构造可安全放入 HTTP Header 的文本 Token
    let access_token = token_bytes
        .iter() // 逐个借用数组中的字节
        .map(|byte| format!("{byte:02x}")) // 将每个字节编码为固定两位十六进制文本
        .collect::<String>(); // 合并为 64 个字符的原始 Token

    // 为 Token 哈希生成独立随机盐，避免可预计算攻击
    let salt = SaltString::generate(&mut random);
    // 创建默认的 Argon2id 哈希器
    let token_hash = Argon2::default()
        .hash_password(access_token.as_bytes(), &salt) // 对原始 Token 进行不可逆哈希
        .map_err(|_| AppError::Internal)? // 若哈希器发生内部错误则返回安全的 500 错误
        .to_string(); // 将 PHC 格式哈希转换为可存入 auth_sessions.token_hash 的字符串

    // 同时返回客户端原始 Token 和数据库安全哈希
    Ok((access_token, token_hash))
}
