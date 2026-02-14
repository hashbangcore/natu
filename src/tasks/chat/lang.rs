pub use crate::utils::normalize_lang_tag;

/// Maps a language tag to a readable English name.
pub fn lang_display_name(tag: &str) -> String {
    match tag.to_ascii_lowercase().as_str() {
        "en" => "English".to_string(),
        "zh" => "Chinese".to_string(),
        "hi" => "Hindi".to_string(),
        "es" => "Spanish".to_string(),
        "fr" => "French".to_string(),
        "ar" => "Arabic".to_string(),
        "bn" => "Bengali".to_string(),
        "pt" => "Portuguese".to_string(),
        "ru" => "Russian".to_string(),
        "ur" => "Urdu".to_string(),
        "id" => "Indonesian".to_string(),
        "de" => "German".to_string(),
        "ja" => "Japanese".to_string(),
        "sw" => "Swahili".to_string(),
        "mr" => "Marathi".to_string(),
        "te" => "Telugu".to_string(),
        "tr" => "Turkish".to_string(),
        "ta" => "Tamil".to_string(),
        "vi" => "Vietnamese".to_string(),
        "it" => "Italian".to_string(),
        "eo" => "Esperanto".to_string(),
        "io" => "Ido".to_string(),
        "ia" => "Interlingua".to_string(),
        "ie" => "Interlingue".to_string(),
        "vo" => "Volapuk".to_string(),
        "jbo" => "Lojban".to_string(),
        "tlh" => "Klingon".to_string(),
        "tok" => "Toki Pona".to_string(),
        "lfn" => "Lingua Franca Nova".to_string(),
        "nov" => "Novial".to_string(),
        _ => tag.to_string(),
    }
}
