use time::error::ComponentRange;
use time::OffsetDateTime;

pub fn round_to_previous_hour(
    start_time: OffsetDateTime,
) -> Result<OffsetDateTime, ComponentRange> {
    start_time
        .replace_minute(0)
        .and_then(|t| t.replace_second(0))
        .and_then(|t| t.replace_nanosecond(0))
}
