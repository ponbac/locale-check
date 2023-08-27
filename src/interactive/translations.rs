use std::{collections::BTreeMap, sync::Arc};

use askama::Template;
use axum::{
    extract::{Query, State},
    http::StatusCode,
    response::IntoResponse,
    Form,
};
use serde::Deserialize;

use crate::interactive::HtmlTemplate;

use super::server::AppState;

// Translations root page
#[derive(Template)]
#[template(path = "pages/translations.html")]
struct TranslationsTemplate {
    en: Vec<(String, String)>,
    sv: Vec<(String, String)>,
}

pub async fn translations(State(state): State<Arc<AppState>>) -> impl IntoResponse {
    let en = state
        .en_translation_file
        .entries
        .iter()
        .map(|(key, value)| (key.clone(), value.clone()))
        .collect::<Vec<(String, String)>>();
    let sv = state
        .sv_translation_file
        .entries
        .iter()
        .map(|(key, value)| (key.clone(), value.clone()))
        .collect::<Vec<(String, String)>>();

    HtmlTemplate(TranslationsTemplate { en, sv })
}

// Translations list
#[derive(Deserialize)]
pub struct TranslationsSearchQuery {
    query: Option<String>,
}

#[derive(Template)]
#[template(path = "components/translations-list.html")]
struct TranslationsList {
    en: Vec<(String, String)>,
    sv: Vec<(String, String)>,
}

pub async fn translations_list(
    State(state): State<Arc<AppState>>,
    Query(query): Query<TranslationsSearchQuery>,
) -> impl IntoResponse {
    let query = query.query.unwrap_or_default().to_ascii_lowercase();

    fn filter_by_query(entries: &BTreeMap<String, String>, query: &str) -> Vec<(String, String)> {
        entries
            .iter()
            .filter(|(key, _)| key.to_ascii_lowercase().contains(query))
            .map(|(key, value)| (key.clone(), value.clone()))
            .collect::<Vec<(String, String)>>()
    }

    let en = filter_by_query(&state.en_translation_file.entries, &query);
    let sv = filter_by_query(&state.sv_translation_file.entries, &query);

    let template = TranslationsList { en, sv };
    HtmlTemplate(template)
}

#[derive(Deserialize)]
pub struct TranslationValueEdit {
    key: String,
    value: String,
    language: String,
}

pub async fn edit_translation_value(
    State(state): State<Arc<AppState>>,
    Form(query): Form<TranslationValueEdit>,
) -> impl IntoResponse {
    match query.language.as_str() {
        "en" => {
            let mut en_translation_file = state.en_translation_file.clone();
            en_translation_file.entries.insert(query.key, query.value);
            en_translation_file
                .write()
                .expect("failed to write en translation file");
        }
        "sv" => {
            let mut sv_translation_file = state.sv_translation_file.clone();
            sv_translation_file.entries.insert(query.key, query.value);
            sv_translation_file
                .write()
                .expect("failed to write sv translation file");
        }
        _ => return (StatusCode::BAD_REQUEST, "invalid language"),
    }

    (StatusCode::OK, "ok")
}
