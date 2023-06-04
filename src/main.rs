#![allow(warnings)]

use clap::{command, Args, Parser, Subcommand};
use kli::{gh_create_repo, gh_remove_repo};

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
    Rep(NewRepArgs),
    Web { name: String },
}

#[derive(Args)]
struct NewRepArgs {
    name: String,
    #[arg(short, long)]
    public: bool,
}

#[derive(Subcommand)]
enum RemCommands {
    Rep(RemRepArgs),
}

#[derive(Args)]
struct RemRepArgs {
    repo: String,
    #[arg(short, long)]
    yes: bool,
}

fn main() {
    let cli = Cli::parse();
    match &cli.command {
        Commands::New(NewCommands::Rep(NewRepArgs { name, public })) => {
            main_handle_result(gh_create_repo(name, *public));
            // match gh_create_repo(name, *public) {
            //     Ok(v) => println!("success"),
            //     Err(e) => eprintln!("{:?}", e),
            // }
        }
        Commands::Rem(RemCommands::Rep(RemRepArgs { repo, yes })) => {
            // match gh_remove_repo(repo, *prompt) {
            //     Ok(v) => println!("success"),
            //     Err(e) => eprintln!("{:?}", e),
            // }
            main_handle_result(gh_remove_repo(repo, *yes));
        }

        Commands::New(NewCommands::Web { name }) => {
            println!("{name}",)
        }
        _ => unreachable!("no commands heere"),
    }
}

fn main_handle_result(result: anyhow::Result<()>) {
    if let Err(e) = result {
        eprintln!("{:?}", e);
    }
}
