use crate::core;
use crate::utils;

pub async fn generate_message(
    service: &core::Service,
    args: &core::Cli,
    request: &str,
    stdin: String,
) -> Result<(), Box<dyn std::error::Error>> {
    let user = utils::get_user();
    let datetime = utils::current_datetime();

    let preamble = format!(
        "LLM name: Netero\nUser name: {}\nDate and hour: {}\n",
        user, datetime
    );

    let prompt = if stdin.trim().is_empty() {
        format!("User request:\n {}\n", request.trim())
    } else {
        format!(
            "== USER REQUEST ==\n{}\n== END USER REQUEST ==\n\n== STDIN FILE ==\n{}\n== END STDIN FILE ==\n",
            request.trim(),
            stdin,
        )
    };

    let wrapper = format!("{}\n{}", preamble, prompt);

    let response = service.complete(&wrapper).await?;

    if args.verbose {
        println!("\x1b[1m{}:\x1b[0m\n\n{}\n", user.to_uppercase(), wrapper);
        println!("\x1b[1mLLM:\x1b[0m\n\n{}", response.trim());
    } else {
        println!("{}", response.trim());
    }

    Ok(())
}
