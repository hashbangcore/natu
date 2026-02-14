/// Normalizes a language tag to a short lowercase form (e.g., "en_US" -> "en").
pub fn normalize_lang_tag(input: &str) -> String {
    let trimmed = input.trim();
    if trimmed.is_empty() {
        return "unknown".to_string();
    }

    let base = trimmed
        .split('.')
        .next()
        .unwrap_or(trimmed)
        .split('@')
        .next()
        .unwrap_or(trimmed)
        .replace('_', "-");

    let mut parts = base.splitn(2, '-');
    let lang = parts.next().unwrap_or("").to_ascii_lowercase();
    if lang.is_empty() {
        return "unknown".to_string();
    }

    let _region = parts.next().unwrap_or("");
    lang
}
