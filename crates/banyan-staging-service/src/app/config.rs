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

    platform_auth_key_path: PathBuf,
    grant_verification_key_path: PathBuf,
    upload_directory: PathBuf,
}

impl Config {
    pub fn db_url(&self) -> Option<&str> {
        self.db_url.as_ref().map(String::as_ref)
    }

    pub fn grant_verification_key_path(&self) -> PathBuf {
        self.grant_verification_key_path.clone()
    }

    pub fn listen_addr(&self) -> SocketAddr {
        self.listen_addr.clone()
    }

    pub fn log_level(&self) -> Level {
        self.log_level.clone()
    }

    pub fn parse_cli_arguments() -> Result<Self, Error> {
        let mut args = Arguments::from_env();

        if args.subcommand().unwrap() == Some("generate-auth".to_string()) {
            let key_path: PathBuf = args
                .opt_value_from_str("--path")?
                .unwrap_or("./data/platform-auth.key".into());

            tracing::info!("generating new platform key at {key_path:?}");

            let _new_key = ES384KeyPair::generate();

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

        let platform_auth_key_path: PathBuf = args
            .opt_value_from_str("--platform-key")?
            .unwrap_or("./data/platform-auth.key".into());

        let grant_verification_key_path: PathBuf = args
            .opt_value_from_str("--grant-verifier")?
            .unwrap_or("./data/verifier.pub".into());

        let upload_directory = args
            .opt_value_from_str("--upload-dir")?
            .unwrap_or("./data/uploads".into());

        Ok(Config {
            listen_addr,
            log_level,

            db_url,

            platform_auth_key_path,
            grant_verification_key_path,
            upload_directory,
        })
    }

    pub fn platform_auth_key_path(&self) -> PathBuf {
        self.platform_auth_key_path.clone()
    }

    pub fn upload_directory(&self) -> PathBuf {
        self.upload_directory.clone()
    }
}
