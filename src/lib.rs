use std::process::{Command, Stdio};

use anyhow::{anyhow, Context, Result};
use clap::{arg, Arg, ArgAction, Parser};

#[derive(Parser)]
#[command(arg_required_else_help = true)]
pub struct Args {
    // #[arg(short = 'o', long = "output")]
    pattern: String,
    path: std::path::PathBuf,
}

pub fn run() -> Result<()> {
    // let args = Args::parse();
    cli();

    // gh_new_remote_repo()?;

    // String::from_utf8(output.stdout)?
    //     .lines()
    //     .for_each(|x| println!("{}", x));

    // if args.pattern.is_empty() {
    //     return Err(anyhow!("pattern cannot be empty"));
    // }

    // let content = std::fs::read_to_string(&args.path)
    //     .with_context(|| format!("could not read file `{}`", &args.path.display()))?;

    // find_matches(&content, &args.pattern, &mut std::io::stdout())?;

    Ok(())
}

fn gh_create_remote_repo(name: &str, is_public: bool) -> Result<()> {
    let privacy = if is_public { "--public" } else { "--private" };

    println!("gh fn recvd: name {} public flag {}", name, privacy);
    // let status = Command::new("/usr/bin/gh")
    //     .arg("repo")
    //     .arg("create")
    //     .arg(name)
    //     .arg(privacy)
    //     .status()
    //     .context("Failed to execute 'gh repo create'")?; // for OS level errors

    // // check success of command exec itself
    // if !status.success() {
    //     return Err(anyhow!(
    //         "'gh repo create' failed with exit status {}",
    //         status
    //     ));
    // }

    // let url = format!("https://github.com/protofarer/{}", name);
    // let status = Command::new("/usr/bin/gh")
    //     .arg("remote")
    //     .arg("add")
    //     .arg("add")
    //     .arg("origin")
    //     .arg(&url)
    //     .status()
    //     .context("Failed to execute 'git remote add'")?;

    // if !status.success() {
    //     return Err(anyhow!(
    //         "'git remote add' failed with exit status {}",
    //         status
    //     ));
    // }

    Ok(())
}

fn cli() -> Result<()> {
    let matches = clap::Command::new("kli")
        .version("0.1.0")
        .author("Kenny <kennybaron@fastmail.com")
        .about("Kenny's CLI Omnitool")
        .subcommand_required(true)
        .arg_required_else_help(true)
        .allow_external_subcommands(true)
        .subcommand(
            clap::Command::new("repo")
                .about("Creates a new GitHub repo, private by default")
                .arg(arg!([NAME]))
                .arg_required_else_help(true)
                .about("Set repo name")
                // .required(true)
                .arg(
                    Arg::new("public")
                        .long("public")
                        .short('p')
                        .action(ArgAction::SetTrue),
                )
                .about("Makes a public repo"),
        )
        .get_matches();
    match matches.subcommand() {
        Some(("repo", sub_matches)) => {
            let name = sub_matches.get_one::<String>("NAME").unwrap();
            let is_public = sub_matches.get_flag("public");
            gh_create_remote_repo(name, is_public)?;
        }
        _ => unreachable!("Exhausted list of subcommands, subcomman_required prevents `None`"),
    }
    // let name = matches.value_of("name").unwrap();
    // let is_public = matches.is_present("public");
    // run_create_repo(name, is_public)?;
    Ok(())
}

pub fn find_matches(content: &str, pattern: &str, mut writer: impl std::io::Write) -> Result<()> {
    for line in content.lines() {
        if line.contains(pattern) {
            writeln!(writer, "{}", line)
                .with_context(|| format!("problem writing to buffer"))
                .unwrap();
        }
    }
    Ok(())
}

// let pb = indicatif::ProgressBar::new(100);
// pb.set_style(
//     ProgressStyle::with_template(
//         "[{elapsed_precise}] {bar:40.cyan/blue} {pos:>7}/{len:7} {msg}",
//     )
//     .unwrap()
//     .progress_chars("##-"),
// );
// for i in 0..20 {
//     thread::sleep(Duration::from_millis(100));
//     pb.println(format!("[+] finished {}%", i * 5));
//     pb.inc(5);
// }
// pb.finish_with_message("Ok, DONE!");
