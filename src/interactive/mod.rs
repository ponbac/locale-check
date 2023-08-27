mod server;
mod translations;

use askama::Template;
use axum::{
    http::StatusCode,
    response::{Html, IntoResponse, Response},
};
pub use server::run_server;
