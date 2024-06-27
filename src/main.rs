use dashboard::controllers::{auth::authentication, members::members_route};
use std::sync::Arc;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    dotenvy::dotenv()?;
    tracing::subscriber::set_global_default(tracing_subscriber::FmtSubscriber::default())?;

    let database_url = std::env::var("DATABASE_URL").expect("DATABASE URL TIDAK DI SET!!");
    let pool = Arc::new(sqlx::postgres::PgPool::connect(&database_url).await?);
    sqlx::migrate!("./migrations").run(&*pool).await?;

    let app = axum::Router::new()
        .merge(authentication(pool.clone()))
        .merge(members_route(pool));

    let tcp_addr = tokio::net::TcpListener::bind("127.0.0.1:3000").await?;
    tracing::info!("Server berjalan pada http://127.0.0.1:3000");
    axum::serve(tcp_addr, app.into_make_service()).await?;

    Ok(())
}
