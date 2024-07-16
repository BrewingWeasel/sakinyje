// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use crate::{
    add_to_anki::add_to_anki,
    ankiconnect::{get_all_deck_names, get_all_note_names, get_note_field_names, remove_deck},
    dictionary::{get_defs, DictionaryInfo},
    language_parsing::{parse_text, start_stanza},
    new_language_template::new_language_from_template,
};
use ankiconnect::get_anki_card_statuses;
use chrono::{DateTime, TimeDelta, Utc};
use commands::run_command;
use serde::{Deserialize, Serialize};
use shared::{LanguageSettings, SakinyjeResult, Settings, ToasterPayload};
use simple_logger::SimpleLogger;
use spyglys_integration::{format_spyglys, get_spyglys_functions};
use stats::{get_words_known_at_levels, time_spent};
use std::{collections::HashMap, fs, io::BufReader, process, sync::Arc, time::Duration};
use tauri::{async_runtime::block_on, GlobalWindowEvent, Manager, State, Window, WindowEvent};

mod add_to_anki;
mod ankiconnect;
mod commands;
mod dictionary;
mod language_parsing;
mod new_language_template;
mod spyglys_integration;
mod stats;

#[derive(Debug, thiserror::Error)]
enum SakinyjeError {
    #[error(transparent)]
    Io(#[from] std::io::Error),
    #[error(transparent)]
    TomlSer(#[from] toml::ser::Error),
    #[error(transparent)]
    TomlDe(#[from] toml::de::Error),
    #[error(transparent)]
    Tauri(#[from] tauri::Error),
    #[error(transparent)]
    Spyglys(#[from] spyglys::SpyglysError),
    #[error(transparent)]
    SpyglysRuntime(#[from] spyglys::interpreter::RuntimeError),
    #[error("No operating system {0} directory was found")]
    MissingDir(String),
    #[error("Anki is not available. This may be because it is not open or ankiconnect is not installed.")]
    AnkiNotAvailable,
    #[error("Unable to download language details from github: {0}")]
    LanugageDetailsDownloading(#[from] reqwest::Error),
    #[error("The selected card has handler that fits its model")]
    NoModelHandler,
    #[error("Ankiconnect return an error: {0}")]
    AnkiConnectError(String),
}

// we must manually implement serde::Serialize
impl serde::Serialize for SakinyjeError {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::ser::Serializer,
    {
        serializer.serialize_str(self.to_string().as_ref())
    }
}

struct SakinyjeState(tauri::async_runtime::Mutex<SharedInfo>);

struct SharedInfo {
    settings: Settings,
    to_save: ToSave,
    language_parser: Option<LanguageParser>,
    current_language: Option<String>,
    dict_info: Arc<tauri::async_runtime::Mutex<DictionaryInfo>>,
    errors: Vec<SakinyjeError>,
    can_save: bool,
}

struct LanguageParser {
    stdin: process::ChildStdin,
    stdout: BufReader<process::ChildStdout>,
}

#[derive(Serialize, Deserialize, Default)]
struct ToSave {
    last_launched: DateTime<Utc>,
    last_language: Option<String>,
    decks_checked: Vec<String>,
    language_specific: HashMap<String, LanguageSpecficToSave>,
    sessions: Vec<(DateTime<Utc>, Duration)>,
}

#[derive(Serialize, Deserialize, Default)]
struct LanguageSpecficToSave {
    words: HashMap<String, WordInfo>,
    cached_defs: HashMap<String, Vec<SakinyjeResult<String>>>,
    previous_file: Option<String>,
    previous_amount: usize,
    words_seen: Vec<(DateTime<Utc>, usize)>,
}

impl Default for SharedInfo {
    fn default() -> Self {
        let mut errors = Vec::new();

        let mut to_save: ToSave = match dirs::data_dir()
            .ok_or_else(|| SakinyjeError::MissingDir(String::from("data")))
            .and_then(|saved_state_file| {
                fs::read_to_string(saved_state_file.join("sakinyje_saved_data.toml"))
                    .map_err(SakinyjeError::Io)
                    .and_then(|v| toml::from_str(&v).map_err(SakinyjeError::TomlDe))
            }) {
            Ok(v) => v,
            Err(e) => {
                if !matches!(e, SakinyjeError::Io(_)) {
                    errors.push(e);
                }
                ToSave::default()
            }
        };

        let settings: Settings = match dirs::config_dir()
            .ok_or_else(|| SakinyjeError::MissingDir(String::from("config")))
            .and_then(|config_file| {
                fs::read_to_string(config_file.join("sakinyje.toml"))
                    .map_err(SakinyjeError::Io)
                    .and_then(|v| toml::from_str(&v).map_err(SakinyjeError::TomlDe))
            }) {
            Ok(v) => v,
            Err(e) => {
                if !matches!(e, SakinyjeError::Io(_)) {
                    errors.push(e);
                }
                Settings::default()
            }
        };

        set_word_knowledge_from_anki(&mut to_save, &settings.languages);

        if let Some(cmds) = &settings.to_run {
            for cmd in cmds {
                _ = run_command(cmd);
            }
        }

        let current_language = to_save
            .last_language
            .clone()
            .or_else(|| settings.languages.keys().next().cloned());
        to_save.sessions.push((Utc::now(), Duration::new(0, 0)));

        let can_save = errors.is_empty();

        Self {
            errors,
            to_save,
            settings,
            language_parser: None,
            current_language,
            dict_info: Default::default(),
            can_save,
        }
    }
}

#[derive(Serialize, Deserialize)]
struct WordInfo {
    rating: i8,
    method: Method,
    history: Vec<(DateTime<Utc>, Method, i8)>,
}

#[derive(Serialize, Deserialize, PartialEq, Eq, Clone, Copy)]
enum Method {
    FromAnki,
    FromSeen,
    Specified,
    FromFrequency,
}

fn main() {
    SimpleLogger::new()
        .with_colors(true)
        .with_level(log::LevelFilter::Info)
        .env()
        .init()
        .unwrap();
    tauri::Builder::default()
        .manage(SakinyjeState(Default::default()))
        .invoke_handler(tauri::generate_handler![
            parse_text,
            get_defs,
            get_settings,
            get_dark_mode,
            add_to_anki,
            write_settings,
            get_all_deck_names,
            get_all_note_names,
            get_note_field_names,
            update_word_knowledge,
            remove_deck,
            get_rating,
            get_language,
            set_language,
            get_languages,
            new_language_from_template,
            start_stanza,
            refresh_anki,
            format_spyglys,
            get_spyglys_functions,
            time_spent,
            get_words_known_at_levels,
            check_startup_errors,
        ])
        .on_window_event(handle_window_event)
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

#[tauri::command]
async fn refresh_anki(state: State<'_, SakinyjeState>, window: Window) -> Result<(), String> {
    window
        .emit(
            "refresh_anki",
            ToasterPayload {
                message: Some("Loading anki data"),
            },
        )
        .unwrap();
    let mut state = state.0.lock().await;
    let languages = state.settings.languages.clone();

    set_word_knowledge_from_anki(&mut state.to_save, &languages);
    window
        .emit("refresh_anki", ToasterPayload { message: None })
        .unwrap();
    Ok(())
}

#[tauri::command]
async fn check_startup_errors(state: State<'_, SakinyjeState>) -> Result<(), Vec<SakinyjeError>> {
    let mut state = state.0.lock().await;
    let errs = std::mem::take(&mut state.errors);
    if errs.is_empty() {
        Ok(())
    } else {
        Err(errs)
    }
}

fn set_word_knowledge_from_anki(
    to_save: &mut ToSave,
    languages: &HashMap<String, LanguageSettings>,
) {
    let new_time = Utc::now();
    let days_passed = new_time
        .signed_duration_since(to_save.last_launched)
        .num_days()
        + 2;

    for (language_name, language) in languages {
        let to_save_language = to_save
            .language_specific
            .entry(language_name.to_owned())
            .or_default();
        for (deck, note_parser) in &language.anki_parser {
            block_on(get_anki_card_statuses(
                deck,
                &note_parser.0,
                &mut to_save_language.words,
                days_passed,
                // If the deck has not been added, it means this is the first time it is being
                // checked, so we should check every card and not just the ones recently
                // updated
                !to_save.decks_checked.contains(deck),
            ))
            .unwrap();
            to_save.decks_checked.push(deck.to_owned());
        }

        if Some(&language.frequency_list) != to_save_language.previous_file.as_ref()
            || language.words_known_by_freq != to_save_language.previous_amount
        {
            to_save_language.previous_file = Some(language.frequency_list.clone());
            to_save_language.previous_amount = language.words_known_by_freq;
            update_words_known(
                &language.frequency_list,
                language.words_known_by_freq,
                &mut to_save_language.words,
            );
        }
    }
    to_save.last_launched = new_time;
}

fn handle_window_event(event: GlobalWindowEvent) {
    block_on(async move {
        #[allow(clippy::single_match)] // Will probably be expanded in the future
        match event.event() {
            &WindowEvent::Destroyed => {
                let state: State<'_, SakinyjeState> = event.window().state();
                let mut locked_state = state.0.lock().await;
                if locked_state.can_save {
                    log::info!("saving details");
                    let saved_state_file =
                        dirs::data_dir().unwrap().join("sakinyje_saved_data.toml");
                    locked_state.to_save.last_language = locked_state.current_language.clone();
                    let session = locked_state
                        .to_save
                        .sessions
                        .last_mut()
                        .expect("sessions should exist");
                    session.1 =
                        TimeDelta::to_std(&(Utc::now() - session.0)).expect("time should be valid");
                    let conts =
                        toml::to_string(&locked_state.to_save).expect("Error serializing to toml");
                    fs::write(saved_state_file, conts).expect("error writing to file");
                }
            }
            _ => (),
        }
    })
}

fn update_words_known(
    file_name: &str,
    words_known: usize,
    original_words: &mut HashMap<String, WordInfo>,
) {
    log::info!("updating words known");
    if let Ok(contents) = fs::read_to_string(file_name) {
        original_words.retain(|_, v| v.method != Method::FromFrequency);
        for word in contents.lines().take(words_known) {
            log::trace!("Checking word: {}", word);
            original_words.insert(
                word.to_owned(),
                WordInfo {
                    rating: 5,
                    method: Method::FromFrequency,
                    history: vec![(Utc::now(), Method::FromFrequency, 5)],
                },
            );
        }
    }
}

#[tauri::command]
async fn get_settings(state: State<'_, SakinyjeState>) -> Result<Settings, String> {
    log::trace!("Settings requested");
    let state = state.0.lock().await;
    Ok(state.settings.clone())
}

#[tauri::command]
async fn get_dark_mode(state: State<'_, SakinyjeState>) -> Result<bool, String> {
    let state = state.0.lock().await;
    Ok(state.settings.dark_mode)
}

#[tauri::command]
async fn get_rating(lemma: String, state: State<'_, SakinyjeState>) -> Result<i8, String> {
    log::trace!("Getting rating for word: {lemma}");
    let mut state = state.0.lock().await;
    let language = state
        .current_language
        .clone()
        .expect("need a language selected to be able to set rating");
    Ok(state
        .to_save
        .language_specific
        .get_mut(&language)
        .expect("language should exist")
        .words
        .entry(lemma)
        .or_insert(WordInfo {
            rating: 0,
            method: Method::FromSeen,
            history: vec![(Utc::now(), Method::FromSeen, 0)],
        })
        .rating)
}

#[tauri::command]
async fn get_languages(state: State<'_, SakinyjeState>) -> Result<Vec<String>, String> {
    let state = state.0.lock().await;
    Ok(state.settings.languages.keys().cloned().collect())
}

#[tauri::command]
async fn get_language(state: State<'_, SakinyjeState>) -> Result<Option<String>, String> {
    let state = state.0.lock().await;
    Ok(state.current_language.to_owned())
}

#[tauri::command]
async fn set_language(state: State<'_, SakinyjeState>, language: String) -> Result<(), String> {
    let mut state = state.0.lock().await;
    state.current_language = Some(language);
    Ok(())
}

#[tauri::command]
async fn write_settings(
    state: State<'_, SakinyjeState>,
    settings: Settings,
) -> Result<(), SakinyjeError> {
    let config_file = dirs::config_dir()
        .ok_or(SakinyjeError::MissingDir("config".to_string()))?
        .join("sakinyje.toml");
    let conts = toml::to_string_pretty(&settings)?;

    let mut state = state.0.lock().await;
    state.settings = settings;

    fs::write(config_file, conts)?;
    Ok(())
}

#[tauri::command]
async fn update_word_knowledge(
    state: State<'_, SakinyjeState>,
    word: &str,
    rating: i8,
    modifiable: bool,
) -> Result<(), String> {
    log::info!("Word {word} rating set to {rating}");
    let mut state = state.0.lock().await;
    let language = state
        .current_language
        .clone()
        .expect("current language should already be chosen");
    let word_knowledge = state
        .to_save
        .language_specific
        .get_mut(&language)
        .expect("current language should already have content to save")
        .words
        .get_mut(word)
        .unwrap();

    let method = if modifiable {
        Method::FromAnki
    } else {
        Method::Specified
    };

    word_knowledge.history.push((Utc::now(), method, rating));
    word_knowledge.rating = rating;
    word_knowledge.method = method;
    Ok(())
}
