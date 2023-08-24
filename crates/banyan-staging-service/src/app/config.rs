use std::net::{IpAddr, Ipv6Addr, SocketAddr};
use std::path::PathBuf;

use jwt_simple::algorithms::ES384KeyPair;
use pico_args::Arguments;
use tracing::Level;

use crate::app::Error;

#[derive(Debug)]
pub struct Config {
    listen_addr: SocketAddr,
    log_level: Level,

    db_url: Option<String>,

    jwt_key_path: PathBuf,
    upload_directory: PathBuf,
}

impl Config {
    pub fn db_url(&self) -> Option<&str> {
        self.db_url.as_ref().map(String::as_ref)
    }

    pub fn listen_addr(&self) -> &SocketAddr {
        &self.listen_addr
    }

    pub fn log_level(&self) -> Level {
        self.log_level.clone()
    }

    pub fn parse_cli_arguments() -> Result<Self, Error> {
        let mut args = Arguments::from_env();

        if args.subcommand().unwrap() == Some("generate".to_string()) {
            let jwt_key_path: PathBuf = args
                .opt_value_from_str("--session-key")?
                .unwrap_or("./data/session.key".into());

            tracing::info!("generating new session key at {jwt_key_path:?}");

            let _new_session_key = ES384KeyPair::generate();

            // todo: write the session key out to the path
            // todo: make sure this doesn't overwrite an existing key

            tracing::info!("key generation complete");

            std::process::exit(0);
        }

        let listen_addr = args
            .opt_value_from_str("--listen")?
            .unwrap_or(SocketAddr::new(IpAddr::V6(Ipv6Addr::UNSPECIFIED), 3000));

        let log_level = args
            .opt_value_from_str("--log-level")?
            .unwrap_or(Level::INFO);

        let db_url = args
            .opt_value_from_str("--db-url")?;

        let jwt_key_path: PathBuf = args
            .opt_value_from_str("--session-key")?
            .unwrap_or("./data/session.key".into());

        let upload_directory = args
            .opt_value_from_str("--upload-dir")?
            .unwrap_or("./data/uploads".into());

        Ok(Config {
            listen_addr,
            log_level,

            db_url,

            jwt_key_path,
            upload_directory,
        })
    }

    pub fn jwt_key_path(&self) -> &PathBuf {
        &self.jwt_key_path
    }

    pub fn upload_directory(&self) -> &PathBuf {
        &self.upload_directory
    }
}
