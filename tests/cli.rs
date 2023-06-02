use assert_cmd::prelude::*;
use assert_fs::prelude::*;
use predicates::prelude::*;
use std::{error::Error, process::Command};

// integration tests here
// benchmarks in ~/benches
// examples in ~/examples
#[test]
fn find_a_match() {
    let mut result = vec![];
    kli::find_matches("lorem ipsum\ndolor sit amet", "lorem", &mut result).unwrap();
    assert_eq!(b"lorem ipsum\n".to_vec(), result);
}

#[test]
fn file_doesnt_exist() -> Result<(), Box<dyn Error>> {
    let mut cmd = Command::cargo_bin("kli")?; // compiles main binary
    cmd.arg("fookwa").arg("test/file/doesnt/exist");
    cmd.assert()
        .failure()
        .stderr(predicate::str::contains("could not read file"));
    Ok(())
}

#[test]
fn find_content_in_file() -> Result<(), Box<dyn Error>> {
    let file = assert_fs::NamedTempFile::new("sample.txt")?;
    file.write_str("A test\nActual content\nMore content\nAnother test")?;

    let mut cmd = Command::cargo_bin("kli")?;
    cmd.arg("test").arg(file.path());
    cmd.assert()
        .success()
        .stdout(predicate::str::contains("test\nAnother test"));

    Ok(())
}

// Add integration tests for passing an empty string as pattern. Adjust the program as needed.
#[test]
fn find_with_empty_string() -> Result<(), Box<dyn Error>> {
    let file = assert_fs::NamedTempFile::new("sample.txt")?;
    file.write_str("A test\nActual content\nMore content\nAnother test")?;

    let mut cmd = Command::cargo_bin("kli")?;
    cmd.arg("".to_string()).arg(file.path());
    cmd.assert()
        .failure()
        .stderr(predicate::str::contains("pattern cannot be empty"));
    Ok(())
}
