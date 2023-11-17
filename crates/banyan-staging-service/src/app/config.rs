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

    platform_name: String,
    platform_auth_key_path: PathBuf,
    platform_base_url: reqwest::Url,
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

    pub fn parse_cli_arguments() -> Result<Self, Error> {
        if dotenvy::dotenv().is_err() {
            #[cfg(debug_assertions)]
            tracing::warn!("no dot-environment file detected");
        }
        
        let mut args = Arguments::from_env();

        let platform_name = match args.opt_value_from_str("--platform-name")? {
            Some(pn) => pn,
            None => match std::env::var("PLATFORM_NAME") {
                Ok(pn) if !pn.is_empty() => pn,
                _ => "banyan-staging".to_string()
            },
        };

        let platform_auth_key_path: PathBuf = args
            .opt_value_from_str("--auth-key")?
            .unwrap_or("./data/platform-auth.key".into());

        if args.contains("--generate-auth") {
            let mut key_path = platform_auth_key_path.clone();
            tracing::info!("generating new platform key at {key_path:?}");

            let mut file = OpenOptions::new()
                .write(true)
                .create_new(true)
                .open(key_path.clone())
                .map_err(Error::PlatformAuthFailedWrite)?;

            let private_new_key = ES384KeyPair::generate();
            let new_key_pem = private_new_key.to_pem().unwrap();
            file.write_all(new_key_pem.as_bytes())
                .map_err(Error::PlatformAuthFailedWrite)?;

            key_path.set_extension("public");

            let public_new_key = private_new_key.public_key();
            let public_pem = public_new_key.to_pem().unwrap();

            let mut file = std::fs::File::create(key_path.clone()).unwrap();
            file.write_all(public_pem.as_bytes()).unwrap();

            key_path.set_extension("fingerprint");

            let fingerprint = crate::app::state::fingerprint_key(&private_new_key);

            let mut file = std::fs::File::create(key_path).unwrap();
            file.write_all(fingerprint.as_bytes()).unwrap();

            tracing::info!("key generation complete");

            std::process::exit(0);
        }

        let hostname = args
            .opt_value_from_str("--hostname")?
            .unwrap_or("http://127.0.0.1:3002".parse().unwrap());

        let listen_addr = args
            .opt_value_from_str("--listen")?
            .unwrap_or(SocketAddr::new(IpAddr::V6(Ipv6Addr::UNSPECIFIED), 3000));

        let log_level = args
            .opt_value_from_str("--log-level")?
            .unwrap_or(Level::INFO);

        let platform_base_url = args
            .opt_value_from_str("--platform-url")?
            .unwrap_or("http://127.0.0.1:3001".parse().unwrap());

        let db_url = match args.opt_value_from_str("--db-url")? {
            Some(du) => du,
            None => match std::env::var("DATABASE_URL") {
                Ok(du) if !du.is_empty() => du,
                _ => "sqlite://./data/server.db".to_string(),
            },
        };

        let platform_verification_key_path: PathBuf = args
            .opt_value_from_str("--verifier-key")?
            .unwrap_or("./data/platform-verifier.public".into());

        let upload_directory = args
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
