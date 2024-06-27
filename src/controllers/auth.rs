use crate::{jwt::Claims, Email};
use argon2::{
    password_hash::{rand_core::OsRng, SaltString},
    Argon2, PasswordHash, PasswordHasher, PasswordVerifier,
};
use axum::{
    extract::{rejection::JsonRejection, State},
    routing::post,
    Json,
};
use serde::{Deserialize, Serialize};
use sqlx::{Pool, Postgres};
use std::sync::Arc;

pub fn authentication(pool: Arc<Pool<Postgres>>) -> axum::Router {
    axum::Router::new()
        .route("/login", post(login_handler))
        .route("/register", post(register_handler))
        .with_state(pool)
}

#[derive(Deserialize)]
struct LoginSchema {
    email: Email,
    password: String,
}

#[derive(Serialize)]
struct ResponseSchema {
    status: String,
    message: String,
    content: Content,
}

#[derive(Serialize)]
struct Content {
    email: String,
    token: String,
    expired: usize,
}

async fn login_handler(
    State(pool): State<Arc<Pool<Postgres>>>,
    input_data: Result<Json<LoginSchema>, JsonRejection>,
) -> Result<Json<ResponseSchema>, axum::http::StatusCode> {
    let input_data = input_data.map_err(|_| axum::http::StatusCode::BAD_REQUEST)?;
    let user_data = sqlx::query!(
        "SELECT password FROM users WHERE email = $1",
        input_data.email.as_str()
    )
    .fetch_optional(&*pool)
    .await
    .map_err(|err| {
        tracing::error!("{err}");
        axum::http::StatusCode::INTERNAL_SERVER_ERROR
    })?
    .ok_or(axum::http::StatusCode::NOT_FOUND)?;

    let parsed_hash = PasswordHash::new(&user_data.password).map_err(|err| {
        tracing::error!("{err}");
        axum::http::StatusCode::INTERNAL_SERVER_ERROR
    })?;

    if let Err(val) =
        Argon2::default().verify_password(input_data.password.as_bytes(), &parsed_hash)
    {
        tracing::error!("{val}");
        return Err(axum::http::StatusCode::BAD_REQUEST);
    }

    let token = Claims::new(input_data.email.as_str().to_string()).to_token()?;
    Ok(Json(ResponseSchema {
        status: "success".to_string(),
        message: "Berhasil login".to_string(),
        content: Content {
            email: input_data.email.as_str().to_string(),
            token,
            expired: 60,
        },
    }))
}

async fn register_handler(
    State(pool): State<Arc<Pool<Postgres>>>,
    input_data: Result<Json<LoginSchema>, JsonRejection>,
) -> Result<Json<ResponseSchema>, axum::http::StatusCode> {
    let input_data = input_data.map_err(|_| axum::http::StatusCode::BAD_REQUEST)?;
    let salt = SaltString::generate(&mut OsRng);
    let hash_password = Argon2::default()
        .hash_password(input_data.password.as_bytes(), &salt)
        .map_err(|err| {
            tracing::error!("{err}");
            axum::http::StatusCode::INTERNAL_SERVER_ERROR
        })?
        .to_string();

    let result = sqlx::query!(
        "INSERT INTO users (email, password) VALUES ($1, $2)",
        input_data.email.as_str(),
        hash_password
    )
    .execute(&*pool)
    .await
    .map_err(|err| match err {
        sqlx::Error::Database(err) if err.code().is_some() && err.code().unwrap() == "23505" => {
            tracing::error!("{err}");
            axum::http::StatusCode::CONFLICT
        }
        _ => {
            tracing::error!("{err}");
            axum::http::StatusCode::INTERNAL_SERVER_ERROR
        }
    })?;

    if result.rows_affected() == 0 {
        return Err(axum::http::StatusCode::NOT_ACCEPTABLE);
    }

    Ok(Json(ResponseSchema {
        status: "success".to_string(),
        message: "Berhasil membuat akun".to_string(),
        content: Content {
            email: input_data.email.as_str().to_string(),
            token: "".to_string(),
            expired: 0,
        },
    }))
}
