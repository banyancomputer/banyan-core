use time::{Date, OffsetDateTime, Time};

use crate::database::DatabaseConnection;

#[derive(sqlx::FromRow)]
pub struct MetricsTraffic {
    pub user_id: String,
    pub ingress: i64,
    pub egress: i64,
    pub storage_host_id: String,
}

impl MetricsTraffic {
    pub async fn find_by_user_for_the_month(
        conn: &mut DatabaseConnection,
        user_id: &str,
    ) -> Result<Option<Self>, sqlx::Error> {
        let today = OffsetDateTime::now_utc();
        let year = today.year();
        let month = today.month();

        let beginning_of_month = OffsetDateTime::new_utc(
            Date::from_calendar_date(year, month, 1).expect("Valid date"),
            Time::MIDNIGHT,
        );

        sqlx::query_as!(
            Self,
            r#"SELECT user_id, SUM(ingress) as ingress, SUM(egress) as egress, storage_host_id
                FROM metrics_traffic
                WHERE user_id = $1 AND slot >= $2
                GROUP BY user_id;"#,
            user_id,
            beginning_of_month,
        )
        .fetch_optional(&mut *conn)
        .await
    }
}
