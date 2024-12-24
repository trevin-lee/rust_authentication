use argon2::{Argon2, PasswordHasher, password_hash::SaltString};
use rand::thread_rng;
use deadpool_redis::{redis::AsyncCommands, Pool, Connection};
use redis::{RedisResult, RedisError, ErrorKind::IoError};
use uuid::Uuid;


pub fn hash_password(password: &str) -> Result<String, argon2::password_hash::Error> {
    let salt = SaltString::generate(&mut thread_rng());
    let argon2 = Argon2::default();
    let hashed = argon2
        .hash_password(password.as_bytes(), &salt)?
        .to_string();
    return Ok(hashed);
}

pub async fn set_session(
    redis_pool: &Pool,
    session_id: &Uuid,
    user_id: &Uuid,
    expires_in: u64,
) -> RedisResult<()> {
    let mut conn: Connection = redis_pool.get()
        .await
        .map_err(
            | e| 
            RedisError::from((
                IoError, 
                "Failed to get connection from pool", 
                e.to_string()
            )))?;
        conn.set_ex::<&str, &str, ()>(
            &session_id.to_string(), 
            &user_id.to_string(), 
            expires_in
        )
        .await?;
    Ok(())
}   

pub async fn get_user_id(
    redis_pool: &Pool,
    session_id: &Uuid
) -> RedisResult<Option<String>> {
    let mut conn: Connection = redis_pool.get()
        .await
        .map_err(
            | e| 
            RedisError::from((
                IoError, 
                "Failed to get connection from pool", 
                e.to_string()
            )))?;
    let val = conn.get::<&str, Option<String>>(&session_id.to_string())
        .await?;
    Ok(val)
}

pub async fn delete_session(
    redis_pool: &Pool,
    session_id: &Uuid,
) -> RedisResult<()> {
    let mut conn: Connection = redis_pool.get()
        .await
        .map_err(
            | e| 
            RedisError::from((
                IoError, 
                "Failed to get connection from pool", 
                e.to_string()
            )))?;
    let _ = conn.del::<&str, ()>(&session_id.to_string()).await?;
    Ok(())
}


pub async fn rotate_session(
    redis_pool: &Pool,
    session_id: &Uuid,
    user_id: &Uuid,
    expires_in: u64,
) -> RedisResult<String> {
    let mut conn: Connection = redis_pool.get()
        .await
        .map_err(|e| 
            RedisError::from((
                IoError, 
                "Failed to get connection from pool", 
                e.to_string()
            ))
        )?;

    let _: () = conn.del(&session_id.to_string()).await?;
    let new_session_id = uuid::Uuid::new_v4().to_string();

    let _: () = conn.set_ex(
        &new_session_id, 
        &user_id.to_string(), 
        expires_in
    )
    .await?;

    Ok(new_session_id)
}