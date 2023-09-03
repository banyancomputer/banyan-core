use std::fs::OpenOptions;
use std::io::Write;
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
    platform_verification_key_path: PathBuf,
    upload_directory: PathBuf,
}

impl Config {
    pub fn db_url(&self) -> Option<&str> {
        self.db_url.as_ref().map(String::as_ref)
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
        let mut args = Arguments::from_env();

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
        }

        let listen_addr = args
            .opt_value_from_str("--listen")?
            .unwrap_or(SocketAddr::new(IpAddr::V6(Ipv6Addr::UNSPECIFIED), 3000));

        let log_level = args
            .opt_value_from_str("--log-level")?
            .unwrap_or(Level::INFO);

        let db_url = args.opt_value_from_str("--db-url")?;

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

            platform_auth_key_path,
            platform_verification_key_path,
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
