#[derive(Clone)]
pub struct AppState {
    pub db: sqlx::PgPool,
}

impl AppState {
    async fn new() -> color_eyre::Result<Self> {
        let database_url = std::env::var("DATABASE_URL")?;
        let db = sqlx::PgPool::connect(&database_url).await?;
        Ok(Self { db })
    }
}

pub struct App;

impl App {
    pub async fn start() -> color_eyre::Result<()> {
        tracing_subscriber::fmt::init();
        dotenvy::dotenv().ok();
        let state = AppState::new().await?;
        let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await?;
        use tower_http::services::ServeDir;

        let router = axum::Router::new()
            .nest_service("/assets", ServeDir::new("src/assets"))
            .nest("/api", crate::routes::api::router())
            .merge(crate::routes::frontend::router())
            .with_state(state);
        tracing::info!("Servidor iniciado em http://0.0.0.0:3000");
        axum::serve(listener, router).await?;
        Ok(())
    }
}
