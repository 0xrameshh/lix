use clap::Parser;
use tracing_subscriber::EnvFilter;

use lix::{Cli, Commands};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    tracing_subscriber::fmt()
        .with_env_filter(EnvFilter::from_default_env())
        .with_writer(std::io::stderr)
        .init();

    let cli = Cli::parse();

    match cli.command {
        Commands::Extract {
            input,
            out,
            model,
            no_clean,
            keep_incomplete,
            concurrency,
            report,
            quiet,
            ..
        } => lix::commands::handle_extract(
            input,
            out,
            model,
            no_clean,
            keep_incomplete,
            concurrency,
            report,
            quiet,
        ),
        Commands::Convert {
            input,
            out,
            model,
            concurrency,
            report,
            quiet,
        } => lix::commands::handle_convert(input, out, model, concurrency, report, quiet),
        Commands::Clean { input, out } => lix::commands::handle_clean(input, out),
        Commands::Info { file } => lix::commands::handle_info(file),
        Commands::Find { all } => lix::commands::handle_find(all),
        Commands::Verify {
            input,
            golden,
            provider: _provider,
        } => lix::commands::handle_verify(input, golden),
        Commands::Generate { out, count, .. } => lix::commands::handle_generate(&out, count),
        Commands::Studio { input, port } => lix::commands::handle_studio(&input, port),
    }
}
