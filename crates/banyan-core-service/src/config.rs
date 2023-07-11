use pico_args::Arguments;
use std::path::PathBuf;

#[derive(Debug)]
pub struct Config {
    pub database: PathBuf,
    pub session_key: PathBuf,
    pub upload_directory: PathBuf,
}

pub fn parse_arguments() -> Result<Config, pico_args::Error> {
    let mut args = Arguments::from_env();

    Ok(Config {
        database: args
            .opt_value_from_str("--db")?
            .unwrap_or("./server.db".into()),

        session_key: args
            .opt_value_from_str("--session-key")?
            .unwrap_or("./session-key.pem".into()),

        upload_directory: args
            .opt_value_from_str("--data")?
            .unwrap_or("./data".into()),
    })
}
