use async_trait::async_trait;
use banyan_task::{CurrentTask, TaskLike};
use serde::{Deserialize, Serialize};
use time::error::ComponentRange;
use time::{Duration, OffsetDateTime};
use url::Url;

use crate::app::AppState;
use crate::clients::{CoreServiceClient, MeterTrafficRequest};
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
}

#[derive(Deserialize, Serialize)]
pub struct ReportBandwidthMetricsTask {}

impl ReportBandwidthMetricsTask {
    pub fn new() -> Self {
        Self {}
    }
}

#[async_trait]
impl TaskLike for ReportBandwidthMetricsTask {
    const TASK_NAME: &'static str = "report_bandwidth_metrics_task";

    type Error = ReportBandwidthMetricsTaskError;
    type Context = ReportBandwidthMetricsTaskContext;

    async fn run(&self, _task: CurrentTask, ctx: Self::Context) -> Result<(), Self::Error> {
        let conn = ctx.database();
        let slot_end = calculate_end_time(OffsetDateTime::now_utc())?;
        let users_metrics = user_bandwidth_over_window(&conn, slot_end).await?;
        let client = CoreServiceClient::new(
            ctx.secrets().service_signing_key(),
            ctx.service_name(),
            ctx.platform_name(),
            ctx.platform_hostname(),
        );
        println!("reporting users_metrics {:?}", users_metrics);

        for metrics in users_metrics.into_iter() {
            let meter_traffic_request = MeterTrafficRequest {
                user_id: metrics.user_id.as_str(),
                ingress: metrics.ingress,
                egress: metrics.egress,
                slot: slot_end.unix_timestamp(),
                storage_host_name: ctx.service_name(),
            };
            println!(
                "reporting meter_traffic_request {:?}",
                meter_traffic_request
            );
            if let Err(err) = client.report_user_bandwidth(meter_traffic_request).await {
                tracing::error!(
                    "could not report metrics for user {} err {}",
                    metrics.user_id.as_str(),
                    err
                );
                continue;
            }

            if let Err(err) = delete_over_window(&conn, slot_end).await {
                tracing::error!(
                    "could not delete the bandwidth end_slot {}, err: {}",
                    slot_end,
                    err
                )
            };
        }

        Ok(())
    }

    fn next_time(&self) -> Option<OffsetDateTime> {
        // every 20 minutes; not to miss the hour window
        Some(OffsetDateTime::now_utc() + Duration::minutes(20))
    }
}

fn calculate_end_time(start_time: OffsetDateTime) -> Result<OffsetDateTime, ComponentRange> {
    let timestamp = start_time.unix_timestamp()
        - (start_time.minute() as i64 * 60 + start_time.second() as i64);
    OffsetDateTime::from_unix_timestamp(timestamp)
}

async fn delete_over_window(db: &Database, end_time: OffsetDateTime) -> Result<u64, sqlx::Error> {
    let rows_deleted = sqlx::query!(
        r#"DELETE FROM bandwidth_metrics WHERE created_at <= $1"#,
        end_time,
    )
    .execute(db)
    .await?
    .rows_affected();

    Ok(rows_deleted)
}

async fn user_bandwidth_over_window(
    db: &Database,
    end_time: OffsetDateTime,
) -> Result<Vec<BandwidthMetrics>, sqlx::Error> {
    let res = sqlx::query_as!(
        BandwidthMetrics,
        "SELECT user_id, SUM(ingress) as ingress, SUM(egress) as egress FROM bandwidth_metrics
        WHERE created_at <= $1 GROUP BY user_id;",
        end_time,
    )
    .fetch_all(db)
    .await?;

    Ok(res)
}
#[cfg(test)]
mod tests {
    use std::ops::Add;

    use time::OffsetDateTime;

    use super::*;
    use crate::database::test_helpers::{create_bandwidth_metric, setup_database};

    #[tokio::test]
    async fn calculate_end_time_with_hardcoded_time() {
        // 2024/02/12 12:00:01 AM
        let start_time = OffsetDateTime::from_unix_timestamp(1707696001).unwrap();
        let slot_end = calculate_end_time(start_time).unwrap();
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

        let slot_end = calculate_end_time(start_time).unwrap();
        let users_metrics = user_bandwidth_over_window(&db, slot_end).await.unwrap();

        assert_eq!(users_metrics.len(), 1);
        assert_eq!(users_metrics[0].user_id, user_id);
        assert_eq!(users_metrics[0].ingress, 100);
        assert_eq!(users_metrics[0].egress, 200);
    }

    #[tokio::test]
    async fn metrics_within_the_hour_wait_for_hour_to_pass() {
        let user_id = "test_user";
        let db = setup_database().await;

        // 2024/02/12 12:10:04 PM
        let start_time = OffsetDateTime::from_unix_timestamp(1707739804).unwrap();
        create_bandwidth_metric(&db, user_id, 100, 200, start_time).await;

        let ten_minutes_in_the_future = OffsetDateTime::from_unix_timestamp(1707739804).unwrap();
        let slot_end = calculate_end_time(ten_minutes_in_the_future).unwrap();
        let users_metrics = user_bandwidth_over_window(&db, slot_end).await.unwrap();
        assert_eq!(users_metrics.len(), 0);

        let hour_in_the_future = start_time.add(Duration::hours(1));
        let slot_end = calculate_end_time(hour_in_the_future).unwrap();
        let users_metrics = user_bandwidth_over_window(&db, slot_end).await.unwrap();
        assert_eq!(users_metrics.len(), 1);
        assert_eq!(users_metrics[0].user_id, user_id);
        assert_eq!(users_metrics[0].ingress, 100);
        assert_eq!(users_metrics[0].egress, 200);
    }
}
