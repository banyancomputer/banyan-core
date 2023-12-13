mod bucket;
mod bucket_key;
mod bucket_type;
mod email_message;
mod email_message_state;
mod escrowed_device;
mod metadata;
mod metadata_state;
mod partial_metadata_with_snapshot;
mod snapshot;
mod storage_class;
mod storage_host;
mod user;

pub use bucket::Bucket;
pub use bucket_key::BucketKey;
pub use bucket_type::BucketType;
#[allow(unused)]
pub use email_message::EmailMessage;
pub use email_message_state::EmailMessageState;
pub use escrowed_device::EscrowedDevice;
pub use metadata::{Metadata, NewMetadata};
pub use metadata_state::MetadataState;
pub use partial_metadata_with_snapshot::PartialMetadataWithSnapshot;
pub use snapshot::Snapshot;
pub use storage_class::StorageClass;
pub use storage_host::{StorageHost, UserStorageReport};
pub use user::User;

/// Something about sqlx's type detection fails on complex queries such as the result of COALESCE
/// that forces it to assume the result is a 32-bit signed integer, and it seems to ignore the sqlx
/// specific type overrides. To get 64-bit values out of sqlx exclusively in these cases, we need
/// an explicit wrapping type that we can then extract the desired value from.
///
/// note(sstelfox): I consider this a bug in sqlx but the maintainers didn't want to accept it as
/// such recommending this workaround.
#[derive(sqlx::FromRow)]
pub struct ExplicitBigInt {
    big_int: i64,
}
