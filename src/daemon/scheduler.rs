use chrono::{DateTime, Local};

pub trait Clock: Send + Sync + 'static {
    fn now(&self) -> DateTime<Local>;
}

pub trait Notifier: Send + Sync + 'static {
    fn notify(&self, title: &str, body: &str);
}

pub struct SystemClock;

impl Clock for SystemClock {
    fn now(&self) -> DateTime<Local> {
        Local::now()
    }
}

pub struct DBusNotifier;

impl Notifier for DBusNotifier {
    fn notify(&self, title: &str, body: &str) {
        // notify-rust returns a NotificationHandle on success;
        // we don't keep it, just fire-and-forget.
        let _ = notify_rust::Notification::new()
            .summary(title)
            .body(body)
            .show();
    }
}

#[cfg(test)]
mod traits_tests {
    use super::*;
    use chrono::TimeZone;

    pub struct FakeClock(pub std::sync::Mutex<DateTime<Local>>);

    impl FakeClock {
        pub fn new(at: DateTime<Local>) -> Self {
            Self(std::sync::Mutex::new(at))
        }
        pub fn advance(&self, by: chrono::Duration) {
            let mut g = self.0.lock().unwrap();
            *g = *g + by;
        }
    }

    impl Clock for FakeClock {
        fn now(&self) -> DateTime<Local> {
            *self.0.lock().unwrap()
        }
    }

    #[derive(Default)]
    pub struct CapturingNotifier(pub std::sync::Mutex<Vec<(String, String)>>);

    impl Notifier for CapturingNotifier {
        fn notify(&self, title: &str, body: &str) {
            self.0.lock().unwrap().push((title.into(), body.into()));
        }
    }

    #[test]
    fn fake_clock_advances() {
        let c = FakeClock::new(Local.with_ymd_and_hms(2026, 5, 31, 8, 0, 0).unwrap());
        c.advance(chrono::Duration::hours(2));
        assert_eq!(
            c.now(),
            Local.with_ymd_and_hms(2026, 5, 31, 10, 0, 0).unwrap()
        );
    }
}
