use std::fs::OpenOptions;
use std::io::Write;
use std::net::{IpAddr, Ipv6Addr, SocketAddr};
use std::path::PathBuf;

use jwt_simple::algorithms::ES384KeyPair;
use pico_args::Arguments;
use tracing::Level;
use url::Url;

use crate::app::Error;

#[derive(Debug)]
pub struct Config {
    listen_addr: SocketAddr,
    log_level: Level,

    db_url: String,
    hostname: Url,

    // TODO: rename to `service_name`
    platform_name: String,
    // TODO: rename to `service_key_path`
    platform_auth_key_path: PathBuf,
    // TODO: rename to `platform_hostname`
    platform_base_url: reqwest::Url,
    // TODO: rename to `platform_public_key_path`
    platform_verification_key_path: PathBuf,

    upload_directory: PathBuf,
}

impl Config {
    pub fn db_url(&self) -> &str {
        &self.db_url
    }

    pub fn hostname(&self) -> Url {
        self.hostname.clone()
    }

    pub fn platform_base_url(&self) -> reqwest::Url {
        self.platform_base_url.clone()
    }

    pub fn platform_verification_key_path(&self) -> PathBuf {
        self.platform_verification_key_path.clone()
    }

    pub fn listen_addr(&self) -> SocketAddr {
        self.listen_addr
    }

    pub fn log_level(&self) -> Level {
        self.log_level
    }

    pub fn from_env_and_args() -> Result<Self, Error> {
        if dotenvy::dotenv().is_err() {
            #[cfg(debug_assertions)]
            tracing::warn!("no dot-environment file detected");
        }

        let mut cli_args = Arguments::from_env();

        let platform_name = match cli_args.opt_value_from_str("--platform-name")? {
            Some(pn) => pn,
            None => match std::env::var("PLATFORM_NAME") {
                Ok(pn) if !pn.is_empty() => pn,
                _ => "banyan-staging".to_string(),
            },
        };

        // TODO: change flag name to `--service-key-path`
        let platform_auth_key_path: PathBuf = cli_args
            .opt_value_from_str("--auth-key")?
            .unwrap_or("./data/service-key.private".into());

        let hostname = cli_args
            .opt_value_from_str("--hostname")?
            .unwrap_or("http://127.0.0.1:3002".parse().unwrap());

        let listen_addr = cli_args
            .opt_value_from_str("--listen")?
            .unwrap_or(SocketAddr::new(IpAddr::V6(Ipv6Addr::UNSPECIFIED), 3000));

        let log_level = cli_args
            .opt_value_from_str("--log-level")?
            .unwrap_or(Level::INFO);

        let platform_base_url = cli_args
            .opt_value_from_str("--platform-url")?
            .unwrap_or("http://127.0.0.1:3001".parse().unwrap());

        let db_url = match cli_args.opt_value_from_str("--db-url")? {
            Some(du) => du,
            None => match std::env::var("DATABASE_URL") {
                Ok(du) if !du.is_empty() => du,
                _ => "sqlite://./data/server.db".to_string(),
            },
        };

        // TODO: rename flag to `--platform-public-key-path`
        let platform_verification_key_path: PathBuf = cli_args
            .opt_value_from_str("--verifier-key")?
            .unwrap_or("./data/platform-key.public".into());

        let upload_directory = cli_args
            .opt_value_from_str("--upload-dir")?
            .unwrap_or("./data/uploads".into());

        Ok(Config {
            listen_addr,
            log_level,

            db_url,
            hostname,

            platform_name,
            platform_auth_key_path,
            platform_base_url,
            platform_verification_key_path,

            upload_directory,
        })
    }

    pub fn platform_name(&self) -> &str {
        &self.platform_name
    }

    pub fn platform_auth_key_path(&self) -> PathBuf {
        self.platform_auth_key_path.clone()
    }

    pub fn upload_directory(&self) -> PathBuf {
        self.upload_directory.clone()
    }
}
