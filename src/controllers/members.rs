use crate::QueryHeader;
use axum::{
    extract::{rejection::JsonRejection, Path, State},
    routing::{delete, get, post, put},
    Json,
};
use serde::{Deserialize, Serialize};
use sqlx::{Pool, Postgres};
use std::sync::Arc;

pub fn members_route(pool: Arc<Pool<Postgres>>) -> axum::Router {
    axum::Router::new()
        .route("/data", get(read_all_members))
        .route("/data/:id", get(get_data))
        .route("/data", post(insert_data))
        .route("/data/:id", put(update_data))
        .route("/data/:id", delete(delete_data))
        .with_state(pool)
}

#[derive(sqlx::Type, Serialize, Deserialize, Clone)]
#[sqlx(type_name = "status_member", rename_all = "lowercase")]
pub enum StatusMember {
    Pekerja,
    #[serde(rename = "ibu rumah tangga")]
    IbuRumahTangga,
    Pelajar,
    Mahasiswa,
    Pengangguran,
}

#[derive(sqlx::Type, Serialize, Deserialize, Clone)]
#[sqlx(type_name = "gender_member", rename_all = "lowercase")]
pub enum GenderMember {
    #[serde(rename = "laki-laki")]
    LakiLaki,
    Perempuan,
}

#[derive(Serialize, Deserialize)]
struct Member {
    nik: i32,
    nama: String,
    umur: i32,
    tanggal_lahir: chrono::NaiveDate,
    tempat_lahir: String,
    status: StatusMember,
    gender: GenderMember,
}

#[derive(Serialize)]
struct ResponseBody {
    status: String,
    message: String,
}

async fn read_all_members(
    State(pool): State<Arc<Pool<Postgres>>>,
    QueryHeader(_): QueryHeader,
) -> Result<Json<Vec<Member>>, axum::http::StatusCode> {
    sqlx::query_as!(
        Member,
        r#"
        SELECT nik, nama, umur, tanggal_lahir, tempat_lahir, 
        status::text AS "status!: StatusMember", 
        gender::text AS "gender!: GenderMember" 
        FROM members LIMIT 10
        "#
    )
    .fetch_all(&*pool)
    .await
    .map(Json)
    .map_err(|err| {
        tracing::error!("{err}");
        axum::http::StatusCode::INTERNAL_SERVER_ERROR
    })
}

async fn get_data(
    State(pool): State<Arc<Pool<Postgres>>>,
    QueryHeader(_): QueryHeader,
    Path(id): Path<u32>,
) -> Result<Json<Member>, axum::http::StatusCode> {
    sqlx::query_as!(
        Member,
        r#"
        SELECT nik, nama, umur, tanggal_lahir, tempat_lahir,
        status::text AS "status!: StatusMember", 
        gender::text AS "gender!: GenderMember" 
        FROM members WHERE id = $1
        "#,
        id as i32
    )
    .fetch_optional(&*pool)
    .await
    .map_err(|err| {
        tracing::error!("{err}");
        axum::http::StatusCode::INTERNAL_SERVER_ERROR
    })?
    .ok_or(axum::http::StatusCode::NOT_FOUND)
    .map(Json)
}

async fn insert_data(
    State(pool): State<Arc<Pool<Postgres>>>,
    QueryHeader(_): QueryHeader,
    input_data: Result<Json<Member>, JsonRejection>,
) -> Result<Json<ResponseBody>, axum::http::StatusCode> {
    let input_data = input_data.map_err(|err| {
        tracing::error!("{err}");
        axum::http::StatusCode::BAD_REQUEST
    })?;
    let result = sqlx::query!(
        "
        INSERT INTO members 
        (nik, nama, umur, tanggal_lahir, tempat_lahir, status, gender) 
        VALUES 
        ($1, $2, $3, $4, $5, $6::status_member, $7::gender_member)
        ",
        input_data.nik,
        input_data.nama,
        input_data.umur,
        input_data.tanggal_lahir,
        input_data.tempat_lahir,
        input_data.status.clone() as StatusMember,
        input_data.gender.clone() as GenderMember
    )
    .execute(&*pool)
    .await
    .map_err(|err| {
        tracing::error!("{err}");
        axum::http::StatusCode::INTERNAL_SERVER_ERROR
    })?;

    if result.rows_affected() == 0 {
        return Err(axum::http::StatusCode::NOT_ACCEPTABLE);
    }

    Ok(Json(ResponseBody {
        status: "success".to_string(),
        message: "Berhasil manambah data".to_string(),
    }))
}

async fn update_data(
    State(pool): State<Arc<Pool<Postgres>>>,
    Path(id): Path<u32>,
    QueryHeader(_): QueryHeader,
    input_data: Result<Json<Member>, JsonRejection>,
) -> Result<Json<ResponseBody>, axum::http::StatusCode> {
    let input_data = input_data.map_err(|_| axum::http::StatusCode::BAD_REQUEST)?;
    let result = sqlx::query!(
        "
        UPDATE members SET 
        nik = $1, nama = $2, umur = $3, tanggal_lahir = $4, 
        tempat_lahir = $5, status = $6::status_member, gender = $7::gender_member
        WHERE id = $8
        ",
        input_data.nik,
        input_data.nama,
        input_data.umur,
        input_data.tanggal_lahir,
        input_data.tempat_lahir,
        input_data.status.clone() as StatusMember,
        input_data.gender.clone() as GenderMember,
        id as i32
    )
    .execute(&*pool)
    .await
    .map_err(|err| {
        tracing::error!("{err}");
        axum::http::StatusCode::INTERNAL_SERVER_ERROR
    })?;

    if result.rows_affected() == 0 {
        return Err(axum::http::StatusCode::NOT_FOUND);
    }

    Ok(Json(ResponseBody {
        status: "success".to_string(),
        message: "Berhasil update data".to_string(),
    }))
}

async fn delete_data(
    State(pool): State<Arc<Pool<Postgres>>>,
    Path(id): Path<u32>,
    QueryHeader(_): QueryHeader,
) -> Result<Json<ResponseBody>, axum::http::StatusCode> {
    let result = sqlx::query!("DELETE FROM members WHERE id = $1", id as i32)
        .execute(&*pool)
        .await
        .map_err(|err| {
            tracing::error!("{err}");
            axum::http::StatusCode::INTERNAL_SERVER_ERROR
        })?;

    if result.rows_affected() == 0 {
        return Err(axum::http::StatusCode::NOT_FOUND);
    }

    Ok(Json(ResponseBody {
        status: "success".to_string(),
        message: "Berhasil menghapus data".to_string(),
    }))
}
