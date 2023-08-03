use pico_args::Arguments;
use std::path::PathBuf;

pub fn parse_arguments() -> Result<Config, pico_args::Error> {
    let mut args = Arguments::from_env();

    Ok(Config {
        upload_directory: args
            .opt_value_from_str("--upload-dir")?
            .unwrap_or("./data/uploads".into()),
    })
}

#[derive(Debug)]
pub struct Config {
    upload_directory: PathBuf,
}

impl Config {
    pub fn upload_directory(&self) -> &PathBuf {
        &self.upload_directory
    }
}
