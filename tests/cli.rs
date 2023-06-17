use assert_cmd::prelude::*;
// use assert_fs::prelude::*;
use anyhow::{anyhow, Context, Result};
use predicates::prelude::*;
use std::io::Write;
use std::{error::Error, fs::File, process::Command};
use tempfile::tempdir;

#[test]
fn subdomain_no_pkgjson_no_name() -> Result<()> {
    // no subdomain word, no package.json cwd
    let mut cmd = Command::cargo_bin("kli")?;
    cmd.arg("new").arg("subdomain");
    cmd.assert()
        .failure()
        .stderr(predicate::str::contains("cannot read package.json"));
    Ok(())
}

#[test]
fn subdomain_pkgjson_exists() -> Result<()> {
    let dir = tempdir().expect("Failed to create temp dir");
    let file_path = dir.path().join("package.json");

    {
        let mut file = File::create(&file_path).expect("Failed to create file");
        writeln!(
            file,
            "{{ \"name\": \"the-foo-project\", \"key2\": \"val2\" }}"
        )?;
    }
    let cwd = std::env::current_dir().expect("Failed to get cwd");

    Command::new("cd")
        .arg(dir.as_ref())
        .output()
        .context("Failed to execute 'cd <tempdir>")?;

    let mut cmd = Command::cargo_bin("kli")?;
    cmd.arg("new").arg("subdomain");
    cmd.assert().success();
    // .stderr(predicate::str::contains("cannot read package.json"));

    Command::new("cd").arg(cwd);
    Ok(())
}

// benchmarks in ~/benches
// examples in ~/examples

// * EXAMPLE: PANIC or ON FAILURE TEST STDERROR
// #[test]
// #[should_panic]
// fn file_doesnt_exist() -> () {
//     let mut cmd = Command::cargo_bin("kli")?; // compiles main binary
//     cmd.arg("some_read_subcmd").arg("test/file/doesnt/exist");
//     cmd.assert()
//         .failure()
//         .stderr(predicate::str::contains("could not read file"));
//     Ok(())
// }

// * EXAMPLE: TEST STDOUT ON SUCCESS
// #[test]
// fn find_content_in_file() -> Result<(), Box<dyn Error>> {
//     let file = assert_fs::NamedTempFile::new("sample.txt")?;
//     file.write_str("A test\nActual content\nMore content\nAnother test")?;

//     let mut cmd = Command::cargo_bin("kli")?;
//     cmd.arg("test").arg(file.path());
//     cmd.assert()
//         .success()
//         .stdout(predicate::str::contains("test\nAnother test"));

//     Ok(())
// }

// Add integration tests for passing an empty string as pattern. Adjust the program as needed.

// * EXAMPLE: Using assert_fs
// ? integration tests for reading structured data from files (json, toml)
// #[test]
// fn find_with_empty_string() -> Result<(), Box<dyn Error>> {
//     let file = assert_fs::NamedTempFile::new("sample.txt")?;
//     file.write_str("A test\nActual content\nMore content\nAnother test")?;

//     let mut cmd = Command::cargo_bin("kli")?;
//     cmd.arg("".to_string()).arg(file.path());
//     cmd.assert()
//         .failure()
//         .stderr(predicate::str::contains("pattern cannot be empty"));
//     Ok(())
// }
