use std::io::Write;
use std::path::PathBuf;
use std::fs::OpenOptions;
use std::net::{IpAddr, Ipv6Addr, SocketAddr};

use pico_args::Arguments;
use tracing::Level;
use url::Url;
use jwt_simple::prelude::*;

use crate::app::Version;
use crate::utils::sha1_fingerprint_publickey;

#[derive(Debug)]
pub struct Config {
    /// The address to bind to
    listen_addr: SocketAddr,
    /// The log level to use
    log_level: Level,

    /// The URL of the database to use (SQLite)
    database_url: Url,
    /// The hostname of this service
    hostname: Url,

    /// The naem fo the service, as registered with the platform
    platform_name: String,
    /// The path to the public key used for authenticating requests from the platform
    platform_public_key_path: PathBuf,
    /// The base URL of the platform
    platform_base_url: Url,

    /// The path to the signing key used for authenticating requests to the platform and other services
    service_key_path: PathBuf,

    /// Where to store uploaded objects
    upload_directory: PathBuf,
}

impl Config {
    pub fn parse_cli_arguments() -> Result<Self, ConfigError> {
        let mut args = Arguments::from_env();

        if args.contains("-h") || args.contains("--help") {
            print_help();
            std::process::exit(0);
        }

        if args.contains("-v") || args.contains("--version") {
            print_version();
            std::process::exit(0);
        }

        let service_key_path: PathBuf = args
            .opt_value_from_str("--serice-key-path")?
            .unwrap_or("./data/serice.key".into());

        if args.contains("--generate-auth") {
            let mut key_path = service_key_path.clone();
            tracing::info!("generating new service key at {key_path:?}");

            let mut file = OpenOptions::new()
                .write(true)
                .create_new(true)
                .open(key_path.clone())
                .map_err(ConfigError::ServiceKeyWriteFailed)?;

            let private_new_key = ES384KeyPair::generate();
            let new_key_pem = private_new_key.to_pem().unwrap();
            file.write_all(new_key_pem.as_bytes())
                .map_err(ConfigError::ServiceKeyWriteFailed)?;

            key_path.set_extension("public");

            let public_new_key = private_new_key.public_key();
            let public_pem = public_new_key
                .to_pem()
                .map_err(ConfigError::ServiceKeyGenFailed)?;

            let mut file = std::fs::File::create(key_path.clone())
                .map_err(ConfigError::ServiceKeyWriteFailed)?;
            file.write_all(public_pem.as_bytes())
                .map_err(ConfigError::ServiceKeyWriteFailed)?;

            key_path.set_extension("fingerprint");

            let fingerprint = sha1_fingerprint_publickey(&public_new_key);

            let mut file =
                std::fs::File::create(key_path).map_err(ConfigError::ServiceKeyWriteFailed)?;
            file.write_all(fingerprint.as_bytes())
                .map_err(ConfigError::ServiceKeyWriteFailed)?;

            tracing::info!("key generation complete");

            std::process::exit(0);
        }

        let listen_addr = args
            .opt_value_from_str("--listen")?
            .unwrap_or(SocketAddr::new(IpAddr::V6(Ipv6Addr::UNSPECIFIED), 3000));

        let log_level = args
            .opt_value_from_str("--log-level")?
            .unwrap_or(Level::INFO);

        let database_str = match args.opt_value_from_str("--database-url")? {
            Some(du) => du,
            None => match std::env::var("DATABASE_URL") {
                Ok(du) if !du.is_empty() => du,
                _ => "sqlite://./data/server.db".to_string(),
            },
        };
        let database_url = Url::parse(&database_str).map_err(ConfigError::InvalidDatabaseUrl)?;

        let hostname = args
            .opt_value_from_str("--hostname")?
            .unwrap_or("http://127.0.0.1:3003".parse().unwrap());

        let platform_name = args
            .opt_value_from_str("--platform-name")?
            .unwrap_or("banyan-storage-provider".into());

        let platform_public_key_path: PathBuf = args
            .opt_value_from_str("--platform-public-key-path")?
            .unwrap_or("./data/platform-public.key".into());
        
        let platform_base_url = args
            .opt_value_from_str("--platform-url")?
            .unwrap_or("http://127.0.0.1:3001".parse().unwrap());

        let upload_dir_str = match args.opt_value_from_str("--upload-dir")? {
            Some(path) => path,
            None => match std::env::var("UPLOAD_DIR") {
                Ok(ud) if !ud.is_empty() => ud,
                _ => "./data/uploads".to_string(),
            },
        };
        let upload_directory = PathBuf::from(upload_dir_str);

        Ok(Config {
            listen_addr,
            log_level,

            database_url,
            hostname,

            platform_name,
            platform_public_key_path,
            platform_base_url,

            service_key_path,

            upload_directory,
        })
    }

    pub fn listen_addr(&self) -> SocketAddr {
        self.listen_addr
    }

    pub fn log_level(&self) -> Level {
        self.log_level
    }

    pub fn database_url(&self) -> Url {
        self.database_url.clone()
    }

    pub fn hostname(&self) -> Url {
        self.hostname.clone()
    }

    pub fn platform_name(&self) -> &str {
        &self.platform_name
    }

    pub fn platform_public_key_path(&self) -> PathBuf {
        self.platform_public_key_path.clone()
    }

    pub fn platform_base_url(&self) -> Url {
        self.platform_base_url.clone()
    }

    pub fn service_key_path(&self) -> PathBuf {
        self.service_key_path.clone()
    }

    pub fn upload_directory(&self) -> PathBuf {
        self.upload_directory.clone()
    }
}

#[derive(Debug, thiserror::Error)]
pub enum ConfigError {
    #[error("failed to read argument from CLI: {0}")]
    ArgumentReadError(#[from] pico_args::Error),

    #[error("invalid database URL: {0}")]
    InvalidDatabaseUrl(url::ParseError),

    #[error("invalid listening address: {0}")]
    InvalidListenAddr(std::net::AddrParseError),

    #[error("service key gen failed: {0}")]
    ServiceKeyGenFailed(jwt_simple::Error),

    #[error("unable to write service key: {0}")]
    ServiceKeyWriteFailed(std::io::Error),
}

fn print_help() {
    println!("Service may be configured using the environment or CLI flags\n");
    println!("  Available options:");
    println!("    -h, --help                    Print this notice and exit");
    println!("    -v, --version                 Display the version of this compiled version");
    println!("                                  and exit\n");
    println!("    --listen, LISTEN_ADDR         Specify the address to bind to, by default");
    println!("                                  this is 127.0.0.1:3001");
    println!("    --mailgun, MAILGUN_KEY        Webhook signature verification key issued by");
    println!("                                  mailgun");
    println!("    --signing-key, SESSION_KEY    Path to the p384 private key used for session");
    println!("                                  key generation and verification");
    println!("    --upload-dir, UPLOAD_DIR      Path used to store uploaded client data\n");
    println!("    --db-url, DATABASE_URL        Configure the url and settings of the sqlite");
    println!("                                  database (default in memory)");
    println!("  Additional Environment Options:");
    println!("    GOOGLE_OAUTH_CLIENT_ID        The client ID associated with this app for");
    println!("                                  performing authentication using Google services.");
    println!("    GOOGLE_OAUTH_CLIENT_SECRET    The client secret paired with the client ID.");
}

fn print_version() {
    let version = Version::new();

    println!(
        "Service version {} built in {} mode with features: {:?}",
        version.version, version.build_profile, version.features
    );
}
