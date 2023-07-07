use anyhow::{anyhow, Context, Result};
use serde::Deserialize;
use std::fs;
use std::io::Read;
use std::path::PathBuf;
use toml;

// Enables using a test config file by specifying path different than production default
pub struct Config {
    // ? CSDR file needed only for read and edit operations, rm from object and read from path as needed
    // pub file: ConfigFile,
    pub path: PathBuf,
    pub ssh_username: Option<String>,
    pub ssh_host: Option<String>,
    pub gh_username: Option<String>,
}

impl Config {
    pub fn new(path: Option<&str>) -> Result<Self> {
        let default_path = "/home/kenny/.config/kli/config.toml";
        let path = match path {
            Some(p) => p,
            None => default_path,
        };

        let cfg = Config::read_config(&path)?;

        let ssh_username: Option<String>;
        let ssh_host: Option<String>;
        match cfg.ssh {
            Some(ssh) => {
                match ssh.username {
                    Some(username) => ssh_username = Some(username),
                    None => ssh_username = None,
                }
                match ssh.host {
                    Some(host) => ssh_host = Some(host),
                    None => ssh_host = None,
                }
            }
            None => {
                ssh_username = None;
                ssh_host = None;
            }
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

    fn read_config(path: &str) -> Result<ConfigFile> {
        let contents = read_toml_to_string(path).unwrap();
        let config_file: ConfigFile = toml::from_str(&contents)
            .with_context(|| format!("Error: could not read toml file"))?;
        Ok(config_file)
    }

    pub fn gh_username(&self) -> Result<&str> {
        match self.gh_username.as_ref() {
            Some(username) => Ok(&username),
            None => Err(anyhow!("No github username specified in config file")),
        }
    }

    pub fn ssh_username(&self) -> Result<&str> {
        match self.ssh_username.as_ref() {
            Some(username) => Ok(&username),
            None => Err(anyhow!("No ssh username specified in config file")),
        }
    }

    pub fn ssh_host(&self) -> Result<&str> {
        match self.ssh_host.as_ref() {
            Some(host) => Ok(&host),
            None => Err(anyhow!("No ssh host specified in config file")),
        }
    }
}

// TOML config file representation
#[derive(Deserialize, Debug)]
pub struct ConfigFile {
    pub ssh: Option<SshConfig>,
    pub github: Option<GithubConfig>,
}

#[derive(Deserialize, Debug)]
pub struct SshConfig {
    pub username: Option<String>,
    pub host: Option<String>,
}

#[derive(Deserialize, Debug)]
pub struct GithubConfig {
    pub username: Option<String>,
}

pub fn read_toml_to_string(filepath: &str) -> Result<String> {
    let mut file = fs::File::open(filepath)
        .with_context(|| format!("Error: could not open file `{}`", filepath))?;
    let mut contents = String::new();
    file.read_to_string(&mut contents)?;
    Ok(contents)
}

#[cfg(test)]
mod test_config {
    use super::*;
    use std::io::Write;

    // consider a struct, impl drop for fs::remove_file
    // creates example toml config file, auto-remove file upon drop
    // auto enumerates/makes unique filenames to avoid collisions during test
    // ...see newrepo pkgjson tests
    // fn test_ctx_toml_cfg(contents: &str) -> Result<String> {}

    #[test]
    fn read_toml_exists() {
        let temp_filepath = "temp.toml";
        let mut temp_file = fs::File::create(temp_filepath).unwrap();
        write!(temp_file, "[ssh]\nuser=\"foo\nhost=\"bar\"").unwrap();

        assert!(read_toml_to_string("temp.toml").is_ok());

        fs::remove_file(temp_filepath).unwrap();
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
        let temp_filepath = "tests/temp.toml";
        let mut temp_file = fs::File::create(temp_filepath).unwrap();
        write!(temp_file, "[foo]\nkey1=\"foo\"\nkey2=\"bar\"").unwrap();

        assert!(Config::new(Some(temp_filepath)).is_ok());

        fs::remove_file(temp_filepath).unwrap();
    }

    #[test]
    fn config_read_y_file_y_ssh() {
        let temp_filepath = "temp.toml";
        let mut temp_file = fs::File::create(temp_filepath).unwrap();
        write!(temp_file, "[ssh]\nuser=\"foo\"\nhost=\"bar\"").unwrap();

        let cfg = Config::new(Some(temp_filepath)).unwrap();

        assert_eq!("foo", cfg.ssh_username().unwrap());
        assert_eq!("bar", cfg.ssh_host().unwrap());
    }
}
