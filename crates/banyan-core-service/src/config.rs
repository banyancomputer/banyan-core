use pico_args::Arguments;
use std::path::PathBuf;

pub fn parse_arguments() -> Result<Config, pico_args::Error> {
    let mut args = Arguments::from_env();

    Ok(Config {
        database_url: args
            .opt_value_from_str("--db-url")?
            .unwrap_or("sqlite://./data/server.db".into()),

        signing_key_path: args
            .opt_value_from_str("--signing-key")?
            .unwrap_or("./data/signing-key.pem".into()),

        bucket_metadata_upload_directory: args
            .opt_value_from_str("--upload-dir")?
            .unwrap_or("./data/uploads".into()),

        storage_host_url: args
            .opt_value_from_str("--storage-host-url")?
            .unwrap_or("http://localhost:3002".into()),
    })
}

#[derive(Debug)]
pub struct Config {
    database_url: String,
    signing_key_path: PathBuf,
    bucket_metadata_upload_directory: PathBuf,
    storage_host_url: String,
}

impl Config {
    pub fn database_url(&self) -> &str {
        self.database_url.as_str()
    }

    pub fn signing_key_path(&self) -> &PathBuf {
        &self.signing_key_path
    }

    pub fn bucket_metadata_upload_directory(&self) -> &PathBuf {
        &self.bucket_metadata_upload_directory
    }

    pub fn storage_host_url(&self) -> &str {
        self.storage_host_url.as_str()
    }
}
