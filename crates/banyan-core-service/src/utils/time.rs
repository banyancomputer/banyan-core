use time::error::ComponentRange;
use time::{Duration, OffsetDateTime};

pub(crate) fn round_to_next_hour(
    start_time: OffsetDateTime,
) -> Result<OffsetDateTime, ComponentRange> {
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
