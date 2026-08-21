//! 用户相关 service

use argon2::{Argon2, PasswordHasher};
use argon2::password_hash::SaltString;
use crate::app::state::AppState;
use crate::common::error::AppError;
use crate::modules::auth::dto::login::LoginResponse;
use crate::modules::auth::dto::register::RegisterRequest;
use crate::modules::auth::repository::{auth, user};
use crate::modules::auth::service::auth::issue_session_token;

/// 注册 service，处理注册请求的业务逻辑，包括验证用户信息、创建新用户并返回给客户端
/// 
/// # Arguments
/// * `state`：应用状态，包含数据库连接池等共享资源
/// * `request`：注册请求，包含邮箱、密码和显示名称
/// 
/// # Returns
/// * `Result<LoginResponse, AppError>`：成功时返回登录响应，包含用户信息，失败时返回应用错误
pub async fn register(state: &AppState, request: RegisterRequest) -> Result<LoginResponse, AppError> {
    // 验证注册请求，确保参数符合要求
    let request = request.validate()?;

    // 声明一个长度为 32 字节的数组，初始化为全 0，用于存放即将生成的安全随机盐值
    let mut salt_bytes = [0_u8; 32];
    // 调用操作系统底层的安全随机数生成器（如 Linux 的 /dev/urandom）填充数组，若失败（如系统熵池耗尽）则映射为内部错误
    getrandom::fill(&mut salt_bytes).map_err(|_| AppError::Internal)?;
    // 将 32 字节的原始随机数据编码为符合密码学规范的 Base64 格式盐值字符串，解析失败则抛出内部错误
    let salt = SaltString::encode_b64(&salt_bytes).map_err(|_| AppError::Internal)?;
    // 使用 Argon2 算法对用户的明文密码进行哈希
    let password_hash = Argon2::default()
        .hash_password(request.password.as_bytes(), &salt) // 将明文密码和随机盐传入 Argon2 哈希函数
        .map_err(|_| AppError::Internal)? // 如果底层的哈希计算意外失败（如内存耗尽等），安全地映射为应用层的内部错误
        .to_string(); // 将生成的哈希值转换为字符串，该字符串会自动包含算法参数、盐和哈希结果，便于日后校验

    // 从连接池取得一条连接，并开始数据库事务
    let mut transaction = state.database_pool.begin().await?;
    // 尝试异步将新注册的用户信息插入数据库
    let new_user = match user::insert_user(&mut transaction, &request.email, &password_hash, &request.display_name).await {
        // 插入成功，提取出构建好的新 User 实体
        Ok(user) => user,

        // 边界情况：用户重复，23505 是 PostgreSQL 的标准错误码，代表违反了 Unique 约束
        Err(sqlx::Error::Database(error))
        if error.code().as_deref() == Some("23505") => {
            return Err(AppError::BadRequest("邮箱已经注册"));
        }

        // 异常情况：其他未知的数据库报错，直接向上层抛出
        Err(error) => {
            return Err(AppError::Database(error));
        }
    };

    // 签发 Token，实现注册即登录
    let (access_token, token_hash) = issue_session_token(state.token_hash_secret.as_bytes())?;
    // 持久化当前用户的 Session Token 哈希
    auth::insert_session(&mut transaction, new_user.id, &token_hash, state.token_expire_days).await?;
    // 只有 users 与 auth_sessions 都成功写入时才提交事务，否则在函数返回时自动回滚，保证数据一致性
    transaction.commit().await?;

    // 构造并返回登录成功响应
    Ok(LoginResponse::from_user(new_user, access_token, "Bearer"))
}
