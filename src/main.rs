use clap::Parser;
use sh_aid::config::Config;
use sh_aid::context::SystemContext;
use sh_aid::error::Result;

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// The natural language prompt to convert to a shell command.
    #[arg(required = true, num_args = 1..)]
    prompt: Vec<String>,
}

#[tokio::main]
async fn main() -> Result<()> {
    let args = Args::parse();
    let user_prompt = args.prompt.join(" ");

    println!("Loading configuration...");
    let config = Config::load()?;
    config.validate()?;
    println!("Configuration loaded successfully.");
    println!("Provider: {:?}", config.provider_type);
    println!("Model: {}", config.model);

    println!("\nGathering system context...");
    let context = SystemContext::gather()?;
    println!("System context gathered successfully.");

    println!("\n--- System Context ---");
    println!("{}", context.build_environment_context());
    println!("----------------------");

    println!("\nUser Prompt: {user_prompt}");

    // In a future step, this would be sent to the AI provider.
    // For now, we just print the information we've gathered.

    println!("\nPhase 2 Complete: Core infrastructure is in place.");

    Ok(())
}
