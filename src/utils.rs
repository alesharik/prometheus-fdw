use crate::error::{Error, Result};
use chrono::{DateTime, Datelike, NaiveDate, NaiveDateTime, NaiveTime, TimeZone, Timelike, Utc};
use pgrx::Timestamp;

pub fn to_chrono(ts: Timestamp) -> Result<DateTime<Utc>> {
    Ok(Utc.from_utc_datetime(&NaiveDateTime::new(
        NaiveDate::from_ymd_opt(ts.year(), ts.month() as u32, ts.day() as u32)
            .ok_or(Error::TimestampInvalid)?,
        NaiveTime::from_hms_micro_opt(
            ts.hour() as u32,
            ts.minute() as u32,
            ts.second() as u32,
            ts.microseconds(),
        )
        .ok_or(Error::TimestampInvalid)?,
    )))
}

pub fn from_chrono(ts: DateTime<Utc>) -> Timestamp {
    Timestamp::new_unchecked(
        ts.year() as isize,
        ts.month() as u8,
        ts.day() as u8,
        ts.hour() as u8,
        ts.minute() as u8,
        ts.second() as f64,
    )
}
