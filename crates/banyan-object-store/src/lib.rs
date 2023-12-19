use std::convert::TryFrom;
use std::ops::Deref;
use std::path::PathBuf;

use object_store::aws::AmazonS3;
pub use object_store::aws::AmazonS3Builder;
use object_store::local::LocalFileSystem;
use object_store::prefix::PrefixStore;
pub use object_store::Error as ObjectStoreError;
use url::Url;

#[derive(Debug, Clone)]
pub enum ObjectStoreConnection {
    Local(PathBuf),
    S3((AmazonS3Builder, Option<ObjectStorePath>)),
}

impl TryFrom<Url> for ObjectStoreConnection {
    type Error = ObjectStoreConnectionError;

    fn try_from(url: Url) -> Result<Self, Self::Error> {
        match url.scheme() {
            // file://<absolute_path>
            "file" => {
                let path = url.to_file_path().map_err(|_| Self::Error::NotFileUrl)?;
                if !path.is_dir() {
                    return Err(Self::Error::NotDirectory(path));
                }

                tracing::info!(path = ?path, "using local backend");

                Ok(Self::Local(path))
            }

            // s3 over http:
            // supported url format:
            // -> http://<access_key_id>:<secret_key>@<host>:<port>?bucket=<bucket_name>&region=<optional_region_name>
            "http" => {
                let access_key_id = url.username();
                let secret_key = url.password().ok_or(Self::Error::MissingS3SecretKey)?;
                let endpoint = format!(
                    "http://{}:{}",
                    url.host_str().ok_or(Self::Error::MissingS3Host)?,
                    url.port().ok_or(Self::Error::MissingS3Port)?
                );

                let query_pairs = url.query_pairs().into_owned();
                let mut maybe_region: Option<String> = None;
                let mut maybe_bucket: Option<String> = None;
                for qp in query_pairs {
                    let (key, val) = qp;
                    match key.as_str() {
                        "bucket" => maybe_bucket = Some(val),
                        "region" => maybe_region = Some(val),
                        _ => {}
                    }
                }

                let bucket = match maybe_bucket {
                    Some(b) => b,
                    None => return Err(Self::Error::MissingS3Bucket),
                };

                // Attempt to parse a prefix from the bucket name
                let mut bucket_parts = bucket.splitn(2, '/');
                let bucket_name = bucket_parts.next().unwrap();
                let maybe_prefix = bucket_parts.next();
                let maybe_prefix_path = maybe_prefix.map(ObjectStorePath::from);

                let region = match maybe_region {
                    Some(r) => r,
                    // MinIo ignores the configured region, so "default" is fine here
                    // for our purposes
                    None => "default".to_string(),
                };

                tracing::info!(
                    scheme = ?url.scheme(),
                    endpoint = ?endpoint,
                    region = ?region,
                    bucket = ?bucket_name,
                    prefix = ?maybe_prefix,
                    "using S3 backend"
                );

                let builder = AmazonS3Builder::new()
                    .with_access_key_id(access_key_id)
                    .with_secret_access_key(secret_key)
                    .with_endpoint(endpoint)
                    .with_region(region)
                    .with_bucket_name(bucket_name)
                    .with_allow_http(true);

                Ok(Self::S3((builder, maybe_prefix_path)))
            }

            // s3 over https:
            // supported url formats:
            // -> https://<access_key_id>:<secret_key>@<host>?bucket=<bucket>&region=<optional_region>
            // -> s3://<access_key_id>:<secret_key>@<host>?bucket=<bucket>&region=<optional_region>
            "https" | "s3" => {
                let access_key_id = url.username();
                let secret_key = url.password().ok_or(Self::Error::MissingS3SecretKey)?;
                let endpoint = format!(
                    "https://{}",
                    url.host_str().ok_or(Self::Error::MissingS3Host)?,
                );

                let query_pairs = url.query_pairs().into_owned();
                let mut maybe_region: Option<String> = None;
                let mut maybe_bucket: Option<String> = None;
                for qp in query_pairs {
                    let (key, val) = qp;
                    match key.as_str() {
                        "bucket" => maybe_bucket = Some(val),
                        "region" => maybe_region = Some(val),
                        _ => {}
                    }
                }

                let bucket = match maybe_bucket {
                    Some(b) => b,
                    None => return Err(Self::Error::MissingS3Bucket),
                };

                // Attempt to parse a prefix from the bucket name
                let mut bucket_parts = bucket.splitn(2, '/');
                let bucket_name = bucket_parts.next().unwrap();
                let maybe_prefix = bucket_parts.next();
                let maybe_prefix_path = maybe_prefix.map(ObjectStorePath::from);

                let region = match maybe_region {
                    Some(r) => r,
                    // MinIo ignores the configured region, so "default" is fine here
                    // for our purposes
                    None => "default".to_string(),
                };

                tracing::info!(
                    scheme = ?url.scheme(),
                    endpoint = ?endpoint,
                    region = ?region,
                    bucket_name = ?bucket_name,
                    prefix = ?maybe_prefix,
                    "using secure S3 backend"
                );

                let builder = AmazonS3Builder::new()
                    .with_access_key_id(access_key_id)
                    .with_secret_access_key(secret_key)
                    .with_endpoint(endpoint)
                    .with_region(region)
                    .with_bucket_name(bucket_name);

                Ok(Self::S3((builder, maybe_prefix_path)))
            }
            // unknown scheme
            scheme => Err(Self::Error::UnknownScheme(scheme.to_string())),
        }
    }
}

pub enum ObjectStore {
    /// An object store against a local filesystem
    Local(LocalFileSystem),
    /// An object store against S3 -- wrapped in a Prefix
    S3(PrefixStore<AmazonS3>),
}

pub type ObjectStorePath = object_store::path::Path;

impl Deref for ObjectStore {
    type Target = dyn object_store::ObjectStore;

    fn deref(&self) -> &Self::Target {
        match self {
            Self::Local(store) => store,
            Self::S3(store) => store,
        }
    }
}

impl ObjectStore {
    pub fn new(connection: &ObjectStoreConnection) -> Result<Self, ObjectStoreError> {
        match connection {
            ObjectStoreConnection::Local(path) => {
                let store = LocalFileSystem::new_with_prefix(path)?;
                Ok(Self::Local(store))
            }
            ObjectStoreConnection::S3(builder) => {
                let (builder, maybe_prefix) = builder;
                let store = builder.clone().build()?;
                let prefix_path = match maybe_prefix {
                    Some(prefix) => prefix.clone(),
                    None => ObjectStorePath::from(""),
                };
                Ok(Self::S3(PrefixStore::new(store, prefix_path)))
            }
        }
    }
}

#[derive(Debug, thiserror::Error)]
pub enum ObjectStoreConnectionError {
    #[error("unable to parse URL: {0}")]
    UrlParse(#[from] url::ParseError),
    #[error("not a file URL")]
    NotFileUrl,
    #[error("not a directory: {0}")]
    NotDirectory(PathBuf),
    #[error("missing secret key")]
    MissingS3SecretKey,
    #[error("missing s3 host")]
    MissingS3Host,
    #[error("missing s3 port")]
    MissingS3Port,
    #[error("missing s3 region")]
    MissingS3Region,
    #[error("missing s3 bucket")]
    MissingS3Bucket,
    #[error("unknow scheme: {0}")]
    UnknownScheme(String),
    #[error("host not recognized: {0}")]
    HostNotRecognized(String),
}

#[cfg(test)]
mod test {
    use object_store::aws::AmazonS3ConfigKey;

    use super::*;

    #[test]
    fn test_parse_local_url() {
        let url = Url::parse("file:///tmp").unwrap();
        let connection = ObjectStoreConnection::try_from(url).unwrap();
        match connection {
            ObjectStoreConnection::Local(path) => {
                assert_eq!(path, PathBuf::from("/tmp"));
            }
            _ => panic!("expected local connection"),
        }
    }

    #[test]
    fn test_parse_local_url_bad_path() {
        let url = Url::parse("file:///tmp/fake.txt").unwrap();
        let connection = ObjectStoreConnection::try_from(url);
        assert!(connection.is_err());
    }

    #[test]
    fn test_parse_http_url() {
        let url =
            Url::parse("http://access_key_id:secret_key@localhost:9000?bucket=bucket").unwrap();
        let connection = ObjectStoreConnection::try_from(url).unwrap();
        match connection {
            ObjectStoreConnection::S3((builder, _)) => {
                assert_eq!(
                    builder.get_config_value(&AmazonS3ConfigKey::AccessKeyId),
                    Some("access_key_id".to_string())
                );
                assert_eq!(
                    builder.get_config_value(&AmazonS3ConfigKey::SecretAccessKey),
                    Some("secret_key".to_string())
                );
                assert_eq!(
                    builder.get_config_value(&AmazonS3ConfigKey::Endpoint),
                    Some("http://localhost:9000".to_string())
                );
                assert_eq!(
                    builder.get_config_value(&AmazonS3ConfigKey::Region),
                    Some("default".to_string())
                );
                assert_eq!(
                    builder.get_config_value(&AmazonS3ConfigKey::Bucket),
                    Some("bucket".to_string())
                );
            }
            _ => panic!("expected S3 connection"),
        }
    }

    #[test]
    fn test_parse_http_url_with_region() {
        let url =
            Url::parse("http://access_key_id:secret_key@localhost:9000?bucket=bucket").unwrap();
        let connection = ObjectStoreConnection::try_from(url).unwrap();
        match connection {
            ObjectStoreConnection::S3((builder, _)) => {
                assert_eq!(
                    builder.get_config_value(&AmazonS3ConfigKey::AccessKeyId),
                    Some("access_key_id".to_string())
                );
                assert_eq!(
                    builder.get_config_value(&AmazonS3ConfigKey::SecretAccessKey),
                    Some("secret_key".to_string())
                );
                assert_eq!(
                    builder.get_config_value(&AmazonS3ConfigKey::Endpoint),
                    Some("http://localhost:9000".to_string())
                );
                assert_eq!(
                    builder.get_config_value(&AmazonS3ConfigKey::Region),
                    Some("default".to_string())
                );
                assert_eq!(
                    builder.get_config_value(&AmazonS3ConfigKey::Bucket),
                    Some("bucket".to_string())
                );
            }
            _ => panic!("expected S3 connection"),
        }
    }
    #[test]
    fn test_parse_http_url_with_path() {
        let url =
            Url::parse("http://access_key_id:secret_key@localhost:9000?bucket=bucket/path/to/dir")
                .unwrap();
        let connection = ObjectStoreConnection::try_from(url).unwrap();
        match connection {
            ObjectStoreConnection::S3((builder, path)) => {
                assert_eq!(
                    builder.get_config_value(&AmazonS3ConfigKey::AccessKeyId),
                    Some("access_key_id".to_string())
                );
                assert_eq!(
                    builder.get_config_value(&AmazonS3ConfigKey::SecretAccessKey),
                    Some("secret_key".to_string())
                );
                assert_eq!(
                    builder.get_config_value(&AmazonS3ConfigKey::Endpoint),
                    Some("http://localhost:9000".to_string())
                );
                assert_eq!(
                    builder.get_config_value(&AmazonS3ConfigKey::Region),
                    Some("default".to_string())
                );
                assert_eq!(
                    builder.get_config_value(&AmazonS3ConfigKey::Bucket),
                    Some("bucket".to_string())
                );
                assert_eq!(path, Some(ObjectStorePath::from("path/to/dir")));
            }
            _ => panic!("expected S3 connection"),
        }
    }

    #[test]
    fn test_parse_https_url() {
        let url =
            Url::parse("https://access_key_id:secret_key@awesome.host.org?bucket=bucket").unwrap();
        let connection = ObjectStoreConnection::try_from(url).unwrap();
        match connection {
            ObjectStoreConnection::S3((builder, _)) => {
                assert_eq!(
                    builder.get_config_value(&AmazonS3ConfigKey::AccessKeyId),
                    Some("access_key_id".to_string())
                );
                assert_eq!(
                    builder.get_config_value(&AmazonS3ConfigKey::SecretAccessKey),
                    Some("secret_key".to_string())
                );
                assert_eq!(
                    builder.get_config_value(&AmazonS3ConfigKey::Endpoint),
                    Some("https://awesome.host.org".to_string())
                );
                assert_eq!(
                    builder.get_config_value(&AmazonS3ConfigKey::Region),
                    Some("default".to_string())
                );
                assert_eq!(
                    builder.get_config_value(&AmazonS3ConfigKey::Bucket),
                    Some("bucket".to_string())
                );
            }
            _ => panic!("expected S3 connection"),
        }
    }

    #[test]
    fn test_parse_https_url_with_region() {
        let url = Url::parse(
            "https://access_key_id:secret_key@awesome.host.org?bucket=bucket&region=region",
        )
        .unwrap();
        let connection = ObjectStoreConnection::try_from(url).unwrap();
        match connection {
            ObjectStoreConnection::S3((builder, _)) => {
                assert_eq!(
                    builder.get_config_value(&AmazonS3ConfigKey::AccessKeyId),
                    Some("access_key_id".to_string())
                );
                assert_eq!(
                    builder.get_config_value(&AmazonS3ConfigKey::SecretAccessKey),
                    Some("secret_key".to_string())
                );
                assert_eq!(
                    builder.get_config_value(&AmazonS3ConfigKey::Endpoint),
                    Some("https://awesome.host.org".to_string())
                );
                assert_eq!(
                    builder.get_config_value(&AmazonS3ConfigKey::Region),
                    Some("region".to_string())
                );
                assert_eq!(
                    builder.get_config_value(&AmazonS3ConfigKey::Bucket),
                    Some("bucket".to_string())
                );
            }
            _ => panic!("expected S3 connection"),
        }
    }

    #[test]
    fn test_parse_https_url_with_path() {
        let url = Url::parse(
            "https://access_key_id:secret_key@awesome.host.org?bucket=bucket/path/to/dir",
        )
        .unwrap();
        let connection = ObjectStoreConnection::try_from(url).unwrap();
        match connection {
            ObjectStoreConnection::S3((builder, path)) => {
                assert_eq!(
                    builder.get_config_value(&AmazonS3ConfigKey::AccessKeyId),
                    Some("access_key_id".to_string())
                );
                assert_eq!(
                    builder.get_config_value(&AmazonS3ConfigKey::SecretAccessKey),
                    Some("secret_key".to_string())
                );
                assert_eq!(
                    builder.get_config_value(&AmazonS3ConfigKey::Endpoint),
                    Some("https://awesome.host.org".to_string())
                );
                assert_eq!(
                    builder.get_config_value(&AmazonS3ConfigKey::Region),
                    Some("default".to_string())
                );
                assert_eq!(
                    builder.get_config_value(&AmazonS3ConfigKey::Bucket),
                    Some("bucket".to_string())
                );
                assert_eq!(path, Some(ObjectStorePath::from("path/to/dir")));
            }
            _ => panic!("expected S3 connection"),
        }
    }

    #[test]
    fn test_parse_s3_url() {
        let url =
            Url::parse("s3://access_key_id:secret_key@awesome.host.org?bucket=bucket").unwrap();
        let connection = ObjectStoreConnection::try_from(url).unwrap();
        match connection {
            ObjectStoreConnection::S3((builder, _)) => {
                assert_eq!(
                    builder.get_config_value(&AmazonS3ConfigKey::AccessKeyId),
                    Some("access_key_id".to_string())
                );
                assert_eq!(
                    builder.get_config_value(&AmazonS3ConfigKey::SecretAccessKey),
                    Some("secret_key".to_string())
                );
                assert_eq!(
                    builder.get_config_value(&AmazonS3ConfigKey::Endpoint),
                    Some("https://awesome.host.org".to_string())
                );
                assert_eq!(
                    builder.get_config_value(&AmazonS3ConfigKey::Region),
                    Some("default".to_string())
                );
                assert_eq!(
                    builder.get_config_value(&AmazonS3ConfigKey::Bucket),
                    Some("bucket".to_string())
                );
            }
            _ => panic!("expected S3 connection"),
        }
    }

    #[test]
    fn test_parse_s3_url_with_region() {
        let url = Url::parse(
            "s3://access_key_id:secret_key@awesome.host.org?bucket=bucket&region=region",
        )
        .unwrap();
        let connection = ObjectStoreConnection::try_from(url).unwrap();
        match connection {
            ObjectStoreConnection::S3((builder, _)) => {
                assert_eq!(
                    builder.get_config_value(&AmazonS3ConfigKey::AccessKeyId),
                    Some("access_key_id".to_string())
                );
                assert_eq!(
                    builder.get_config_value(&AmazonS3ConfigKey::SecretAccessKey),
                    Some("secret_key".to_string())
                );
                assert_eq!(
                    builder.get_config_value(&AmazonS3ConfigKey::Endpoint),
                    Some("https://awesome.host.org".to_string())
                );
                assert_eq!(
                    builder.get_config_value(&AmazonS3ConfigKey::Region),
                    Some("region".to_string())
                );
                assert_eq!(
                    builder.get_config_value(&AmazonS3ConfigKey::Bucket),
                    Some("bucket".to_string())
                );
            }
            _ => panic!("expected S3 connection"),
        }
    }

    #[test]
    fn test_parse_s3_url_with_path() {
        let url =
            Url::parse("s3://access_key_id:secret_key@awesome.host.org?bucket=bucket/path/to/dir")
                .unwrap();
        let connection = ObjectStoreConnection::try_from(url).unwrap();
        match connection {
            ObjectStoreConnection::S3((builder, path)) => {
                assert_eq!(
                    builder.get_config_value(&AmazonS3ConfigKey::AccessKeyId),
                    Some("access_key_id".to_string())
                );
                assert_eq!(
                    builder.get_config_value(&AmazonS3ConfigKey::SecretAccessKey),
                    Some("secret_key".to_string())
                );
                assert_eq!(
                    builder.get_config_value(&AmazonS3ConfigKey::Endpoint),
                    Some("https://awesome.host.org".to_string())
                );
                assert_eq!(
                    builder.get_config_value(&AmazonS3ConfigKey::Region),
                    Some("default".to_string())
                );
                assert_eq!(
                    builder.get_config_value(&AmazonS3ConfigKey::Bucket),
                    Some("bucket".to_string())
                );
                assert_eq!(path, Some(ObjectStorePath::from("path/to/dir")));
            }
            _ => panic!("expected S3 connection"),
        }
    }
}
