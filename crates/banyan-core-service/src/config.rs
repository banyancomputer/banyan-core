use pico_args::Arguments;
use std::path::PathBuf;

#[derive(Debug)]
pub struct Config {
    pub database_url: String,
    pub signing_key: PathBuf,
    pub upload_directory: PathBuf,
}

pub fn parse_arguments() -> Result<Config, pico_args::Error> {
    let mut args = Arguments::from_env();

    Ok(Config {
        database_url: args
            .opt_value_from_str("--db-url")?
            .unwrap_or("sqlite://./data/server.db".into()),

        signing_key: args
            .opt_value_from_str("--signing-key")?
            .unwrap_or("./data/signing-key.pem".into()),

        upload_directory: args
            .opt_value_from_str("--upload-dir")?
            .unwrap_or("./data/uploads".into()),
    })
}
