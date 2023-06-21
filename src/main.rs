#![allow(warnings)]

use clap::{command, Args, Parser, Subcommand};
use kli::{create_vhost_subdomain, gh_create_repo, gh_remove_repo};

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    #[command(subcommand)]
    New(NewCommands),

    #[command(subcommand)]
    Rem(RemCommands),
}

#[derive(Subcommand)]
enum NewCommands {
    Repo(NewRepoArgs),
    Web { name: String },
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
    Repo(RemRepoArgs),
}

#[derive(Args)]
struct RemRepoArgs {
    repo: String,
    #[arg(short, long)]
    yes: bool,
}

fn main() {
    let cli = Cli::parse();
    match &cli.command {
        Commands::New(NewCommands::Repo(NewRepoArgs { name, public })) => {
            handle_result(gh_create_repo(name, *public));
        }
        Commands::New(NewCommands::Subdomain { name }) => {
            handle_result(create_vhost_subdomain(name));
        }
        Commands::Rem(RemCommands::Repo(RemRepoArgs { repo, yes })) => {
            handle_result(gh_remove_repo(repo, *yes));
        }

        Commands::New(NewCommands::Web { name }) => {
            println!("{name}",)
        }
        _ => unreachable!("no commands here"),
    }
}

fn handle_result(result: anyhow::Result<()>) {
    if let Err(e) = result {
        panic!("{}", e)
        // eprintln!("{:?}", e);
    }
}
