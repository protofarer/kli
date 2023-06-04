#![allow(warnings)]

use std::process::{Command, Stdio};

use anyhow::{anyhow, Context, Result};
use clap::{arg, command, Arg, ArgAction};

pub fn gh_create_repo(name: &str, is_public: bool) -> Result<()> {
    // cwd must have a git repo
    let output = Command::new("/usr/bin/git")
        .arg("rev-parse")
        .arg("--is-inside-work-tree")
        .output()
        .context("Failed to execute 'git rev-parse --is-inside-work-tree")?;

    if !output.status.success() {
        return Err(anyhow!(
            "Error: Cannot create remote repo without a local repo in cwd"
        ));
    }

    let status = Command::new("/usr/bin/gh")
        .arg("repo")
        .arg("create")
        .arg(name)
        .arg(if is_public { "--public" } else { "--private" })
        .status()
        .context("Failed to execute 'gh repo create'")?;

    if !status.success() {
        return Err(anyhow!(
            "'gh repo create' failed with exit status {}",
            status
        ));
    }

    let url = format!("https://github.com/protofarer/{}", name);
    let status = Command::new("/usr/bin/git")
        .arg("remote")
        .arg("add")
        .arg("origin")
        .arg(&url)
        .status()
        .context("Failed to execute 'git remote add'")?;

    if !status.success() {
        return Err(anyhow!(
            "'git remote add' failed with exit status {}",
            status
        ));
    }

    let status = Command::new("/usr/bin/git")
        .arg("push")
        .arg("-u")
        .arg("origin")
        .arg("main")
        .status()
        .context("Failed to execute 'git push -u origin main'")?;

    if !status.success() {
        return Err(anyhow!(
            "'git push -u origin main' failed with exit status {}",
            status
        ));
    }

    Ok(())
}

pub fn gh_remove_repo(name: &str, yes: bool) -> Result<()> {
    let status = match yes {
        true => Command::new("/usr/bin/gh")
            .arg("repo")
            .arg("delete")
            .arg("https://github.com/protofarer/".to_owned() + name)
            .arg("--yes")
            .status()
            .context("Failed to execute 'gh repo delete --yes'")?,
        false => Command::new("/usr/bin/gh")
            .arg("repo")
            .arg("delete")
            .arg("https://github.com/protofarer/".to_owned() + name)
            .status()
            .context("Failed to execute 'gh repo delete'")?,
    };

    if !status.success() {
        return Err(anyhow!(
            "'gh repo delete' failed with exit status {}",
            status
        ));
    }

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
