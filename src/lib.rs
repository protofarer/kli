use std::process::{Command, Stdio};

use anyhow::{anyhow, Context, Result};
use clap::{arg, command, Arg, ArgAction};

pub fn gh_create_remote_repo(name: &str, is_public: bool) -> Result<()> {
    println!("gh fn recvd: name {} public flag {}", name, is_public);
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
