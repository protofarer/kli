#![allow(warnings)]

use clap::{command, Args, Parser, Subcommand};
use kli::gh_create_remote_repo;

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
}

#[derive(Subcommand)]
enum NewCommands {
    Repo(NewRepoArgs),
    Web { name: String },
}

#[derive(Args)]
struct NewRepoArgs {
    name: String,
    #[arg(short, long)]
    public: bool,
}

fn main() {
    let cli = Cli::parse();
    match &cli.command {
        Commands::New(NewCommands::Repo(NewRepoArgs { name, public })) => {
            gh_create_remote_repo(name, *public);
        }
        Commands::New(NewCommands::Web { name }) => {
            println!("{name}",)
        }
        _ => unreachable!("no commands heere"),
    }
}
