use std::collections::HashMap;

use serde::{Deserialize, Serialize};

#[derive(Clone, Serialize, Deserialize, Debug)]
pub struct Word {
    pub text: String,
    pub lemma: String,
    pub rating: u8,
    pub morph: HashMap<String, String>,
    pub clickable: bool,
}

#[derive(Serialize, Deserialize, PartialEq, Clone, Eq, Debug)]
pub enum DictFileType {
    TextSplitAt(String),
    StarDict,
}

#[derive(Serialize, Deserialize, PartialEq, Eq, Clone, Debug)]
#[serde(tag = "t", content = "c")]
pub enum Dictionary {
    File(String, DictFileType),
    Url(String),
    Command(String),
    Wiktionary(String, bool),
}

#[derive(Serialize, Deserialize, PartialEq, Eq, Clone, Debug)]
#[serde(tag = "t", content = "conts")]
pub enum SakinyjeResult<T> {
    Ok(T),
    Err(String),
}

impl<T, E> From<Result<T, E>> for SakinyjeResult<T>
where
    E: ToString,
{
    fn from(value: Result<T, E>) -> Self {
        match value {
            Ok(v) => Self::Ok(v),
            Err(e) => Self::Err(e.to_string()),
        }
    }
}

impl<T> From<SakinyjeResult<T>> for Result<T, String> {
    fn from(value: SakinyjeResult<T>) -> Self {
        match value {
            SakinyjeResult::Ok(v) => Self::Ok(v),
            SakinyjeResult::Err(e) => Self::Err(e),
        }
    }
}

#[derive(Deserialize, Serialize, Clone)]
pub struct NoteToWordHandling {
    pub field_to_use: String,
    pub remove_everything_in_parens: bool,
    pub only_first_word_or_line: bool,
}

#[derive(Deserialize, Serialize, Clone)]
pub struct Settings {
    pub deck: String,
    pub note_type: String,
    pub note_fields: String,
    pub model: String,
    pub dicts: Vec<Dictionary>,
    pub to_remove: Option<usize>,
    pub css: Option<String>,
    pub anki_parser: Option<HashMap<String, HashMap<String, NoteToWordHandling>>>,
    pub to_run: Option<Vec<String>>,
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            deck: String::from("Default"),
            note_type: String::from("Basic"),
            note_fields: String::from(
                "Front:{sent}
Back:{word}:{def}",
            ),
            model: String::new(),
            dicts: Vec::new(),
            to_remove: None,
            css: None,
            anki_parser: None,
            to_run: None,
        }
    }
}
