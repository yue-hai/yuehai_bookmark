//! 登录相关 service

use crate::app::state::AppState;
use crate::common::error::AppError;
use crate::modules::auth::dto::login::{LoginRequest, LoginResponse};
use argon2::{Argon2, PasswordHash, PasswordVerifier};
use hmac::{Hmac, KeyInit, Mac};
use sha2::Sha256;
use crate::modules::auth::model::user::User;
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
    
    // 查询未删除且状态为 active 的用户凭据，用户不存在或已禁用时返回统一凭据错误
    let user = user::find_active_user_by_email(&state.database_pool, &request.email).await?.ok_or(AppError::InvalidCredentials)?;
    // 解析数据库中密码哈希
    let password_hash = PasswordHash::new(&user.password_hash).map_err(|_| AppError::Internal)?;
    // 创建默认的 Argon2id 密码校验器，使用客户端明文密码与数据库哈希进行安全比对，密码不匹配时统一返回凭据错误
    Argon2::default().verify_password(request.password.as_bytes(), &password_hash).map_err(|_| AppError::InvalidCredentials)?;
    
    // 生成原始 Token，并计算只保存到数据库中的 HMAC
    let (access_token, token_hash) = issue_session_token(state.token_hash_secret.as_bytes())?;
    
    // 从连接池取得一条连接，并开始数据库事务
    let mut transaction = state.database_pool.begin().await?;
    // 持久化当前用户的 Session Token 哈希
    auth::insert_session(&mut transaction, user.id, &token_hash, state.token_expire_days).await?;
    // 提交事务
    transaction.commit().await?;
    
    // 构造并返回登录成功响应
    Ok(LoginResponse::from_user(user, access_token, "Bearer"))
}

/// 生成原始 Session Token，并返回 Token 与其 HMAC
///
/// # Arguments
/// * `secret`：服务端密钥，用于计算 Token 的 HMAC
/// 
/// # Returns
/// * `Result<(String, String), AppError>`：成功时返回原始 Token 和 HMAC，失败时返回应用错误
pub(crate) fn issue_session_token(secret: &[u8], ) -> Result<(String, String), AppError> {
    // 声明 32 字节数组，用于保存安全随机数据
    let mut token_bytes = [0_u8; 32];
    // 使用操作系统安全随机源生成 Token
    getrandom::fill(&mut token_bytes).map_err(|_| AppError::Internal)?;
    
    // 将随机字节编码为 64 个十六进制字符
    let access_token = token_bytes.iter().map(|byte| format!("{byte:02x}")).collect::<String>();
    // 对原始 Token 计算 HMAC-SHA-256
    let token_hash = hash_access_token(&access_token, secret)?;
    
    // 返回原始 Token 和数据库保存的 HMAC
    Ok((access_token, token_hash))
}

/// 使用服务端密钥计算 Token 的 HMAC-SHA-256
/// 
/// # Arguments
/// * `access_token`：原始 Token 字符串切片
/// * `secret`：服务端密钥字节切片
/// 
/// # Returns
/// * `Result<String, AppError>`：成功时返回 64 个十六进制字符的 HMAC，失败时返回应用错误
fn hash_access_token(access_token: &str, secret: &[u8], ) -> Result<String, AppError> {
    // 使用服务端密钥初始化 HMAC
    let mut mac = Hmac::<Sha256>::new_from_slice(secret).map_err(|_| AppError::Internal)?;
    // 将原始 Token 作为 HMAC 的消息输入
    mac.update(access_token.as_bytes());
    
    // 完成 HMAC 计算，得到 32 字节结果
    let result = mac.finalize().into_bytes();
    // 将 32 字节结果编码为 64 个十六进制字符
    Ok(result.iter().map(|byte| format!("{byte:02x}")).collect())
}

/// 根据客户端 Token 验证当前用户
/// 
/// # Arguments
/// * `state`：应用状态，包含数据库连接池等共享资源
/// * `access_token`：客户端提供的原始 Token 字符串切片
/// 
/// # Returns
/// * `Result<User, AppError>`：成功时返回当前用户信息，失败时返回应用错误
pub async fn authenticate(state: &AppState, access_token: &str, ) -> Result<User, AppError> {
    // Token 不能为空
    if access_token.is_empty() { return Err(AppError::Unauthorized); }
    
    // 使用同一个服务端密钥计算客户端 Token 的 HMAC
    let token_hash = hash_access_token(access_token, state.token_hash_secret.as_bytes() )?;
    // 根据 HMAC 直接查询有效 Session
    let session = auth::find_active_session(&state.database_pool, &token_hash ).await?.ok_or(AppError::Unauthorized)?;
    // 根据 Session 关联的用户 ID 查询有效用户
    let current_user = user::find_active_user_by_id(&state.database_pool, session.user_id, ).await?.ok_or(AppError::Unauthorized)?;
    
    // 返回当前用户信息
    Ok(current_user)
}
