use chrono::{FixedOffset, Local, NaiveDate, NaiveDateTime, Utc};

/// AEST = UTC+10
const AEST_OFFSET_SECS: i32 = 10 * 3600;

pub fn today() -> NaiveDate {
    now().date_naive()
}

pub fn now_naive() -> NaiveDateTime {
    now().naive_local()
}

fn now() -> chrono::DateTime<FixedOffset> {
    let local = Local::now().fixed_offset();

    // If detected offset is UTC and there's no explicit timezone config,
    // the system likely has no timezone set — use AEST as fallback.
    if local.offset().local_minus_utc() == 0 && !is_utc_intentional() {
        let aest = FixedOffset::east_opt(AEST_OFFSET_SECS).unwrap();
        return Utc::now().with_timezone(&aest);
    }

    local
}

fn is_utc_intentional() -> bool {
    if std::env::var("TZ").is_ok() {
        return true;
    }

    #[cfg(unix)]
    if std::path::Path::new("/etc/localtime").exists() {
        return true;
    }

    false
}
