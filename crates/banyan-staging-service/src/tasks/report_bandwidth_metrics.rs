use std::collections::HashMap;

use async_trait::async_trait;
use banyan_task::{CurrentTask, RecurringTask, RecurringTaskError, TaskLike};
use serde::{Deserialize, Serialize};
use time::error::ComponentRange;
use time::{Duration, OffsetDateTime};
use url::Url;

use crate::app::AppState;
use crate::clients::{CoreServiceClient, CoreServiceError, MeterTrafficRequest};
use crate::database::models::BandwidthMetrics;
use crate::database::Database;

pub type ReportBandwidthMetricsTaskContext = AppState;

#[non_exhaustive]
#[derive(Debug, thiserror::Error)]
pub enum ReportBandwidthMetricsTaskError {
    #[error("sql error: {0}")]
    Database(#[from] sqlx::Error),

    #[error("reqwest error: {0}")]
    Reqwest(#[from] reqwest::Error),

    #[error("jwt error: {0}")]
    Jwt(#[from] jwt_simple::Error),

    #[error("http error: {0} response from {1}")]
    Http(http::StatusCode, Url),

    #[error("could not calculate end slot: {0}")]
    EndSlotParsingError(#[from] ComponentRange),

    #[error("core service error: {0}")]
    CoreServiceError(#[from] CoreServiceError),
}

#[derive(Default, Deserialize, Serialize)]
pub struct ReportBandwidthMetricsTask;

#[async_trait]
impl TaskLike for ReportBandwidthMetricsTask {
    const TASK_NAME: &'static str = "report_bandwidth_metrics_task";

    type Error = ReportBandwidthMetricsTaskError;
    type Context = ReportBandwidthMetricsTaskContext;

    async fn run(&self, _task: CurrentTask, ctx: Self::Context) -> Result<(), Self::Error> {
        let conn = ctx.database();
        let slot_end = round_to_previous_hour(OffsetDateTime::now_utc())?;
        let users_metrics = bandwidth_metrics_until(&conn, slot_end).await?;
        let partitioned_metrics = partition_bandwidth_metrics_by_hour_and_user(users_metrics)?;
        let client = CoreServiceClient::new(
            ctx.secrets().service_signing_key(),
            ctx.service_name(),
            ctx.platform_name(),
            ctx.platform_hostname(),
        )?;

        for metrics in partitioned_metrics {
            let meter_traffic_request = MeterTrafficRequest {
                user_id: &metrics.user_id,
                ingress: metrics.ingress,
                egress: metrics.egress,
                slot: metrics.created_at.unix_timestamp(),
            };
            if let Err(err) = client.report_user_bandwidth(meter_traffic_request).await {
                tracing::error!(
                    "could not report metrics for user {} err {}",
                    &metrics.user_id,
                    err
                );
                continue;
            }

            if let Err(err) = delete_bandwidth_metrics_until(&conn, slot_end).await {
                tracing::error!(
                    "could not delete the bandwidth end_slot {}, err: {}",
                    slot_end,
                    err
                )
            };
        }

        Ok(())
    }
}

impl RecurringTask for ReportBandwidthMetricsTask {
    fn next_schedule(&self) -> Result<Option<OffsetDateTime>, RecurringTaskError> {
        OffsetDateTime::now_utc()
            .checked_add(Duration::minutes(20))
            .ok_or(RecurringTaskError::DateTimeAddition)
            .map(Some)
    }
}

fn round_to_previous_hour(start_time: OffsetDateTime) -> Result<OffsetDateTime, ComponentRange> {
    start_time
        .replace_minute(0)
        .and_then(|t| t.replace_second(0))
        .and_then(|t| t.replace_nanosecond(0))
}

fn round_to_next_hour(start_time: OffsetDateTime) -> Result<OffsetDateTime, ComponentRange> {
    let rounded_time = start_time
        .replace_minute(0)
        .and_then(|t| t.replace_second(0))
        .and_then(|t| t.replace_nanosecond(0));

    match rounded_time {
        Ok(time) => {
            if time == start_time {
                Ok(time)
            } else {
                Ok(time + Duration::hours(1))
            }
        }
        Err(e) => Err(e),
    }
}

async fn delete_bandwidth_metrics_until(
    db: &Database,
    end_time: OffsetDateTime,
) -> Result<u64, sqlx::Error> {
    let rows_deleted = sqlx::query!(
        r#"DELETE FROM bandwidth_metrics WHERE created_at <= $1"#,
        end_time,
    )
    .execute(db)
    .await?
    .rows_affected();

    Ok(rows_deleted)
}

async fn bandwidth_metrics_until(
    db: &Database,
    end_time: OffsetDateTime,
) -> Result<Vec<BandwidthMetrics>, sqlx::Error> {
    let res = sqlx::query_as!(
        BandwidthMetrics,
        "SELECT *  FROM bandwidth_metrics WHERE created_at <= $1",
        end_time,
    )
    .fetch_all(db)
    .await?;

    Ok(res)
}

fn partition_bandwidth_metrics_by_hour_and_user(
    metrics: Vec<BandwidthMetrics>,
) -> Result<Vec<BandwidthMetrics>, ComponentRange> {
    let mut partitioned_metrics: HashMap<(OffsetDateTime, String), BandwidthMetrics> =
        HashMap::new();

    for metric in metrics {
        let hour = round_to_next_hour(metric.created_at)?;
        let entry = partitioned_metrics
            .entry((hour, metric.user_id.clone()))
            .or_insert_with(|| BandwidthMetrics {
                user_id: metric.user_id.clone(),
                ingress: 0,
                egress: 0,
                created_at: hour,
            });

        entry.ingress += metric.ingress;
        entry.egress += metric.egress;
    }

    Ok(partitioned_metrics.values().cloned().collect::<Vec<_>>())
}

#[cfg(test)]
mod tests {
    use std::ops::Add;

    use super::*;
    use crate::database::test_helpers::{create_bandwidth_metric, setup_database};

    #[tokio::test]
    async fn round_to_previous_hour_works() {
        // 2024/02/12 12:00:01 AM
        let start_time = OffsetDateTime::from_unix_timestamp(1707696001).unwrap();
        let slot_end = round_to_previous_hour(start_time).unwrap();
        // 2024/02/12 12:00:00 AM
        assert_eq!(slot_end.unix_timestamp(), 1707696000);

        // 2024/02/12 12:00:00 AM
        let start_time = OffsetDateTime::from_unix_timestamp(1707696000).unwrap();
        let slot_end = round_to_previous_hour(start_time).unwrap();
        // 2024/02/12 12:00:00 AM
        assert_eq!(slot_end.unix_timestamp(), 1707696000);
    }

    #[tokio::test]
    async fn round_up_time_works() {
        // 2024/02/12 12:00:01 AM
        let start_time = OffsetDateTime::from_unix_timestamp(1707696001).unwrap();
        let slot_end = round_to_next_hour(start_time).unwrap();
        // 2024/02/12 01:00:00 AM
        assert_eq!(slot_end.unix_timestamp(), 1707699600);

        // 2024/02/12 12:00:00 AM
        let start_time = OffsetDateTime::from_unix_timestamp(1707696000).unwrap();
        let slot_end = round_to_next_hour(start_time).unwrap();
        // 2024/02/12 12:00:00 AM
        assert_eq!(slot_end.unix_timestamp(), 1707696000);
    }

    #[derive(sqlx::FromRow, Debug)]
    pub struct BandwidthMetricsDebug {
        pub user_id: String,
        pub ingress: i64,
        pub egress: i64,
        pub created_at: OffsetDateTime,
    }

    #[tokio::test]
    async fn bandwidth_works_at_time() {
        let user_id = "test_user";
        let db = setup_database().await;

        // 2024/02/12 12:00:01 AM
        let start_time = OffsetDateTime::from_unix_timestamp(1707696001).unwrap();
        let two_seconds_in_past = start_time.add(Duration::seconds(-2));
        create_bandwidth_metric(&db, user_id, 100, 200, two_seconds_in_past).await;

        let slot_end = round_to_previous_hour(start_time).unwrap();
        let users_metrics = bandwidth_metrics_until(&db, slot_end).await.unwrap();

        assert_eq!(users_metrics.len(), 1);
        assert_eq!(users_metrics[0].user_id, user_id);
        assert_eq!(users_metrics[0].ingress, 100);
        assert_eq!(users_metrics[0].egress, 200);
    }

    #[tokio::test]
    async fn partition_bandwidth_metrics_by_hour_and_user_works() {
        let user_id1 = "test_user1";
        let user_id2 = "test_user2";
        let db = setup_database().await;

        // 2024/02/12 13:00:00 PM
        let start_time1 = OffsetDateTime::from_unix_timestamp(1707742800).unwrap();
        let hour_in_the_past = start_time1.add(Duration::minutes(-60));
        create_bandwidth_metric(&db, user_id1, 100, 200, hour_in_the_past).await;
        create_bandwidth_metric(&db, user_id1, 100, 200, hour_in_the_past).await;
        create_bandwidth_metric(&db, user_id2, 300, 400, hour_in_the_past).await;
        create_bandwidth_metric(&db, user_id2, 300, 400, hour_in_the_past).await;

        let thirty_minutes_in_the_past = start_time1.add(Duration::minutes(-30));
        create_bandwidth_metric(&db, user_id1, 500, 600, thirty_minutes_in_the_past).await;
        create_bandwidth_metric(&db, user_id1, 500, 600, thirty_minutes_in_the_past).await;
        create_bandwidth_metric(&db, user_id2, 700, 800, thirty_minutes_in_the_past).await;
        create_bandwidth_metric(&db, user_id2, 700, 800, thirty_minutes_in_the_past).await;

        let slot_end = round_to_previous_hour(start_time1).unwrap();
        let users_metrics = bandwidth_metrics_until(&db, slot_end).await.unwrap();
        let partitioned_metrics =
            partition_bandwidth_metrics_by_hour_and_user(users_metrics).unwrap();

        let mut partitioned_metrics = partitioned_metrics;
        partitioned_metrics.sort_by(|a, b| a.created_at.cmp(&b.created_at));
        partitioned_metrics.sort_by(|a, b| a.user_id.cmp(&b.user_id));

        assert_eq!(partitioned_metrics.len(), 4);

        assert_eq!(partitioned_metrics[0].user_id, user_id1);
        assert_eq!(partitioned_metrics[0].ingress, 200);
        assert_eq!(partitioned_metrics[0].egress, 400);
        // 2024/02/12 12:00:00 AM
        assert_eq!(
            partitioned_metrics[0].created_at.unix_timestamp(),
            1707739200
        );

        assert_eq!(partitioned_metrics[1].user_id, user_id1);
        assert_eq!(partitioned_metrics[1].ingress, 1000);
        assert_eq!(partitioned_metrics[1].egress, 1200);
        // 2024/02/12 13:00:00 PM
        assert_eq!(
            partitioned_metrics[1].created_at.unix_timestamp(),
            1707742800
        );

        assert_eq!(partitioned_metrics[2].user_id, user_id2);
        assert_eq!(partitioned_metrics[2].ingress, 600);
        assert_eq!(partitioned_metrics[2].egress, 800);
        // 2024/02/12 12:00:00 AM
        assert_eq!(
            partitioned_metrics[2].created_at.unix_timestamp(),
            1707739200
        );

        assert_eq!(partitioned_metrics[3].user_id, user_id2);
        assert_eq!(partitioned_metrics[3].ingress, 1400);
        assert_eq!(partitioned_metrics[3].egress, 1600);
        // 2024/02/12 13:00:00 PM
        assert_eq!(
            partitioned_metrics[3].created_at.unix_timestamp(),
            1707742800
        );
    }

    #[tokio::test]
    async fn metrics_within_the_hour_wait_for_hour_to_pass() {
        let user_id = "test_user";
        let db = setup_database().await;

        // 2024/02/12 12:10:04 PM
        let start_time = OffsetDateTime::from_unix_timestamp(1707739804).unwrap();
        create_bandwidth_metric(&db, user_id, 100, 200, start_time).await;

        let ten_minutes_in_the_future = OffsetDateTime::from_unix_timestamp(1707739804).unwrap();
        let slot_end = round_to_previous_hour(ten_minutes_in_the_future).unwrap();
        let users_metrics = bandwidth_metrics_until(&db, slot_end).await.unwrap();
        assert_eq!(users_metrics.len(), 0);

        let hour_in_the_future = start_time.add(Duration::hours(1));
        let slot_end = round_to_previous_hour(hour_in_the_future).unwrap();
        let users_metrics = bandwidth_metrics_until(&db, slot_end).await.unwrap();
        assert_eq!(users_metrics.len(), 1);
        assert_eq!(users_metrics[0].user_id, user_id);
        assert_eq!(users_metrics[0].ingress, 100);
        assert_eq!(users_metrics[0].egress, 200);
    }
}
