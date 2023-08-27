use std::{path::Path, sync::Arc};

use anyhow::{Context, Result};
use axum::{
    routing::{get, put},
    Router,
};
use tower_http::services::ServeDir;
use tracing::info;
use tracing_subscriber::{prelude::__tracing_subscriber_SubscriberExt, util::SubscriberInitExt};

use crate::{
    interactive::translations::{edit_translation_value, translations, translations_list},
    translation_file::TranslationFile,
};

pub struct AppState {
    pub en_translation_file: TranslationFile,
    pub sv_translation_file: TranslationFile,
}

// https://joeymckenzie.tech/blog/templates-with-rust-axum-htmx-askama/
pub async fn run_server(en_path: &Path, sv_path: &Path) -> Result<()> {
    // let env_filter = EnvFilter::from("info,kobo_sync=debug,tower_http=debug,axum=debug");
    // tracing_subscriber::fmt().with_env_filter(env_filter).init();
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "ramilang=debug".into()),
        )
        .with(tracing_subscriber::fmt::layer())
        .init();

    info!("initializing router...");

    let en_translation_file =
        TranslationFile::new(en_path.to_path_buf()).expect("failed to open en translation file");
    let sv_translation_file =
        TranslationFile::new(sv_path.to_path_buf()).expect("failed to open sv translation file");

    let app_state = Arc::new(AppState {
        en_translation_file,
        sv_translation_file,
    });

    let api_router = Router::new()
        .route("/translations", get(translations_list))
        .route("/translations", put(edit_translation_value))
        .with_state(app_state);

    let assets_path = std::env::current_dir().unwrap();
    let port = 8000_u16;
    let addr = std::net::SocketAddr::from(([0, 0, 0, 0], port));
    let router = Router::new()
        .nest("/api", api_router)
        .route("/", get(translations))
        .nest_service(
            "/assets",
            ServeDir::new(format!("{}/assets", assets_path.to_str().unwrap())),
        );

    info!(
        "router initialized, now listening on http://localhost:{}",
        port
    );

    axum::Server::bind(&addr)
        .serve(router.into_make_service())
        .await
        .context("error while starting server")?;

    Ok(())
}
