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

    let en_file = state.en_translation_file.lock().unwrap();
    let sv_file = state.sv_translation_file.lock().unwrap();

    let rows = en_file
        .entries
        .keys()
        .map(|key| get_translation_row(key, &en_file.entries, &sv_file.entries))
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

pub async fn search_translations_keys(
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

    let en_file = state.en_translation_file.lock().unwrap();
    let sv_file = state.sv_translation_file.lock().unwrap();

    TranslationsList {
        translations: filter_by_query(&en_file.entries, &sv_file.entries, &query),
    }
}

pub async fn search_translations_values(
    State(state): State<Arc<AppState>>,
    Query(query): Query<TranslationsSearchQuery>,
) -> impl IntoResponse {
    let query = query.query.unwrap_or_default().to_ascii_lowercase();

    fn filter_by_query(
        en_entries: &BTreeMap<String, String>,
        sv_entries: &BTreeMap<String, String>,
        query: &str,
    ) -> Vec<TranslationRow> {
        let combined_rows_iter = en_entries.iter().map(|(key, value)| TranslationRow {
            key: key.to_string(),
            en: value.to_string(),
            sv: sv_entries.get(key).unwrap_or(&"".to_string()).to_string(),
        });

        combined_rows_iter
            .filter(|row| {
                row.en.to_ascii_lowercase().contains(query)
                    || row.sv.to_ascii_lowercase().contains(query)
            })
            .collect()
    }

    let en_file = state.en_translation_file.lock().unwrap();
    let sv_file = state.sv_translation_file.lock().unwrap();

    TranslationsList {
        translations: filter_by_query(&en_file.entries, &sv_file.entries, &query),
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
    let mut en_file = state.en_translation_file.lock().unwrap();
    let mut sv_file = state.sv_translation_file.lock().unwrap();

    match query.language.as_str() {
        "en" => {
            en_file.entries.insert(query.key, query.value);
            en_file
                .write()
                .expect("failed to write en translation file");
        }
        "sv" => {
            sv_file.entries.insert(query.key, query.value);
            sv_file
                .write()
                .expect("failed to write sv translation file");
        }
        _ => return (StatusCode::BAD_REQUEST, "invalid language"),
    }

    (StatusCode::OK, "ok")
}

#[derive(Deserialize)]
pub struct TranslationInsert {
    key: String,
    en: String,
    sv: String,
}

pub async fn insert_translation(
    State(state): State<Arc<AppState>>,
    Form(query): Form<TranslationInsert>,
) -> impl IntoResponse {
    let mut en_translation_file = state.en_translation_file.lock().unwrap();
    let mut sv_translation_file = state.sv_translation_file.lock().unwrap();

    // if en_translation_file.entries.get(&query.key).is_some() {
    //     return (StatusCode::BAD_REQUEST, "key already exists");
    // }

    en_translation_file
        .entries
        .insert(query.key.clone(), query.en.clone());
    en_translation_file
        .write()
        .expect("failed to write en translation file");

    sv_translation_file
        .entries
        .insert(query.key.clone(), query.sv.clone());
    sv_translation_file
        .write()
        .expect("failed to write sv translation file");

    let new_translation_row = TranslationRow {
        key: query.key,
        en: query.en,
        sv: query.sv,
    };

    let mut translations = en_translation_file
        .entries
        .keys()
        .filter(|key| key != &&new_translation_row.key)
        .map(|key| TranslationRow {
            key: key.to_string(),
            en: en_translation_file
                .entries
                .get(key)
                .unwrap_or(&"".to_string())
                .to_string(),
            sv: sv_translation_file
                .entries
                .get(key)
                .unwrap_or(&"".to_string())
                .to_string(),
        })
        .collect::<Vec<TranslationRow>>();

    translations.insert(0, new_translation_row);

    TranslationsList { translations }
}
