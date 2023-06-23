#![allow(warnings)]

use std::{
    io::BufReader,
    process::{Command, Stdio},
};

use anyhow::{anyhow, Context, Result};
use clap::{arg, command, Arg, ArgAction};

pub fn create_vhost_subdomain(input_name: &Option<String>) -> Result<()> {
    let mut subdomain_word: String;
    if let Some(name) = input_name {
        subdomain_word = String::from(name);
    } else {
        subdomain_word = read_json_value_from_file("package.json", "name").with_context(|| {
            format!("Error: no subdomain was given, neither arg nor via package.json name")
        })?;
    }

    // ! try "kli new subdomain"
    // ! try "kli new subdomain hoohee"
    // ! try "kli new subdomain" w/ test .json (write test)

    // ? CSDR
    // - options to set username, host, or ssh nickname for command
    // - read ssh info from a config file or env

    // TODO
    // determine subdomain:
    // 1. if name None => default to cwd package.json name or cargo.toml package name
    // 2. else use given name

    // read username:host or ssh nickname (ssh config file)

    // initiate strings for vhost, sites_available, sites_enabled
    // initiate string for vhost file itself, with interpolation
    // save vhost string to tmp file

    // run ssh commands for:
    // - connect
    // - mv from /tmp to sites-avail
    // - soft link from sites-avail to sites-enabled
    // service nginx reload

    // TODO get host string from config file
    println!(
        "Successfully created subdomain {} at {}",
        subdomain_word, "CONFIG_FILE_HOST_STRING"
    );
    Ok(())
}

pub fn new_remote_repo(input_name: &Option<String>, is_public: bool) -> Result<()> {
    let mut repo_name: String;

    if let Some(name) = input_name {
        repo_name = String::from(name);
    } else {
        repo_name = read_json_value_from_file("package.json", "name").with_context(|| {
            format!("Error: no repo name was given, neither arg nor via package.json name")
        })?;
    }

    // check for local repo
    let output = Command::new("/usr/bin/git")
        .arg("rev-parse")
        .arg("--is-inside-work-tree")
        .output()
        .context("Failed to execute 'git rev-parse --is-inside-work-tree")?;

    // make a local repo first if it doesn't already exist
    if !output.status.success() {
        println!("No local repo detected, creating one for you...");
        Command::new("/usr/bin/git")
            .arg("init")
            .status()
            .context("Failed to 'git init'");
    }

    println!("Attempting to create new repo {}", repo_name);
    let status = Command::new("/usr/bin/gh")
        .arg("repo")
        .arg("create")
        .arg(&repo_name)
        .arg(if is_public { "--public" } else { "--private" })
        .status()
        .context("Failed to execute 'gh repo create'")?;

    if !status.success() {
        return Err(anyhow!(
            "'gh repo create' failed with exit status {}",
            status
        ));
    }

    let url = format!("https://github.com/protofarer/{}", &repo_name);
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

    // * not generally used
    // let status = Command::new("/usr/bin/git")
    //     .arg("push")
    //     .arg("-u")
    //     .arg("origin")
    //     .arg("main")
    //     .status()
    //     .context("Failed to execute 'git push -u origin main'")?;

    // if !status.success() {
    //     return Err(anyhow!(
    //         "'git push -u origin main' failed with exit status {}",
    //         status
    //     ));
    // }

    println!("Successfully created remote repo {}", repo_name);

    Ok(())
}

pub fn remove_remote_repo(name: &str) -> Result<()> {
    let status = Command::new("/usr/bin/gh")
        .arg("repo")
        .arg("delete")
        .arg("https://github.com/protofarer/".to_owned() + name)
        .arg("--yes")
        .status()
        .context("Failed to execute 'gh repo delete --yes'")?;

    if !status.success() {
        return Err(anyhow!(
            "'gh repo delete' failed with exit status {}",
            status
        ));
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

use serde_json;
use std::fs::File;
pub fn read_json_value_from_file(filepath: &str, key: &str) -> Result<String> {
    let file = File::open(filepath)
        .with_context(|| format!("Error: could not open file `{}`", filepath))?;

    let reader = BufReader::new(file);

    let json_value: serde_json::Value = serde_json::from_reader(reader)
        .with_context(|| format!("Error: value could not be read from json file"))?;

    Ok(json_value
        .get(key)
        .and_then(serde_json::Value::as_str)
        .with_context(|| format!("Error: key not found"))?
        .to_string())
}

#[cfg(test)] // compile and run only on `cargo test`
mod read_json_file {
    use super::*;
    use std::fs::File;
    use std::io::Write;
    use tempfile::tempdir;

    #[test]
    fn read_json_missing_file() {
        assert!(read_json_value_from_file("foo", "testkey").is_err());
    }

    #[test]
    fn read_value_from_json_file() {
        let dir = tempdir().expect("Failed to create temp dir");
        let file_path = dir.path().join("test.json");

        {
            let mut file = File::create(&file_path).expect("Failed to create file");
            writeln!(file, "{{ \"key1\": \"val1\", \"key2\": \"val2\" }}");
        }

        let result = read_json_value_from_file(file_path.to_str().unwrap(), "key1").unwrap();
        assert_eq!(result, "val1");

        let result = read_json_value_from_file(file_path.to_str().unwrap(), "key2").unwrap();
        assert_eq!(result, "val2");
    }

    #[test]
    fn missing_key_json_file() {
        let dir = tempdir().expect("Failed to create temp dir");
        let file_path = dir.path().join("test.json");

        {
            let mut file = File::create(&file_path).expect("Failed to create file");
            writeln!(file, "{{ \"key1\": \"val1\", \"key2\": \"val2\" }}");
        }

        let result = read_json_value_from_file(file_path.to_str().unwrap(), "keyZ");
        assert_eq!(result.is_err(), true);
    }
}
