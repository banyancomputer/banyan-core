mod block_location;
mod bucket;
mod bucket_key;
mod bucket_type;
mod deal;
mod deal_state;
mod email_message;
mod email_message_state;
mod escrowed_device;
mod metadata;
mod metadata_state;
mod partial_metadata_with_snapshot;
mod pending_expiration;
mod price_units;
mod snapshot;
mod snapshot_segments;
mod snapshot_state;
mod storage_class;
mod storage_grant;
mod storage_host;
mod stripe_product;
mod subscription;
mod tax_class;
mod user;

pub use block_location::MinimalBlockLocation;
pub use bucket::Bucket;
pub use bucket_key::BucketKey;
pub use bucket_type::BucketType;
pub use deal::Deal;
pub use deal_state::DealState;
#[allow(unused)]
pub use email_message::EmailMessage;
pub use email_message_state::EmailMessageState;
pub use escrowed_device::EscrowedDevice;
pub use metadata::{Metadata, NewMetadata};
pub use metadata_state::MetadataState;
pub use partial_metadata_with_snapshot::PartialMetadataWithSnapshot;
pub use pending_expiration::PendingExpiration;
pub use price_units::PriceUnits;
pub use snapshot::Snapshot;
pub use snapshot_segments::SnapshotSegment;
pub use snapshot_state::SnapshotState;
pub use storage_class::StorageClass;
pub use storage_grant::NewStorageGrant;
pub use storage_host::{SelectedStorageHost, StorageHost, UserStorageReport};
pub use stripe_product::StripeProduct;
pub use subscription::{NewSubscription, Subscription};
pub use tax_class::TaxClass;
pub use user::User;

/// Something about sqlx's type detection fails on complex queries such as the result of COALESCE
/// that forces it to assume the result is a 32-bit signed integer, and it seems to ignore the sqlx
/// specific type overrides. To get 64-bit values out of sqlx exclusively in these cases, we need
/// an explicit wrapping type that we can then extract the desired value from.
///
/// note(sstelfox): I consider this a bug in sqlx but the maintainers didn't want to accept it as
/// such recommending this workaround. See launchbadge/sqlx#2814.
#[derive(sqlx::FromRow)]
pub struct ExplicitBigInt {
    big_int: i64,
}
