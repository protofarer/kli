use std::{io::BufReader, process::Command};

use anyhow::{anyhow, Context, Result};

pub mod config;

/// # Errors
///
/// Will return `anyhow::Error`:
///     - fails to open or read package.json and no virtual host name was given
///     - fails to find ssh parameters
///     - fails to write virtual host config to remote host
///     - fails nginx config file check
pub fn create_vhost_subdomain(_cfg: &Config, input_name: &Option<String>) -> Result<()> {
    let subdomain_word: String;

    // determine subdomain:
    // 1. if name None => default to cwd package.json name or cargo.toml package name
    // 2. else use given name
    if let Some(name) = input_name {
        subdomain_word = String::from(name);
    } else {
        subdomain_word = read_json_value_from_file("package.json", "name").with_context(|| {
            "Error: no subdomain was given, neither arg nor via package.json name".to_owned()
        })?;
    }

    // options for ssh user/server info
    // - read ssh info from a config file or env
    // - options to set username, host, or ssh nickname for command

    // TODO read ssh info from TOML config file

    // TODO
    // initiate strings for vhost, sites_available, sites_enabled
    // initiate string for vhost file itself, with interpolation
    // save vhost string to tmp file

    // run ssh commands for:
    // - connect
    // - mv from /tmp to sites-avail
    // - soft link from sites-avail to sites-enabled
    // service nginx reload

    // TODO get host string from config file
    println!("Successfully created subdomain {subdomain_word} at CONFIG_FILE_HOST_STRING");
    Ok(())
}

/// # Errors
///
/// Will return `anyhow::Error`:
///     - no repo name was given and fails to open or read package.json
///     - remote repo already exists
///     - fails to create remote repo
///     - fails to set remote repo
pub fn new_remote_repo(cfg: &Config, input_name: &Option<String>, is_public: bool) -> Result<()> {
    let repo_name: String;

    if let Some(name) = input_name {
        repo_name = String::from(name);
    } else {
        repo_name = read_json_value_from_file("package.json", "name").with_context(|| {
            "Error: no repo name was given, neither arg nor via package.json name".to_owned()
        })?;
    }

    // check for local repo
    let output = Command::new("/usr/bin/git")
        .arg("rev-parse")
        .arg("--is-inside-work-tree")
        .output()
        .context("Error: Failed to execute 'git rev-parse --is-inside-work-tree")?;

    if output.status.success() {
        let remote_output = Command::new("/usr/bin/git")
            .arg("remote")
            .arg("-v")
            .output()
            .context("Error: Failed to execute 'git remote -v'")?;

        let remote_str = std::str::from_utf8(&remote_output.stdout).unwrap_or("");
        if remote_str.contains("origin") {
            return Err(anyhow!("Error: Remote repository already exists"));
        }
    }

    // make a local repo first if it doesn't already exist
    if !output.status.success() {
        println!("No local repo detected, creating one for you...");
        Command::new("/usr/bin/git")
            .arg("init")
            .status()
            .context("Error: Failed to 'git init'")?;
    }

    println!("Attempting to create new repo {repo_name}");
    let status = Command::new("/usr/bin/gh")
        .arg("repo")
        .arg("create")
        .arg(&repo_name)
        .arg(if is_public { "--public" } else { "--private" })
        .status()
        .context("Error: Failed to execute 'gh repo create'")?;

    if !status.success() {
        return Err(anyhow!(
            "Error: 'gh repo create' failed with exit status {}",
            status
        ));
    }

    let gh_username = cfg.gh_username()?;

    let url = format!("https://github.com/{}/{}", gh_username, &repo_name);
    let status = Command::new("/usr/bin/git")
        .arg("remote")
        .arg("add")
        .arg("origin")
        .arg(&url)
        .status()
        .context("Error: Failed to execute 'git remote add'")?;

    if !status.success() {
        return Err(anyhow!(
            "Error: 'git remote add' failed with exit status {}",
            status
        ));
    }

    println!("Successfully created remote repo {repo_name}");

    Ok(())
}

/// # Errors
///
/// Will return `anyhow::Error`:
///     - gh command errors out
pub fn remove_remote_repo(_cfg: &Config, name: &str) -> Result<()> {
    let status = Command::new("/usr/bin/gh")
        .arg("repo")
        .arg("delete")
        .arg("https://github.com/protofarer/".to_owned() + name)
        .arg("--yes")
        .status()
        .context("Error: Failed to execute 'gh repo delete --yes'")?;

    if !status.success() {
        return Err(anyhow!(
            "Error: 'gh repo delete' failed with exit status {}",
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

use config::Config;
use std::fs::File;
/// # Errors
///
/// Will return `anyhow::Error`:
///     - fails to open or read json file
pub fn read_json_value_from_file(filepath: &str, key: &str) -> Result<String> {
    let file =
        File::open(filepath).with_context(|| format!("Error: could not open file `{filepath}`"))?;

    let reader = BufReader::new(file);

    let json_value: serde_json::Value = serde_json::from_reader(reader)
        .with_context(|| "Error: value could not be read from json file".to_owned())?;

    Ok(json_value
        .get(key)
        .and_then(serde_json::Value::as_str)
        .with_context(|| "Error: key not found".to_owned())?
        .to_string())
}

#[cfg(test)] // compile and run only on `cargo test`
mod test_read_json_file {
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
            writeln!(file, "{{ \"key1\": \"val1\", \"key2\": \"val2\" }}")
                .expect("Failed to write to file");
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
            writeln!(file, "{{ \"key1\": \"val1\", \"key2\": \"val2\" }}")
                .expect("Failed to write to file");
        }

        let result = read_json_value_from_file(file_path.to_str().unwrap(), "keyZ");
        assert_eq!(result.is_err(), true);
    }
}
