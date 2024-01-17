use std::net::SocketAddr;
use std::path::PathBuf;

use pico_args::Arguments;
use tracing::Level;
use url::Url;

use crate::app::Version;

#[derive(Debug)]
pub struct Config {
    listen_addr: SocketAddr,
    log_level: Level,

    database_url: Url,

    google_client_id: String,
    google_client_secret: String,

    mailgun_signing_key: Option<String>,
    stripe_secret: Option<String>,

    service_name: String,
    service_key_path: PathBuf,
    upload_directory: PathBuf,
}

impl Config {
    pub fn database_url(&self) -> Url {
        self.database_url.clone()
    }

    pub fn from_env_and_args() -> Result<Self, ConfigError> {
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

        let database_str = match cli_args.opt_value_from_str("--db-url")? {
            Some(du) => du,
            None => match std::env::var("DATABASE_URL") {
                Ok(du) if !du.is_empty() => du,
                _ => "sqlite://./data/server.db".to_string(),
            },
        };
        let database_url = Url::parse(&database_str).map_err(ConfigError::InvalidDatabaseUrl)?;

        let mailgun_signing_key = match cli_args.opt_value_from_str("--mailgun")? {
            Some(key) => Some(key),
            None => match std::env::var("MAILGUN_KEY") {
                Ok(mk) if !mk.is_empty() => Some(mk),
                _ => None,
            },
        };

        let service_name = match cli_args.opt_value_from_str("--service-name")? {
            Some(name) => name,
            None => match std::env::var("SERVICE_NAME") {
                Ok(sn) if !sn.is_empty() => sn,
                _ => "banyan-core".to_string(),
            },
        };
        let service_key_str = match cli_args.opt_value_from_str("--service-key")? {
            Some(path) => path,
            None => match std::env::var("SERVICE_KEY") {
                Ok(sk) if !sk.is_empty() => sk,
                _ => "./data/service-key.pem".to_string(),
            },
        };
        let service_key_path = PathBuf::from(service_key_str);

        let stripe_secret = match cli_args.opt_value_from_str("--stripe-key")? {
            Some(key) => Some(key),
            None => match std::env::var("STRIPE_KEY") {
                Ok(sk) if !sk.is_empty() => Some(sk),
                _ => {
                    tracing::warn!(
                        "no stripe key present, purchase actions and verifications will fail"
                    );
                    None
                }
            },
        };

        let upload_dir_str = match cli_args.opt_value_from_str("--upload-dir")? {
            Some(path) => path,
            None => match std::env::var("UPLOAD_DIR") {
                Ok(ud) if !ud.is_empty() => ud,
                _ => "./data/uploads".to_string(),
            },
        };
        let upload_directory = PathBuf::from(upload_dir_str);

        let google_client_id = match std::env::var("GOOGLE_OAUTH_CLIENT_ID") {
            Ok(cid) if !cid.is_empty() => cid,
            _ => return Err(ConfigError::MissingGoogleClientId),
        };
        let google_client_secret = match std::env::var("GOOGLE_OAUTH_CLIENT_SECRET") {
            Ok(cs) if !cs.is_empty() => cs,
            _ => return Err(ConfigError::MissingGoogleClientSecret),
        };

        let listen_str = match cli_args.opt_value_from_str("--listen")? {
            Some(l) => l,
            None => match std::env::var("LISTEN_ADDR") {
                Ok(l) if !l.is_empty() => l,
                _ => "127.0.0.1:3001".to_string(),
            },
        };
        let listen_addr: SocketAddr = listen_str.parse().map_err(ConfigError::InvalidListenAddr)?;

        let log_level = cli_args
            .opt_value_from_str("--log-level")?
            .unwrap_or(Level::INFO);

        Ok(Config {
            listen_addr,
            log_level,

            database_url,

            google_client_id,
            google_client_secret,

            mailgun_signing_key,
            stripe_secret,

            service_name,
            service_key_path,
            upload_directory,
        })
    }

    pub fn google_client_id(&self) -> &str {
        self.google_client_id.as_str()
    }

    pub fn google_client_secret(&self) -> &str {
        self.google_client_secret.as_str()
    }

    pub fn listen_addr(&self) -> SocketAddr {
        self.listen_addr
    }

    pub fn log_level(&self) -> Level {
        self.log_level
    }

    pub fn mailgun_signing_key(&self) -> Option<&str> {
        self.mailgun_signing_key.as_deref()
    }

    pub fn service_name(&self) -> &str {
        self.service_name.as_str()
    }

    pub fn service_key_path(&self) -> PathBuf {
        self.service_key_path.clone()
    }

    pub fn stripe_secret(&self) -> Option<String> {
        self.stripe_secret.clone()
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

    #[error("a google auth client ID needs to be provided")]
    MissingGoogleClientId,

    #[error("a google auth client secret needs to be provided")]
    MissingGoogleClientSecret,

    #[error("a stripe key needs to be provided in production")]
    MissingStripeKey,
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
    println!("    --service-name, SERVICE_NAME  Name of the service, used for identifying");
    println!("                                  the service to other services");
    println!("    --service-key, SERVICE_KEY    Path to the p384 private key used for signing");
    println!("                                  tokens and identifying to other services");
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
