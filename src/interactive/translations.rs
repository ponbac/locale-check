use std::{collections::BTreeMap, sync::Arc};

use askama::Template;
use axum::{
    extract::{Query, State},
    response::IntoResponse,
};
use serde::Deserialize;

use crate::interactive::HtmlTemplate;

use super::server::AppState;

#[derive(Template)]
#[template(path = "pages/translations.html")]
struct TranslationsTemplate {}

pub async fn translations() -> impl IntoResponse {
    HtmlTemplate(TranslationsTemplate {})
}

#[derive(Template)]
#[template(path = "components/translations-list.html")]
struct TranslationsList {
    en: Vec<(String, String)>,
    sv: Vec<(String, String)>,
}

#[derive(Deserialize)]
pub struct TranslationsQuery {
    query: Option<String>,
}

pub async fn translations_list(
    State(state): State<Arc<AppState>>,
    Query(query): Query<TranslationsQuery>,
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
