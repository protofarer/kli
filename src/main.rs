#![allow(warnings)]

use clap::{command, Args, Parser, Subcommand};
use kli::{config::Config, create_vhost_subdomain, new_remote_repo, remove_remote_repo};

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
struct Cli {
    /// top level command
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Create something new
    #[command(subcommand)]
    New(NewCommands),

    /// Delete something existing
    #[command(subcommand)]
    Rem(RemCommands),
}

#[derive(Subcommand)]
enum NewCommands {
    /// Create remote repo
    Repo(NewRepoArgs),

    /// Start new web app project
    Web { name: String },

    /// Create subdomain on designated host
    Subdomain { name: Option<String> },
}

#[derive(Args)]
struct NewRepoArgs {
    name: Option<String>,
    #[arg(short, long)]
    public: bool,
}

#[derive(Subcommand)]
enum RemCommands {
    /// Delete remote repo
    Repo(RemRepoArgs),
}

#[derive(Args)]
struct RemRepoArgs {
    repo_name: String,
}

fn main() {
    let cli = Cli::parse();
    let cfg = Config::new(None).unwrap_or_else(|e| {
        eprintln!("{}", e);
        panic!();
    });

    match &cli.command {
        Commands::New(NewCommands::Repo(NewRepoArgs { name, public })) => {
            handle_result(new_remote_repo(&cfg, name, *public));
        }
        Commands::New(NewCommands::Subdomain { name }) => {
            handle_result(create_vhost_subdomain(&cfg, name));
        }
        Commands::Rem(RemCommands::Repo(RemRepoArgs { repo_name })) => {
            handle_result(remove_remote_repo(&cfg, repo_name));
        }

        Commands::New(NewCommands::Web { name }) => {
            println!("{name}",)
        }
        _ => unreachable!("no commands here"),
    }
}

fn handle_result(result: anyhow::Result<()>) {
    if let Err(e) = result {
        eprintln!("{}", e);
        panic!()
        // eprintln!("{:?}", e);
    }
}
