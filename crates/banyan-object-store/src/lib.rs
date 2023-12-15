use std::convert::TryFrom;
use std::ops::Deref;
use std::path::PathBuf;

use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use axum::Json;
use itertools::Itertools;
use object_store::aws::AmazonS3;
pub use object_store::aws::AmazonS3Builder;
use object_store::local::LocalFileSystem;
use url::Url;

#[derive(Debug, Clone)]
pub enum ObjectStoreConnection {
    Local(PathBuf),
    S3(AmazonS3Builder),
}

impl TryFrom<Url> for ObjectStoreConnection {
    type Error = ObjectStoreConnectionError;

    fn try_from(url: Url) -> Result<Self, Self::Error> {
        match url.scheme() {
            // file://<path>
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
            // -> http://<access_key_id>:<secret_key>@<host>:<port>/<region>/<bucket>/<optional_path>
            "http" => {
                let access_key_id = url.username();
                let secret_key = url.password().ok_or(Self::Error::MissingS3SecretKey)?;
                let endpoint = format!(
                    "http://{}:{}",
                    url.host_str().ok_or(Self::Error::MissingS3Host)?,
                    url.port().ok_or(Self::Error::MissingS3Port)?
                );

                let mut url_path = url.path().split('/').collect::<Vec<&str>>().into_iter();
                // Strip off the leading slash
                url_path.next();
                // Split off the first path segment as the region
                let region = url_path.next().ok_or(Self::Error::MissingS3Region)?;
                // Collect the rest of the path segments as the bucket name
                let bucket_name = url_path.join("/");

                tracing::info!(
                    scheme = ?url.scheme(),
                    endpoint = ?endpoint,
                    region = ?region,
                    bucket_name = ?bucket_name,
                    "using S3 backend"
                );

                let builder = AmazonS3Builder::new()
                    .with_access_key_id(access_key_id)
                    .with_secret_access_key(secret_key)
                    .with_endpoint(endpoint)
                    .with_region(region)
                    .with_bucket_name(bucket_name)
                    .with_allow_http(true);

                Ok(Self::S3(builder))
            }

            // s3 over https:
            // supported url format:
            // https://<access_key_id>:<secret_key>@s3.<region>.<host>/<bucket>/<optional_path>
            "https" => {
                let access_key_id = url.username();
                let secret_key = url.password().ok_or(Self::Error::MissingS3SecretKey)?;
                let host = url.host_str().ok_or(Self::Error::MissingS3Host)?;

                match host.splitn(3, '.').collect_tuple() {
                    Some(("s3", region, host)) => {
                        let endpoint = format!("https://{}", host);

                        let mut url_path = url.path().split('/').collect::<Vec<&str>>().into_iter();
                        // Strip off the leading slash
                        url_path.next();
                        // Collect the rest of the path segments as the bucket name
                        let bucket_name = url_path.join("/");

                        tracing::info!(
                            scheme = ?url.scheme(),
                            endpoint = ?endpoint,
                            region = ?region,
                            bucket_name = ?bucket_name,
                            "using secure S3 backend"
                        );

                        let builder = AmazonS3Builder::new()
                            .with_access_key_id(access_key_id)
                            .with_secret_access_key(secret_key)
                            .with_endpoint(endpoint)
                            .with_region(region)
                            .with_bucket_name(bucket_name);
                        Ok(Self::S3(builder))
                    }
                    _ => Err(Self::Error::HostNotRecognized(host.to_string())),
                }
            }
            // unknown scheme
            scheme => Err(Self::Error::UnknownScheme(scheme.to_string())),
        }
    }
}

pub enum ObjectStore {
    Local(LocalFileSystem),
    S3(AmazonS3),
}

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
                let store = builder.clone().build()?;
                Ok(Self::S3(store))
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

#[derive(Debug, thiserror::Error)]
pub enum ObjectStoreError {
    #[error("unable to access upload store: {0}")]
    ObjectStore(#[from] object_store::Error),
}

impl IntoResponse for ObjectStoreError {
    fn into_response(self) -> Response {
        match self {
            ObjectStoreError::ObjectStore(err) => {
                tracing::error!(err = ?err, "configured object store is inaccessible");
                let err_msg = serde_json::json!({ "msg": "a backend service issue occurred" });
                (StatusCode::INTERNAL_SERVER_ERROR, Json(err_msg)).into_response()
            }
        }
    }
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
            Url::parse("http://access_key_id:secret_key@localhost:9000/us-east-1/bucket").unwrap();
        let connection = ObjectStoreConnection::try_from(url).unwrap();
        match connection {
            ObjectStoreConnection::S3(builder) => {
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
                    Some("us-east-1".to_string())
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
        let url = Url::parse(
            "http://access_key_id:secret_key@localhost:9000/us-east-1/bucket/path/to/dir",
        )
        .unwrap();
        let connection = ObjectStoreConnection::try_from(url).unwrap();
        match connection {
            ObjectStoreConnection::S3(builder) => {
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
                    Some("us-east-1".to_string())
                );
                assert_eq!(
                    builder.get_config_value(&AmazonS3ConfigKey::Bucket),
                    Some("bucket/path/to/dir".to_string())
                );
            }
            _ => panic!("expected S3 connection"),
        }
    }

    #[test]
    fn test_parse_https_url() {
        let url =
            Url::parse("https://access_key_id:secret_key@s3.us-east-1.awesome.host.org/bucket")
                .unwrap();
        let connection = ObjectStoreConnection::try_from(url).unwrap();
        match connection {
            ObjectStoreConnection::S3(builder) => {
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
                    Some("us-east-1".to_string())
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
            "https://access_key_id:secret_key@s3.us-east-1.awesome.host.org/bucket/path/to/dir",
        )
        .unwrap();
        let connection = ObjectStoreConnection::try_from(url).unwrap();
        match connection {
            ObjectStoreConnection::S3(builder) => {
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
                    Some("us-east-1".to_string())
                );
                assert_eq!(
                    builder.get_config_value(&AmazonS3ConfigKey::Bucket),
                    Some("bucket/path/to/dir".to_string())
                );
            }
            _ => panic!("expected S3 connection"),
        }
    }
}
