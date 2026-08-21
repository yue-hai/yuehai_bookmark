//! 登录相关 service

use crate::app::state::AppState;
use crate::common::error::AppError;
use crate::modules::auth::dto::login::{LoginRequest, LoginResponse};
use argon2::password_hash::SaltString;
use argon2::{Argon2, PasswordHash, PasswordHasher, PasswordVerifier};
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
    let user = user::find_active_user_by_email(&state.database_pool, &request.email).await?;
    // 用户不存在或已禁用时返回统一凭据错误，不泄露具体原因
    let user = user.ok_or(AppError::InvalidCredentials)?;

    // 解析数据库中密码哈希
    let password_hash = PasswordHash::new(&user.password_hash).map_err(|_| AppError::Internal)?;
    // 创建默认的 Argon2id 密码校验器，使用客户端明文密码与数据库哈希进行安全比对，密码不匹配时统一返回凭据错误
    Argon2::default().verify_password(request.password.as_bytes(), &password_hash).map_err(|_| AppError::InvalidCredentials)?;

    // 生成原始 Token，并生成只保存到数据库的 Argon2 哈希值
    let (access_token, token_hash) = issue_session_token()?;
    // 从连接池取得一条连接，并开始数据库事务
    let mut transaction = state.database_pool.begin().await?;
    // 持久化当前用户的 Session Token 哈希
    auth::insert_session(&mut transaction, user.id, &token_hash, state.token_expire_days).await?;
    // 提交事务
    transaction.commit().await?;
    
    // 构造并返回登录成功响应
    Ok(LoginResponse::from_user(user, access_token, "Bearer"))
}

/// 生成原始 Token，并返回原始值与数据库哈希值
/// 
/// # Returns
/// * `Result<(String, String), AppError>`：成功时返回原始 Token和数据库哈希，失败时返回应用错误
pub(crate) fn issue_session_token() -> Result<(String, String), AppError> {
    // 声明一个长度为 32 字节的数组，初始化为全 0，作为承接安全随机源的缓冲区
    let mut token_bytes = [0_u8; 32];
    // 调用操作系统底层的安全随机数生成器（如 Linux 的 /dev/urandom）填充数组，若失败（如系统熵池耗尽）则映射为内部错误
    getrandom::fill(&mut token_bytes).map_err(|_| AppError::Internal)?;
    // 将 32 个随机字节通过迭代器逐一转换为两位的十六进制小写字符 (`{byte:02x}`)，最终拼接拼装成 64 字符长度的明文 Token 字符串
    let access_token = token_bytes.iter().map(|byte| format!("{byte:02x}")).collect::<String>();

    // 因为要把 Token Hash 落盘，所以这里再次在栈上分配 32 字节空间，用于存放 Token Hash 专用的盐值
    let mut salt_bytes = [0_u8; 32];
    // 调用操作系统底层的安全随机数生成器填充盐值数组，若失败则映射为内部错误
    getrandom::fill(&mut salt_bytes).map_err(|_| AppError::Internal)?;

    // 将 32 字节的原始随机数据编码为符合密码学规范的 Base64 格式盐值字符串，解析失败则抛出内部错误
    let salt = SaltString::encode_b64(&salt_bytes).map_err(|_| AppError::Internal)?;
    // 使用 Argon2 算法对用户的明文密码进行哈希
    let token_hash = Argon2::default()
        .hash_password(access_token.as_bytes(), &salt) // 将明文 token 和随机盐传入 Argon2 哈希函数
        .map_err(|_| AppError::Internal)? // 如果底层的哈希计算意外失败（如内存耗尽等），安全地映射为应用层的内部错误
        .to_string(); // 将生成的哈希值转换为字符串，该字符串会自动包含算法参数、盐和哈希结果，便于日后校验

    // 同时返回客户端原始 Token 和数据库安全哈希
    Ok((access_token, token_hash))
}
