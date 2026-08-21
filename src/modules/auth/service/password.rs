//! 用户密码哈希和验证

use argon2::password_hash::SaltString;
use argon2::{Argon2, PasswordHash, PasswordHasher, PasswordVerifier};

use crate::common::error::AppError;

/// 在 Tokio 阻塞线程池中使用 Argon2 哈希密码
///
/// Argon2 是 CPU 和内存密集型算法，不能长时间占用 Tokio 的异步工作线程
/// 因此使用 spawn_blocking 将同步计算交给专用阻塞线程池
/// 
/// # Arguments
/// * `password`：客户端明文密码
/// 
/// # Returns
/// * `Result<String, AppError>`：成功时返回 PHC 格式的密码哈希，失败时返回应用错误
pub async fn hash_password(password: String) -> Result<String, AppError> {
    // 使用 spawn_blocking 将密码哈希计算放入阻塞线程池，避免阻塞异步工作线程
    tokio::task::spawn_blocking(move || {
        // 声明 32 字节数组，用于生成每个密码独立的随机盐
        let mut salt_bytes = [0_u8; 32];
        // 使用操作系统安全随机源填充密码盐
        getrandom::fill(&mut salt_bytes).map_err(|_| AppError::Internal)?;

        // 将随机字节转换为 Argon2 所需的 Base64 盐值
        let salt = SaltString::encode_b64(&salt_bytes).map_err(|_| AppError::Internal)?;

        // 使用 Argon2 算法对用户的明文密码进行哈希
        Argon2::default()
            .hash_password(password.as_bytes(), &salt)  // 将明文密码和随机盐传入 Argon2 哈希函数
            .map_err(|_| AppError::Internal) // 如果底层的哈希计算意外失败（如内存耗尽等），安全地映射为应用层的内部错误
            .map(|password_hash| password_hash.to_string()) // 将生成的哈希值转换为字符串，该字符串会自动包含算法参数、盐和哈希结果，便于日后校验
    })
        
        .await // 等待阻塞线程池完成任务
        .map_err(|_| AppError::Internal)? // 如果阻塞线程发生 panic 或线程池无法完成任务，映射为内部错误
}

/// 在 Tokio 阻塞线程池中验证密码，密码错误统一返回 InvalidCredentials，避免泄露更多认证信息
/// 
/// # Arguments
/// * `password`：客户端明文密码
/// * `stored_password_hash`：数据库中存储的 PHC 格式密码哈希
/// 
/// # Returns
/// * `Result<(), AppError>`：成功时返回 Ok，失败时返回应用错误
pub async fn verify_password(password: String, stored_password_hash: String, ) -> Result<(), AppError> {
    // 使用 spawn_blocking 将密码验证计算放入阻塞线程池，避免阻塞异步工作线程
    tokio::task::spawn_blocking(move || {
        // 解析数据库保存的 PHC 格式密码哈希
        let parsed_hash = PasswordHash::new(&stored_password_hash).map_err(|_| AppError::Internal)?;

        // 创建默认的 Argon2id 密码校验器，使用客户端明文密码与数据库哈希进行安全比对，密码不匹配时统一返回凭据错误
        Argon2::default().verify_password(password.as_bytes(), &parsed_hash).map_err(|_| AppError::InvalidCredentials)
    })
        .await  // 等待阻塞线程池完成任务
        .map_err(|_| AppError::Internal)? // 如果阻塞线程发生 panic 或线程池无法完成任务，映射为内部错误
}