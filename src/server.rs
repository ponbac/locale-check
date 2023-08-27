use std::{path::Path, sync::Arc};

use anyhow::{Context, Result};
use askama::Template;
use axum::{
    extract::{Query, State},
    http::StatusCode,
    response::{Html, IntoResponse, Response},
    routing::get,
    Router,
};
use serde::Deserialize;
use tower_http::services::ServeDir;
use tracing::info;
use tracing_subscriber::{prelude::__tracing_subscriber_SubscriberExt, util::SubscriberInitExt};

use crate::translation_file::TranslationFile;

struct AppState {
    en_translation_file: TranslationFile,
    sv_translation_file: TranslationFile,
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

#[derive(Template)]
#[template(path = "pages/translations.html")]
struct TranslationsTemplate {}

async fn translations() -> impl IntoResponse {
    let template = TranslationsTemplate {};

    HtmlTemplate(template)
}

#[derive(Template)]
#[template(path = "components/translations-list.html")]
struct TranslationsList {
    en: Vec<(String, String)>,
    sv: Vec<(String, String)>,
}

#[derive(Deserialize)]
struct TranslationsQuery {
    query: Option<String>,
}

async fn translations_list(
    State(state): State<Arc<AppState>>,
    Query(query): Query<TranslationsQuery>,
) -> impl IntoResponse {
    let query = query.query.unwrap_or_default();

    let mut en = state
        .en_translation_file
        .entries
        .iter()
        .map(|(key, value)| (key.clone(), value.clone()))
        .collect::<Vec<(String, String)>>();
    en.sort_by(|(a, _), (b, _)| a.cmp(b));
    en = en
        .into_iter()
        .filter(|(key, _)| key.contains(&query))
        .collect::<Vec<(String, String)>>();

    let mut sv = state
        .sv_translation_file
        .entries
        .iter()
        .map(|(key, value)| (key.clone(), value.clone()))
        .collect::<Vec<(String, String)>>();
    sv.sort_by(|(a, _), (b, _)| a.cmp(b));
    sv = sv
        .into_iter()
        .filter(|(key, _)| key.contains(&query))
        .collect::<Vec<(String, String)>>();

    let template = TranslationsList { en, sv };

    HtmlTemplate(template)
}

/// A wrapper type that we'll use to encapsulate HTML parsed by askama into valid HTML for axum to serve.
struct HtmlTemplate<T>(T);

/// Allows us to convert Askama HTML templates into valid HTML for axum to serve in the response.
impl<T> IntoResponse for HtmlTemplate<T>
where
    T: Template,
{
    fn into_response(self) -> Response {
        // Attempt to render the template with askama
        match self.0.render() {
            // If we're able to successfully parse and aggregate the template, serve it
            Ok(html) => Html(html).into_response(),
            // If we're not, return an error or some bit of fallback HTML
            Err(err) => (
                StatusCode::INTERNAL_SERVER_ERROR,
                format!("Failed to render template. Error: {}", err),
            )
                .into_response(),
        }
    }
}
