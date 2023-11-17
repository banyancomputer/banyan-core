use std::fs::OpenOptions;
use std::io::Write;
use std::net::{IpAddr, Ipv6Addr, SocketAddr};
use std::path::PathBuf;

use jwt_simple::prelude::*;
use pico_args::Arguments;
use tracing::Level;
use url::Url;

use crate::app::Version;
use crate::utils::fingerprint_key_pair;

#[derive(Debug)]
pub struct Config {
    /// The address to bind to
    listen_addr: SocketAddr,
    /// The log level to use
    log_level: Level,

    /// The URL of the database to use (SQLite)
    database_url: Url,
    /// Where to store uploaded objects
    upload_directory: PathBuf,

    /// The unique name fo the service, as registered with the platform
    service_name: String,
    /// The hostname of this service
    service_hostname: Url,
    /// The path to the signing key used for authenticating requests to the platform and other services
    service_key_path: PathBuf,

    /// The name of the platform
    platform_name: String,
    /// The base URL of the platform
    platform_hostname: Url,
    /// The path to the public key used for authenticating requests from the platform
    platform_verfication_key_path: PathBuf,
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

        // Generate a new signing key for the service if requested

        let service_key_path: PathBuf = args
            .opt_value_from_str("--service-key-path")?
            .unwrap_or("./data/service-key.private".into());

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

            let fingerprint = fingerprint_key_pair(&private_new_key);

            let mut file =
                std::fs::File::create(key_path).map_err(ConfigError::ServiceKeyWriteFailed)?;
            file.write_all(fingerprint.as_bytes())
                .map_err(ConfigError::ServiceKeyWriteFailed)?;

            tracing::info!("key generation complete");

            std::process::exit(0);
        }

        let listen_addr = args
            .opt_value_from_str("--listen")?
            .unwrap_or(SocketAddr::new(IpAddr::V6(Ipv6Addr::UNSPECIFIED), 3003));

        let log_level = args
            .opt_value_from_str("--log-level")?
            .unwrap_or(Level::INFO);

        // Resource configuration

        let database_str = match args.opt_value_from_str("--database-url")? {
            Some(du) => du,
            None => match std::env::var("DATABASE_URL") {
                Ok(du) if !du.is_empty() => du,
                _ => "sqlite://./data/server.db".to_string(),
            },
        };
        let database_url = Url::parse(&database_str).map_err(ConfigError::InvalidDatabaseUrl)?;

        let upload_dir_str = match args.opt_value_from_str("--upload-dir")? {
            Some(path) => path,
            None => match std::env::var("UPLOAD_DIR") {
                Ok(ud) if !ud.is_empty() => ud,
                _ => "./data/uploads".to_string(),
            },
        };
        let upload_directory = PathBuf::from(upload_dir_str);

        // Service identity configuration

        let service_name = args
            .opt_value_from_str("--service-name")?
            .unwrap_or("banyan-storage-provider".into());

        let service_hostname = args
            .opt_value_from_str("--service-hostname")?
            .unwrap_or("http://127.0.0.1:3003".parse().unwrap());

        // Platform configuration

        let platform_name = args
            .opt_value_from_str("--platform-name")?
            .unwrap_or("banyan-platform".into());

        let platform_hostname = args
            .opt_value_from_str("--platform-hostname")?
            .unwrap_or("http://127.0.0.1:3001".parse().unwrap());

        let platform_verfication_key_path: PathBuf = args
            .opt_value_from_str("--platform-key-verifier-path")?
            .unwrap_or("./data/platform-key.public".into());

        Ok(Config {
            listen_addr,
            log_level,

            database_url,
            upload_directory,

            service_name,
            service_hostname,
            service_key_path,

            platform_name,
            platform_hostname,
            platform_verfication_key_path,
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

    pub fn upload_directory(&self) -> PathBuf {
        self.upload_directory.clone()
    }

    pub fn service_name(&self) -> &str {
        &self.service_name
    }

    pub fn service_hostname(&self) -> Url {
        self.service_hostname.clone()
    }

    pub fn service_key_path(&self) -> PathBuf {
        self.service_key_path.clone()
    }

    pub fn platform_name(&self) -> &str {
        &self.platform_name
    }

    pub fn platform_hostname(&self) -> Url {
        self.platform_hostname.clone()
    }

    pub fn platform_verification_key_path(&self) -> PathBuf {
        self.platform_verfication_key_path.clone()
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

    #[error("signing key gen failed: {0}")]
    ServiceKeyGenFailed(jwt_simple::Error),

    #[error("unable to write signing key: {0}")]
    ServiceKeyWriteFailed(std::io::Error),
}

fn print_help() {
    println!("Service may be configured using the following CLI flags\n");
    println!("  Available options:");
    println!("    -h, --help                            Print this notice and exit");
    println!(
        "    -v, --version                         Display the version of this compiled version"
    );
    println!("                                          and exit\n");
    println!("    --generate-auth                       Generate a new signing key for the service. Exits upon key generation if used.\n");
    println!(
        "    --listen LISTEN_ADDR                  Specify the address to bind to, by default"
    );
    println!("                                          this is 127.0.0.1:3001");
    println!("    --log-level LOG_LEVEL                 Specify the log level to use, by default");
    println!("                                          this is INFO\n");
    println!("    --database-url DATABASE_URL           Configure the url for the sqlite database (default ./data/server.db)");
    println!("    --upload-dir UPLOAD_DIR               Path used to store uploaded client data. (default ./data/uploads)\n");
    println!("    --service-name SERVICE_NAME           The unique name of the service, as registered with the platform. (default banyan-storage-provider)");
    println!("    --service-hostname SERVICE_HOSTNAME   The hostname of this service (default http://127.0.0.1:3002)");
    println!("    --service-key-path SERVICE_KEY_PATH   Path to the p384 private key used for service token signing and verification");
    println!("                                          (default ./data/service-key.private)\n");
    println!("    --platform-name PLATFORM_NAME         The name of the platform (default banyan-platform)");
    println!("    --platform-hostname PLATFORM_HOSTNAME The base URL of the platform (default http://127.0.0.1:3001)");
    println!("    --platform-key-verifier-path PLATFORM_KEY_VERIFIER_PATH");
    println!("                                          Path to the public key used for authenticating requests from the platform");
    println!("                                          (default ./data/platform-key.public)\n");
}

fn print_version() {
    let version = Version::new();

    println!(
        "Service version {} built in {} mode with features: {:?}",
        version.version, version.build_profile, version.features
    );
}
