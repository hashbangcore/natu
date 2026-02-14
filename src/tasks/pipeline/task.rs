use crate::core;
use crate::tasks::attach;
use crate::tasks::render;
use crate::utils;

pub async fn generate_message(
    service: &core::Service,
    args: &core::Cli,
    request: &str,
    stdin: String,
) -> Result<(), Box<dyn std::error::Error>> {
    let user_lang = utils::get_user_lang();
    let user = utils::get_user();
    let (cleaned_request, attachments) = attach::extract_attachments_from_input(request);
    let attachment_block = attach::format_attachments(&attachments);

    let preamble = format!("USER LANG: {}", user_lang);

    let stdin_with_files = if let Some(extra) = attachment_block.as_deref() {
        format!("{stdin}{extra}")
    } else {
        stdin
    };

    let prompt = if stdin_with_files.trim().is_empty() {
        format!("User request:\n{}", cleaned_request.trim())
    } else {
        format!(
            "== USER REQUEST ==\n{}\n== END USER REQUEST ==\n== STDIN FILE ==\n{}\n== END STDIN FILE ==",
            cleaned_request.trim(),
            stdin_with_files,
        )
    };

    let wrapper = format!("{}\n{}", preamble, prompt);

    let response = service.complete(&wrapper).await?;

    if args.verbose {
        println!("\x1b[1m{}:\x1b[0m\n\n{}\n", user.to_uppercase(), wrapper);
        println!("\x1b[1mLLM:\x1b[0m\n\n{}", response.trim());
    } else {
        println!("{}", render::render_markdown(&response));
    }

    Ok(())
}
