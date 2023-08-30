use std::{
    path::Path,
    sync::{Arc, Mutex},
};

use anyhow::{Context, Result};
use axum::{
    http::{HeaderMap, HeaderValue, StatusCode},
    response::IntoResponse,
    routing::{get, post, put},
    Router,
};
use console::style;
use tracing_subscriber::{prelude::__tracing_subscriber_SubscriberExt, util::SubscriberInitExt};

use crate::{
    interactive::translations::{
        edit_translation_value, insert_translation, search_translations_keys,
        search_translations_values, translations,
    },
    translation_file::TranslationFile,
};

pub struct AppState {
    pub en_translation_file: Mutex<TranslationFile>,
    pub sv_translation_file: Mutex<TranslationFile>,
}

static HTMX_FILE: &str = include_str!("../../assets/scripts/htmx_1.9.4.js");
static CSS_FILE: &str = include_str!("../../assets/main.css");
static FAV_ICON: &[u8] = include_bytes!("../../assets/favicon.ico");

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

    let en_translation_file =
        TranslationFile::new(en_path.to_path_buf()).expect("failed to open en translation file");
    let sv_translation_file =
        TranslationFile::new(sv_path.to_path_buf()).expect("failed to open sv translation file");

    let app_state = Arc::new(AppState {
        en_translation_file: Mutex::new(en_translation_file),
        sv_translation_file: Mutex::new(sv_translation_file),
    });

    let port = 3333_u16;
    let addr = std::net::SocketAddr::from(([0, 0, 0, 0], port));
    let router = Router::new()
        .route("/", get(translations))
        .route("/api/search-keys", get(search_translations_keys))
        .route("/api/search-values", get(search_translations_values))
        .route("/api/translations", put(edit_translation_value))
        .route("/api/translations", post(insert_translation))
        .route("/assets/htmx.js", get(get_htmx_js))
        .route("/assets/main.css", get(get_css))
        .route("/favicon.ico", get(get_favicon))
        .with_state(app_state);

    //clear console
    print!("\x1B[2J\x1B[1;1H");

    println!("{}", style("Server is now running!").green().bold());
    println!(
        "🚀 {}{} 🚀",
        style("Started at ").green().italic(),
        style(format!("http://localhost:{}", port))
            .yellow()
            .bold()
            .italic()
            .underlined()
    );

    // open browser if release build
    #[cfg(not(debug_assertions))]
    webbrowser::open(&format!("http://localhost:{}", port)).context("failed to open browser")?;

    axum::Server::bind(&addr)
        .serve(router.into_make_service())
        .await
        .context("error while starting server")?;

    Ok(())
}

async fn get_htmx_js() -> impl IntoResponse {
    let mut headers = HeaderMap::new();
    headers.insert("Content-Type", HeaderValue::from_static("text/javascript"));

    (StatusCode::OK, headers, HTMX_FILE)
}

async fn get_css() -> impl IntoResponse {
    let mut headers = HeaderMap::new();
    headers.insert("Content-Type", HeaderValue::from_static("text/css"));

    (StatusCode::OK, headers, CSS_FILE)
}

async fn get_favicon() -> impl IntoResponse {
    FAV_ICON
}
