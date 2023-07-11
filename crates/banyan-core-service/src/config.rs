use pico_args::Arguments;

#[derive(Debug)]
pub struct Config {
    database: String,
    data_directory: String,
    session_key: String,
}

pub fn parse_arguments() -> Result<Config, pico_args::Error> {
    let mut args = Arguments::from_env();

    Ok(Config {
        database: args
            .opt_value_from_str("--db")?
            .unwrap_or("./server.db".into()),
        data_directory: args
            .opt_value_from_str("--data")?
            .unwrap_or("./data".into()),
        session_key: args
            .opt_value_from_str("--session-key")?
            .unwrap_or("./session-key.pem".into()),
    })
}
