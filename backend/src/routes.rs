use actix_web::{get, post, web, HttpResponse,HttpRequest, Responder, cookie::Cookie};
use sqlx::{Postgres, query_as};
use deadpool_redis;
use uuid::Uuid;

use crate::models::{Credentials, GetUserData, Login, Logout, User};
use crate::utils;


#[post("/create_user")]
pub async fn create_user(
    sql_pool: web::Data<sqlx::Pool<Postgres>>,
    user: web::Json<User>,
) -> impl Responder {

    let result = query_as::<_, Credentials>(
        r#"
        SELECT email, password_hash
        FROM users
        WHERE email = $1
        "#,
    )
    .bind(&user.email)
    .fetch_one(sql_pool.get_ref())
    .await;

    if result.is_ok() {
        return HttpResponse::Conflict().body("User already exists");
    }

    let password_hash = match utils::hash_password(&user.password) {
        Ok(hash) => hash,
        Err(_) => return HttpResponse::InternalServerError().body("Failed to hash password"),
    };

    let result = query_as::<_, User>(
        r#"
        INSERT INTO users (id, first_name, last_name, email, password_hash)
        VALUES ($1, $2, $3, $4, $5)
        RETURNING id, first_name, last_name, email, password_hash
        "#
    )
    .bind(Uuid::new_v4())
    .bind(&user.first_name)
    .bind(&user.last_name)
    .bind(&user.email)
    .bind(&password_hash)
    .fetch_one(sql_pool.get_ref())
    .await;

    match result {
        Ok(user) => HttpResponse::Ok().json(user),
        Err(_) => HttpResponse::InternalServerError().body("Failed to create user"),
    }

}


#[post("/login")]
pub async fn login(
    sql_pool: web::Data<sqlx::Pool<Postgres>>,
    redis_pool: web::Data<deadpool_redis::Pool>,
    login: web::Json<Login>,
) -> impl Responder {

    let password_hash = match utils::hash_password(&login.password) {
        Ok(hash) => hash,
        Err(_) => return HttpResponse::InternalServerError().body("Failed to hash password"),
    };

    let result = query_as::<_, Credentials>(
        r#"
        SELECT id, email, password_hash
        FROM users
        WHERE email = $1
        "#,
    )
    .bind(&login.email)
    .fetch_one(sql_pool.get_ref());

    match result.await {
        Ok(user) => {
            if password_hash == user.password_hash {

                let cookie_lifespan: u64 = 60 * 10;
                let session_id = Uuid::new_v4();
                
                let _ = utils::set_session(
                    &redis_pool,
                    &session_id,
                    &user.id,
                    cookie_lifespan,
                ).await;

                let session_id = session_id.to_string();   
        
                let cookie = Cookie::build(
                    "session_id", 
                    &session_id
                 )
                    .http_only(true)
                    .secure(true)
                    .finish();

                return HttpResponse::Ok()
                    .cookie(cookie)
                    .body("Login successful");
            };
        },
        Err(_) => return HttpResponse::Unauthorized().body("Login failed"),
    }
    return HttpResponse::Unauthorized().body("Login failed");
}



#[post("/logout")]
pub async fn logout(
    redis_pool: web::Data<deadpool_redis::Pool>,
    logout: web::Json<Logout>,
) -> impl Responder {

    let session_id = match Uuid::parse_str(&logout.session_id) {
        Ok(id) => id,
        Err(_) => return HttpResponse::Unauthorized().body("Unauthorized"),
    };
    

    let _ = utils::delete_session(
        &redis_pool,
        &session_id,
    ).await;

    return HttpResponse::Ok().body("Logout route");
}


#[get("/user_data")]
pub async fn get_user_data(
    sql_pool: web::Data<sqlx::Pool<Postgres>>,
    redis_pool: web::Data<deadpool_redis::Pool>,
    request: HttpRequest,
) -> impl Responder {

    let session_cookie = match request.cookie("session_id") {
        Some(cookie) => cookie,
        None => return HttpResponse::Unauthorized().body("No session cookie found"),
    };
    
    let session_id = session_cookie.value();
    let session_id = match Uuid::parse_str(session_id) {
        Ok(id) => id,
        Err(_) => return HttpResponse::Unauthorized().body("Unauthorized"),
    };

    let user_id = match utils::get_user_id(
        &redis_pool,
        &session_id,
    ).await {
        Ok(id) => id,
        Err(_) => return HttpResponse::Unauthorized().body("Unauthorized"),
    };

    let result = query_as::<_, GetUserData>(
        r#"
        SELECT id, first_name, last_name, email
        FROM users
        WHERE id = $1
        "#,
    )
    .bind(&user_id)
    .fetch_one(sql_pool.get_ref())
    .await;

    let user_data = match result {
        Ok(data) => data,
        Err(_) => return HttpResponse::InternalServerError().body("Failed to get user data"),
    };

    let new_session_id = utils::rotate_session(
        &redis_pool,
        &session_id,
        &user_data.id,
        60 * 10,
    ).await;

    let new_session_id = match new_session_id {
        Ok(id) => id,
        Err(_) => return HttpResponse::InternalServerError().body("Failed to rotate session"),
    };

    let cookie = Cookie::build("session_id", new_session_id)
        .http_only(true)
        .secure(true)
        .finish();

    HttpResponse::Ok()
        .cookie(cookie)
        .json(user_data)
}