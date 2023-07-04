use std::io::Write;
use std::path::{Path, PathBuf};
use std::{fs::File, process::Command};

use anyhow::{anyhow, Context, Result};
use assert_cmd::prelude::*;
use ctor::ctor;
use predicates::prelude::*;
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

// * Keep here (instead of standalone script) since tests under development may not cleanup correctly
fn cleanup_remote_repos() -> Result<()> {
    // gh repo list protofarer --json name --jq '.[] | .name | match("^name_via.*") | .string'
    // returns a string like "repoOne\nrepoTwo\nrepoThree\n"

    // TODO replace protofarer with TOML config gh username read

    let output = Command::new("/usr/bin/gh")
        .arg("repo")
        .arg("list")
        .arg("protofarer")
        .arg("--json")
        .arg("name")
        .arg("--jq")
        .arg(".[] | .name | match(\"^name_via.*\") | .string")
        .output()
        .context("Error: Failed to run 'gh repo list'")?;

    dbg!(&output.stdout);
    let output_str = std::str::from_utf8(&output.stdout)
        .context("Error: Failed to convert gh repo list output to string")?;
    println!("***** Test repos to delete: ***** \n{}", output_str);

    for repo_name in output_str.lines() {
        if let Err(e) = repo_remote_delete(repo_name) {
            eprintln!("Error deleting repo {}: {:?}", repo_name, e);
        } else {
            println!("Cleaned up repo {}", repo_name);
        }
    }
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

#[test]
fn newrepo_remote_exists() -> Result<()> {
    let ctx = TestContext::new();

    let project_name = ProjectName::FromArg.value();

    Command::new("/usr/bin/git")
        .current_dir(ctx.path())
        .arg("init")
        .status()
        .context("Error: Failed to 'git init'")?;

    let status = Command::new("/usr/bin/gh")
        .arg("repo")
        .arg("create")
        .arg(&project_name)
        .arg("--private")
        .status()
        .context("Error: Failed to execute 'gh repo create'")?;

    if !status.success() {
        return Err(anyhow!(
            "Error: 'gh repo create' failed with exit status {}",
            status
        ));
    }

    let url = format!("https://github.com/protofarer/{}", &project_name);
    let status = Command::new("/usr/bin/git")
        .current_dir(ctx.path())
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

    let mut cmd = Command::cargo_bin("kli")?;
    cmd.arg("new").arg("repo").arg(&project_name);
    cmd.current_dir(ctx.path());
    cmd.assert()
        .stderr(predicate::str::contains(
            "Error: Remote repository already exists",
        ))
        .failure();
    // repo_remote_delete(&project_name)?;

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

#[test]
fn zzz_cleanup() -> Result<()> {
    println!("*********************************************");
    println!("Cleanup for tests",);
    println!("*********************************************");
    cleanup_remote_repos().unwrap();
    println!("*********************************************");
    println!("Fin cleanup",);
    println!("*********************************************");
    Ok(())
}

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
