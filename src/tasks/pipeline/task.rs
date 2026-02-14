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
    let user_lang = utils::normalize_lang_tag(&user_lang);
    let user = utils::get_user();
    let (cleaned_request, attachments) = attach::extract_attachments_from_input(request);
    let stdin_content = stdin;
    let attached_files = attach::format_attached_files(
        if stdin_content.trim().is_empty() {
            None
        } else {
            Some(stdin_content.as_str())
        },
        &attachments,
    );

    let mut prompt = format!(
        "USER LANG: {} !important\n:: USER HINT ::\n{}\n:: END USER HINT ::",
        user_lang,
        cleaned_request.trim()
    );
    if let Some(block) = attached_files {
        prompt.push_str("\n\n");
        prompt.push_str(&block);
    }

    let response = service.complete(&prompt).await?;

    if args.verbose {
        println!("\x1b[1m{}:\x1b[0m\n\n{}\n", user.to_uppercase(), prompt);
        println!("\x1b[1mLLM:\x1b[0m\n\n{}", response.trim());
    } else {
        println!("{}", render::render_markdown(&response));
    }

    Ok(())
}
