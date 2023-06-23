use anyhow::{Context, Result};
use assert_cmd::prelude::*;
use ctor::ctor;
use predicates::prelude::*;
use std::io::Write;
use std::path::{Path, PathBuf};
use std::{fs::File, process::Command};
use tempfile::{tempdir, TempDir};
use uuid::Uuid;

pub struct TestContext {
    _tmpdir: TempDir,
    path: PathBuf,
}

impl TestContext {
    pub fn new() -> Self {
        let tmpdir = tempdir().expect("Failed to create temp dir");
        let path = tmpdir.path().to_path_buf();

        Self {
            _tmpdir: tmpdir,
            path,
        }
    }
    pub fn path(&self) -> &PathBuf {
        &self.path
    }
}

pub enum ProjectName {
    FromArg,
    FromJson,
}

impl ProjectName {
    pub fn value(&self) -> String {
        let uuid = Uuid::new_v4();
        match *self {
            ProjectName::FromArg => format!("name_via_arg_{}", uuid),
            ProjectName::FromJson => format!("name_via_json_{}", uuid),
        }
    }
}

// TODO test repos now have unique names base on timestamp, consider doing gh
// list repos and running cleanup code on this with appropriate prefixes per
// ProjectName struct
fn _cleanup_remote_repos() -> Result<()> {
    // run `gh repo list protofarer --json name --jq '.[] | .name | match("^name_via.*") | .string'
    // returns a string like "repoOne\nrepoTwo\nrepoThree\n"
    // ? CSDR moving to a standalone script since tests are reliably doing their own cleanup
    // if let Err(e) = repo_remote_delete(&ProjectName::FromArg.value()) {
    //     eprintln!("{:?}", e);
    // }
    // if let Err(e) = repo_remote_delete(&ProjectName::FromJson.value()) {
    //     eprintln!("{:?}", e);
    // }
    Ok(())
}

#[ctor]
fn before_all_tests() {
    println!("*********************************************");
    println!("Running before_all_tests setup",);
    println!("*********************************************");
    // cleanup_remote_repos().unwrap();
    println!("*********************************************");
    println!("Fin setup",);
    println!("*********************************************");
}

// * NEW REPO

#[test]
fn newrepo_n_localrepo_y_namearg() -> Result<()> {
    let ctx = TestContext::new();

    let project_name = ProjectName::FromArg.value();

    let mut cmd = Command::cargo_bin("kli")?;
    cmd.arg("new").arg("repo").arg(&project_name);
    cmd.current_dir(ctx.path());
    cmd.assert()
        .stdout(predicate::str::contains("No local repo detected"))
        .stdout(predicate::str::contains(format!(
            "Successfully created remote repo {}",
            project_name
        )))
        .success();

    repo_remote_delete(&project_name)?;

    Ok(())
}

#[test]
fn newrepo_n_pkgjson_n_namearg() -> Result<()> {
    let ctx = TestContext::new();

    let mut cmd = Command::cargo_bin("kli")?;
    cmd.arg("new").arg("repo");
    cmd.current_dir(ctx.path());
    cmd.assert()
        .stderr(predicate::str::contains("Error: no repo name was given"))
        .failure();

    Ok(())
}

#[test]
fn newrepo_y_pkgjson_n_namearg() -> Result<()> {
    let ctx = TestContext::new();

    Command::new("/usr/bin/git")
        .arg("init")
        .current_dir(ctx.path())
        .status()
        .context("Failed to execute 'git init'")?;

    let project_name = ProjectName::FromJson.value();

    create_json_file_with_entry(ctx.path(), "name", &project_name)?;

    let mut cmd = Command::cargo_bin("kli")?;
    cmd.arg("new").arg("repo");
    cmd.current_dir(ctx.path());
    cmd.assert()
        .stdout(predicate::str::contains(format!(
            "Successfully created remote repo {}",
            project_name
        )))
        .success();

    repo_remote_delete(&project_name)?;

    Ok(())
}

#[test]
fn newrepo_n_pkgjson_y_namearg() -> Result<()> {
    let ctx = TestContext::new();

    // init git repo
    Command::new("git").arg("init").current_dir(ctx.path());

    let project_name = ProjectName::FromArg.value();

    let mut cmd = Command::cargo_bin("kli")?;
    cmd.arg("new").arg("repo").arg(&project_name);
    cmd.current_dir(ctx.path());
    cmd.assert()
        .stdout(predicate::str::contains(format!(
            "Successfully created remote repo {}",
            project_name
        )))
        .success();

    repo_remote_delete(&project_name)?;

    Ok(())
}

#[test]
fn newrepo_y_pkgjson_y_namearg() -> Result<()> {
    let ctx = TestContext::new();

    create_json_file_with_entry(ctx.path(), "name", "pkgjson_project_name")?;

    let project_name = ProjectName::FromArg.value();

    let mut cmd = Command::cargo_bin("kli")?;
    cmd.arg("new").arg("repo").arg(&project_name);
    cmd.current_dir(ctx.path());
    cmd.assert()
        .stdout(predicate::str::contains(format!(
            "Successfully created remote repo {}",
            project_name
        )))
        .success();

    repo_remote_delete(&project_name)?;

    Ok(())
}

// * NEW SUBDOMAIN

pub enum SubdomainWord {
    FromArg,
    FromJson,
}

impl SubdomainWord {
    fn value(&self) -> &'static str {
        match *self {
            SubdomainWord::FromArg => "subdomain_via_arg",
            SubdomainWord::FromJson => "subdomain_via_json",
        }
    }
}

#[test]
fn newsubdomain_n_pkgjson_n_namearg() -> Result<()> {
    let ctx = TestContext::new();

    let mut cmd = Command::cargo_bin("kli")?;
    cmd.arg("new")
        .arg("subdomain")
        .current_dir(ctx.path())
        .assert()
        .failure()
        .stderr(predicate::str::contains("Error: no subdomain was given"));

    Ok(())
}

#[test]
fn newsubdomain_y_pkgjson_n_namearg() -> Result<()> {
    let ctx = TestContext::new();

    let subdomain_word = SubdomainWord::FromJson.value();
    create_json_file_with_entry(ctx.path(), "name", &subdomain_word)?;

    let mut cmd = Command::cargo_bin("kli")?;
    cmd.arg("new")
        .arg("subdomain")
        .current_dir(ctx.path())
        .assert()
        .stdout(predicate::str::contains(format!(
            "Successfully created subdomain {}",
            subdomain_word
        )))
        .success();

    Ok(())
}

#[test]
fn newsubdomain_n_pkgjson_y_namearg() -> Result<()> {
    let _ctx = TestContext::new();

    let subdomain_word = SubdomainWord::FromArg.value();

    let mut cmd = Command::cargo_bin("kli")?;
    cmd.arg("new")
        .arg("subdomain")
        .arg(&subdomain_word)
        .assert()
        .stdout(predicate::str::contains(format!(
            "Successfully created subdomain {}",
            subdomain_word
        )))
        .success();

    Ok(())
}

#[test]
fn newsubdomain_y_pkgjson_y_namearg() -> Result<()> {
    let ctx = TestContext::new();

    create_json_file_with_entry(ctx.path(), "name", "pkgjson_name")?;

    let subdomain_word = SubdomainWord::FromArg.value();

    let mut cmd = Command::cargo_bin("kli")?;
    cmd.arg("new")
        .arg("subdomain")
        .arg(&subdomain_word)
        .current_dir(ctx.path())
        .assert()
        .stdout(predicate::str::contains(format!(
            "Successfully created subdomain {}",
            subdomain_word
        )))
        .success();

    Ok(())
}

// #[test]
// fn zzz_cleanup() -> Result<()> {
//     println!("*********************************************");
//     println!("Cleanup for tests",);
//     println!("*********************************************");
//     cleanup_remote_repos().unwrap();
//     println!("*********************************************");
//     println!("Fin cleanup",);
//     println!("*********************************************");
//     Ok(())
// }

fn create_json_file_with_entry(dir: &Path, key: &str, value: &str) -> Result<()> {
    let file_path = dir.join("package.json");

    {
        let mut file = File::create(&file_path).expect("Failed to create file");
        writeln!(file, "{{ \"{}\": \"{}\" }}", key, value)?;
    }

    Ok(())
}

fn repo_remote_delete(name: &str) -> Result<()> {
    let status = Command::new("/usr/bin/gh")
        .arg("repo")
        .arg("delete")
        .arg(name)
        .arg("--yes")
        .status()
        .context("Failed to 'gh repo delete'")?;

    if !status.success() {
        eprintln!("gh repo delete <name> --yes failed in some way");
    }

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
