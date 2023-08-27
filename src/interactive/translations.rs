use std::{collections::BTreeMap, sync::Arc};

use askama::Template;
use axum::{
    extract::{Query, State},
    http::StatusCode,
    response::IntoResponse,
    Form,
};
use serde::Deserialize;

use super::server::AppState;

struct TranslationRow {
    key: String,
    en: String,
    sv: String,
}

// Translations root page
#[derive(Template)]
#[template(path = "pages/translations.html")]
struct TranslationsTemplate {
    translations: Vec<TranslationRow>,
}

pub async fn translations(State(state): State<Arc<AppState>>) -> impl IntoResponse {
    fn get_translation_row(
        key: &str,
        en: &BTreeMap<String, String>,
        sv: &BTreeMap<String, String>,
    ) -> TranslationRow {
        TranslationRow {
            key: key.to_string(),
            en: en.get(key).unwrap_or(&"".to_string()).to_string(),
            sv: sv.get(key).unwrap_or(&"".to_string()).to_string(),
        }
    }

    let rows = state
        .en_translation_file
        .entries
        .keys()
        .map(|key| {
            get_translation_row(
                key,
                &state.en_translation_file.entries,
                &state.sv_translation_file.entries,
            )
        })
        .collect::<Vec<TranslationRow>>();

    TranslationsTemplate { translations: rows }
}

// Translations list
#[derive(Deserialize)]
pub struct TranslationsSearchQuery {
    query: Option<String>,
}

#[derive(Template)]
#[template(path = "components/translations-list/translations-list.html")]
struct TranslationsList {
    translations: Vec<TranslationRow>,
}

pub async fn translations_list(
    State(state): State<Arc<AppState>>,
    Query(query): Query<TranslationsSearchQuery>,
) -> impl IntoResponse {
    let query = query.query.unwrap_or_default().to_ascii_lowercase();

    fn filter_by_query(
        en_entries: &BTreeMap<String, String>,
        sv_entries: &BTreeMap<String, String>,
        query: &str,
    ) -> Vec<TranslationRow> {
        en_entries
            .iter()
            .filter(|(key, _)| key.to_ascii_lowercase().contains(query))
            .map(|(key, _)| TranslationRow {
                key: key.to_string(),
                en: en_entries.get(key).unwrap_or(&"".to_string()).to_string(),
                sv: sv_entries.get(key).unwrap_or(&"".to_string()).to_string(),
            })
            .collect::<Vec<TranslationRow>>()
    }

    TranslationsList {
        translations: filter_by_query(
            &state.en_translation_file.entries,
            &state.sv_translation_file.entries,
            &query,
        ),
    }
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
