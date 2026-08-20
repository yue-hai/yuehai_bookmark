//! 用户相关 service

use argon2::{Argon2, PasswordHasher};
use argon2::password_hash::SaltString;
use rand_core::OsRng;
use crate::app::state::AppState;
use crate::common::error::AppError;
use crate::modules::auth::dto::login::LoginResponse;
use crate::modules::auth::dto::register::RegisterRequest;
use crate::modules::auth::repository::{auth, user};

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

    // 生成一个密码学安全的随机盐值，使用操作系统底层提供的随机数发生器保证足够高的随机熵
    let salt = SaltString::generate(&mut OsRng);
    // 使用 Argon2 算法对用户的明文密码进行哈希
    let password_hash = Argon2::default()
        .hash_password(request.password.as_bytes(), &salt) // 将明文密码和随机盐传入 Argon2 哈希函数
        .map_err(|_| AppError::Internal)? // 如果底层的哈希计算意外失败（如内存耗尽等），安全地映射为应用层的内部错误
        .to_string(); // 将生成的哈希值转换为字符串，该字符串会自动包含算法参数、盐和哈希结果，便于日后校验

    // 尝试异步将新注册的用户信息插入数据库
    let new_user = match user::insert_user(&state.database_url, &request.email, &password_hash, &request.display_name ).await {
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
    let (access_token, token_hash) = crate::modules::auth::service::auth::issue_session_token()?;
    // 持久化当前用户的 Session Token 哈希
    auth::insert_session(&state.database_url, new_user.id, &token_hash ).await?;

    // 构造并返回登录成功响应
    Ok(LoginResponse::from_user(new_user, access_token, "Bearer"))
}
