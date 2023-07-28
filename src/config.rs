use anyhow::{anyhow, Context, Result};
use serde::Deserialize;
use std::fs::{self};
use std::io::Read;
use std::path::PathBuf;
use toml;

// TODO cleanup temp files

// Enables using a test config file by specifying path different than production default
pub struct Config {
    // ? CSDR file needed only for read and edit operations, rm from object and read from path as needed
    // pub file: ParsedFile,
    pub path: PathBuf,
    pub ssh_username: Option<String>,
    pub ssh_host: Option<String>,
    pub gh_username: Option<String>,
}

impl Config {
    /// # Errors
    ///
    /// Returns `anyhow::Result/Error` if problems reading config file
    pub fn new(path: Option<&str>) -> Result<Self> {
        let default_path = "/home/kenny/.config/kli/config.toml";
        let path = path.map_or(default_path, |p| p);

        let cfg = Self::read_config(path)?;

        let ssh_username: Option<String>;
        let ssh_host: Option<String>;

        if let Some(ssh) = cfg.ssh {
            match ssh.username {
                Some(username) => ssh_username = Some(username),
                None => ssh_username = None,
            }
            match ssh.host {
                Some(host) => ssh_host = Some(host),
                None => ssh_host = None,
            }
        } else {
            ssh_username = None;
            ssh_host = None;
        }

        let gh_username: Option<String>;
        match cfg.github {
            Some(github) => match github.username {
                Some(username) => gh_username = Some(username),
                None => gh_username = None,
            },
            None => gh_username = None,
        }

        Ok(Self {
            path: path.into(),
            ssh_username,
            ssh_host,
            gh_username,
            // file: config_file,
        })
    }

    fn read_config(path: &str) -> Result<ParsedFile> {
        let contents = read_toml_to_string(path)?;
        let config_file: ParsedFile = toml::from_str(&contents)
            .with_context(|| "Error: could not read toml file".to_owned())?;
        Ok(config_file)
    }

    /// # Errors
    ///
    /// Will return `anyhow::Error` if github username not in file
    pub fn gh_username(&self) -> Result<&str> {
        self.gh_username.as_deref().map_or_else(
            || Err(anyhow!("No github username specified in config file")),
            Ok,
        )
    }

    /// # Errors
    ///
    /// Will return `anyhow::Error` if ssh username not in file
    pub fn ssh_username(&self) -> Result<&str> {
        self.ssh_username.as_deref().map_or_else(
            || Err(anyhow!("No ssh username specified in config file")),
            Ok,
        )
    }

    /// # Errors
    ///
    /// Will return `anyhow::Error` if ssh host not in file
    pub fn ssh_host(&self) -> Result<&str> {
        self.ssh_host
            .as_deref()
            .map_or_else(|| Err(anyhow!("No ssh host specified in config file")), Ok)
    }
}

// TOML config file representation
#[derive(Deserialize, Debug)]
pub struct ParsedFile {
    pub ssh: Option<SshParameters>,
    pub github: Option<GithubParameters>,
}

#[derive(Deserialize, Debug)]
pub struct SshParameters {
    pub username: Option<String>,
    pub host: Option<String>,
}

#[derive(Deserialize, Debug)]
pub struct GithubParameters {
    pub username: Option<String>,
}

/// # Errors
///
/// Will return `anyhow::Error` if cannot open or read file
pub fn read_toml_to_string(filepath: &str) -> Result<String> {
    let mut file = fs::File::open(filepath)
        .with_context(|| format!("Error: could not open file `{filepath}`"))?;
    let mut contents = String::new();
    file.read_to_string(&mut contents)?;
    Ok(contents)
}

#[cfg(test)]
mod test_config {
    use super::*;
    use std::fs::File;
    use std::io::Write;
    use std::path::Path;
    use uuid::Uuid;

    pub struct TestFile {
        file: File,
        path: PathBuf,
    }

    impl TestFile {
        pub fn new() -> Self {
            let temp_filepath = generate_unique_filepath("tests/temp.toml");
            let temp_file = fs::File::create(&temp_filepath).unwrap();

            Self {
                file: temp_file,
                path: PathBuf::from(temp_filepath),
            }
        }
        pub fn path(&self) -> &PathBuf {
            &self.path
        }
        pub fn file(&self) -> &File {
            &self.file
        }
    }

    impl Drop for TestFile {
        fn drop(&mut self) {
            let _ = fs::remove_file(&self.path).unwrap();
        }
    }

    fn generate_unique_filepath(path: &str) -> String {
        let uuid = Uuid::new_v4();
        let path = Path::new(path);

        // get directory, filename without extension, and extension
        let parent = path.parent().unwrap_or(Path::new("")).to_path_buf();
        let stem = path.file_stem().unwrap().to_str().unwrap();
        let ext = path.extension().unwrap().to_str().unwrap();

        // construct the new path
        let new_filename = format!("{}-{}.{}", stem, uuid, ext);
        let new_path = parent.join(new_filename);

        new_path.to_str().unwrap().to_string()
    }

    #[test]
    fn read_toml_exists() {
        let ctx = TestFile::new();
        // let (filepath, mut file) = generate_tempfile();
        write!(ctx.file(), "[ssh]\nusername=\"foo\"\nhost=\"bar\"").unwrap();
        assert!(read_toml_to_string(ctx.path.to_str().unwrap()).is_ok());
    }

    #[test]
    fn read_toml_missing() {
        assert!(read_toml_to_string("ref/wumpus.toml").is_err());
    }

    #[test]
    fn config_read_n_file() {
        assert!(Config::new(Some("config.toml")).is_err());
    }

    #[test]
    fn config_read_y_file_n_ssh() {
        // create temp config file
        // let temp_filepath = "tests/temp.toml";
        let ctx = TestFile::new();
        write!(ctx.file(), "[foo]\nkey1=\"foo\"\nkey2=\"bar\"").unwrap();

        assert!(Config::new(Some(ctx.path().to_str().unwrap())).is_ok());
    }

    #[test]
    fn config_read_y_file_y_ssh() {
        let ctx = TestFile::new();
        write!(ctx.file(), "[ssh]\nusername=\"foo\"\nhost=\"bar\"").unwrap();

        let cfg = Config::new(Some(&ctx.path().to_str().unwrap())).unwrap();

        assert_eq!("foo", cfg.ssh_username().unwrap());
        assert_eq!("bar", cfg.ssh_host().unwrap());
    }
}
