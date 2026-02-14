/// Loads the commit instruction prompt template.
pub fn instruction() -> &'static str {
    include_str!("prompts/instruction.txt")
}

/// Loads the commit convention prompt template.
pub fn convention() -> &'static str {
    include_str!("prompts/convention.txt")
}

/// Loads the commit message skeleton prompt template.
pub fn skeleton() -> &'static str {
    include_str!("prompts/skeleton.txt")
}
