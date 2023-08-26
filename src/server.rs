use std::sync::{Arc, Mutex};

use anyhow::{Context, Result};
use askama::Template;
use axum::{
    extract::State,
    http::StatusCode,
    response::{Html, IntoResponse, Response},
    routing::{get, post},
    Form, Router,
};
use serde::Deserialize;
use tower_http::services::ServeDir;
use tracing::info;
use tracing_subscriber::{prelude::__tracing_subscriber_SubscriberExt, util::SubscriberInitExt};

struct AppState {
    todos: Mutex<Vec<String>>,
}

// https://joeymckenzie.tech/blog/templates-with-rust-axum-htmx-askama/
pub async fn run_server() -> Result<()> {
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

    let app_state = Arc::new(AppState {
        todos: Mutex::new(vec![]),
    });

    let api_router = Router::new()
        .route("/hello", get(hello_from_the_server))
        .route("/todos", post(add_todo))
        .with_state(app_state);

    let assets_path = std::env::current_dir().unwrap();
    let port = 8000_u16;
    let addr = std::net::SocketAddr::from(([0, 0, 0, 0], port));
    let router = Router::new()
        .nest("/api", api_router)
        .route("/", get(hello))
        .route("/another-page", get(another_page))
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
#[template(path = "components/todo-list.html")]
struct TodoList {
    todos: Vec<String>,
}

#[derive(Deserialize)]
struct TodoRequest {
    todo: String,
}

async fn add_todo(
    State(state): State<Arc<AppState>>,
    Form(todo): Form<TodoRequest>,
) -> impl IntoResponse {
    let mut lock = state.todos.lock().unwrap();
    lock.push(todo.todo);

    let template = TodoList {
        todos: lock.clone(),
    };

    HtmlTemplate(template)
}

async fn hello_from_the_server() -> &'static str {
    "Hello from the sweaty backend!"
}

#[derive(Template)]
#[template(path = "pages/hello.html")]
struct HelloTemplate;

async fn hello() -> impl IntoResponse {
    let template = HelloTemplate {};
    HtmlTemplate(template)
}

#[derive(Template)]
#[template(path = "pages/another-page.html")]
struct AnotherPageTemplate;

async fn another_page() -> impl IntoResponse {
    let template = AnotherPageTemplate {};
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
