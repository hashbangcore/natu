#![allow(unused)]

use ron::de::from_str;
use serde::Deserialize;
use std::fs;
use std::process::Command;

#[derive(Deserialize)]
struct Prompts {
    convention: String,
    instruction: String,
    skeleton: String,
}

fn expand_prompt(path: &str) -> String {
    fs::read_to_string(path)
        .unwrap_or_else(|_| format!("Error: No se pudo leer el archivo {}", path))
}

fn cover(title: &str, content: &str) -> String {
    let t = title.to_uppercase();
    format!("== START {t} ==\n{content}\n== END {t} ==")
}

fn staged_changes() -> String {
    let output = Command::new("git").args(["diff", "--staged"]).output();

    match output {
        Ok(out) if out.status.success() => String::from_utf8(out.stdout)
            .unwrap_or_else(|e| format!("Error UTF-8 en git diff --staged: {e}")),
        Ok(out) => {
            let err = String::from_utf8_lossy(&out.stderr);
            format!("git diff --staged falló:\n{err}")
        }
        Err(e) => format!("No se pudo ejecutar git diff --staged: {e}"),
    }
}

pub fn generate(hint: Option<&str>) -> String {
    let user_hint = match hint {
        Some(h) => h,
        None => "",
    };

    let ron_data = include_str!("prompt.ron");
    let p: Prompts = from_str(ron_data).expect("Error parseando RON");
    let context = String::from("contexto del repositorio");
    let staged_changes = staged_changes();

    let sections = [
        ("CONVENTION", expand_prompt(&p.convention)),
        ("PROJECT CONTEXT", context),
        ("USER HINT", user_hint.to_string()),
        ("INSTRUCTION", expand_prompt(&p.instruction)),
        ("SKELETON", expand_prompt(&p.skeleton)),
        ("STAGED CHANGES", staged_changes),
    ];

    sections
        .iter()
        .map(|(title, content)| cover(title, content))
        .collect::<Vec<_>>()
        .join("\n\n")
}
