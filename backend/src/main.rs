use std::env;
use dotenv::dotenv;
use actix_web::{web, App, HttpServer};
use sqlx::PgPool;
use deadpool_redis::{Config, Runtime};

use routes::{create_user, login, logout, get_user_data};

mod routes;
mod models;
mod utils;


#[actix_web::main]
async fn main() -> std::io::Result<()> {
    dotenv().ok();
    let server_port = env::var("PORT").expect("PORT must be set");
    let server_address = env::var("ADDRESS").expect("ADDRESS must be set");
   
    let user_db = env::var("USER_DB_URL").expect("USER_DB_URL must be set");
    let session_db = env::var("SESSION_DB_URL must be set").expect("SESSION_DB_URL must be set");

    let sql_pool = PgPool::connect(&user_db)
        .await
        .unwrap()
    ;

    let redis_pool = Config::from_url(&session_db)
        .create_pool(Some(Runtime::Tokio1))
        .expect("Failed to create Redis pool");


    HttpServer::new(move || {
        App::new()
            .app_data(web::Data::new(sql_pool.clone()))
            .app_data(web::Data::new(redis_pool.clone()))
            .service(create_user)
            .service(login)
            .service(logout)
            .service(get_user_data)
    })
    .bind(format!("{}:{}",server_address, server_port ))?
    .run()
    .await
}


