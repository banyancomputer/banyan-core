use std::net::{IpAddr, Ipv6Addr, SocketAddr};
use std::path::PathBuf;

use pico_args::Arguments;
use tracing::Level;
use url::Url;

use crate::app::Version;

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
    platform_public_key_path: PathBuf,
}

impl Config {
    pub fn parse_cli_arguments() -> Result<Self, ConfigError> {
        if dotenvy::dotenv().is_err() {
            #[cfg(debug_assertions)]
            tracing::warn!("no dot-environment file detected");
        }

        let mut cli_args = Arguments::from_env();

        if cli_args.contains("-h") || cli_args.contains("--help") {
            print_help();
            std::process::exit(0);
        }

        if cli_args.contains("-v") || cli_args.contains("--version") {
            print_version();
            std::process::exit(0);
        }

        // Server configuration

        let listen_addr = match cli_args.opt_value_from_str("--listen")? {
            Some(la) => la,
            None => match std::env::var("LISTEN_ADDR") {
                Ok(la) if !la.is_empty() => la.parse().map_err(ConfigError::InvalidListenAddr)?,
                _ => SocketAddr::new(IpAddr::V6(Ipv6Addr::UNSPECIFIED), 3002),
            },
        };

        let log_level = match cli_args.opt_value_from_str("--log-level")? {
            Some(ll) => ll,
            None => match std::env::var("LOG_LEVEL") {
                Ok(ll) if !ll.is_empty() => ll.parse().map_err(ConfigError::InvalidLogLevel)?,
                _ => Level::INFO,
            },
        };

        // Resource configuration

        let database_str = match cli_args.opt_value_from_str("--database-url")? {
            Some(du) => du,
            None => match std::env::var("DATABASE_URL") {
                Ok(du) if !du.is_empty() => du,
                _ => "sqlite://./data/server.db".to_string(),
            },
        };
        let database_url = Url::parse(&database_str).map_err(ConfigError::InvalidDatabaseUrl)?;

        let upload_dir_str = match cli_args.opt_value_from_str("--upload-dir")? {
            Some(path) => path,
            None => match std::env::var("UPLOAD_DIR") {
                Ok(ud) if !ud.is_empty() => ud,
                _ => "./data/uploads".to_string(),
            },
        };
        let upload_directory = PathBuf::from(upload_dir_str);

        // Service identity configuration

        let service_name = match cli_args.opt_value_from_str("--service-name")? {
            Some(sn) => sn,
            None => match std::env::var("SERVICE_NAME") {
                Ok(sn) if !sn.is_empty() => sn,
                _ => "banyan-staging".into(),
            },
        };

        let service_hostname_str = match cli_args.opt_value_from_str("--service-hostname")? {
            Some(sh) => sh,
            None => match std::env::var("SERVICE_HOSTNAME") {
                Ok(sh) if !sh.is_empty() => sh,
                _ => "http://127.0.0.1:3002".to_string()
            },
        };
        let service_hostname = Url::parse(&service_hostname_str).unwrap();

        let service_key_path: PathBuf = match cli_args.opt_value_from_str("--service-key-path")? {
            Some(sk) => sk,
            None => match std::env::var("SERVICE_KEY_PATH") {
                Ok(sk) if !sk.is_empty() => sk.into(),
                _ => "./data/service-key.private".into(),
            },
        };


        // Platform configuration

        let platform_name = match cli_args.opt_value_from_str("--platform-name")? {
            Some(pn) => pn,
            None => match std::env::var("PLATFORM_NAME") {
                Ok(pn) if !pn.is_empty() => pn,
                _ => "banyan-platform".into(),
            },
        };

        let platform_hostname_str = match cli_args.opt_value_from_str("--platform-hostname")? {
            Some(ph) => ph,
            None => match std::env::var("PLATFORM_HOSTNAME") {
                Ok(ph) if !ph.is_empty() => ph,
                _ => "http://127.0.0.1:3001".to_string(),
            },
        };
        let platform_hostname = Url::parse(&platform_hostname_str).unwrap();

        let platform_public_key_path: PathBuf = match cli_args.opt_value_from_str("--platform-public-key-path")? {
            Some(pk) => pk,
            None => match std::env::var("PLATFORM_PUBLIC_KEY_PATH") {
                Ok(pk) if !pk.is_empty() => pk.into(),
                _ => "./data/platform-key.public".into(),
            },
        };

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
            platform_public_key_path,
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

    pub fn platform_public_key_path(&self) -> PathBuf {
        self.platform_public_key_path.clone()
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

    #[error("invalid log level: {0}")]
    InvalidLogLevel(tracing::metadata::ParseLevelError),

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
    println!("    --platform-public-key-path PLATFORM_PUBLIC_KEY_PATH");
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
