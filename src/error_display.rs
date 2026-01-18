use std::time::{Duration, Instant};

use crate::pane::ErrorPane;

#[derive(Debug, Clone)]
pub enum ErrorSeverity {
    Blocking,      // Stops app, requires acknowledgment
    Notification,  // Temporary overlay, auto-dismisses
}

#[derive(Debug, Clone)]
pub struct ErrorContext {
    pub severity: ErrorSeverity,
    pub user_message: Option<String>,
    pub show_details: bool,
}

impl std::fmt::Display for ErrorContext {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if let Some(msg) = &self.user_message {
            write!(f, "{}", msg)
        } else {
            write!(f, "Error")
        }
    }
}

impl std::error::Error for ErrorContext {}

pub trait ErrorExt<T> {
    fn blocking(self) -> anyhow::Result<T>;
    fn with_message(self, msg: impl Into<String>) -> anyhow::Result<T>;
    fn with_details(self) -> anyhow::Result<T>;
}

impl<T, E> ErrorExt<T> for Result<T, E>
where
    E: Into<anyhow::Error>,
{
    fn blocking(self) -> anyhow::Result<T> {
        self.map_err(|e| {
            e.into().context(ErrorContext {
                severity: ErrorSeverity::Blocking,
                user_message: None,
                show_details: false,
            })
        })
    }

    fn with_message(self, msg: impl Into<String>) -> anyhow::Result<T> {
        self.map_err(|e| {
            e.into().context(ErrorContext {
                severity: ErrorSeverity::Notification,
                user_message: Some(msg.into()),
                show_details: false,
            })
        })
    }

    fn with_details(self) -> anyhow::Result<T> {
        self.map_err(|e| {
            e.into().context(ErrorContext {
                severity: ErrorSeverity::Notification,
                user_message: None,
                show_details: true,
            })
        })
    }
}

pub struct ErrorHandler;

impl ErrorHandler {
    pub fn handle(&self, err: &anyhow::Error) -> (ErrorPane, ErrorSeverity) {
        // Walk the error chain looking for ErrorContext
        let mut current: &dyn std::error::Error = err.as_ref();
        let mut severity = ErrorSeverity::Notification;
        let mut user_message: Option<String> = None;
        let mut show_details = false;

        // Collect all contexts in the chain
        loop {
            if let Some(ctx) = current.downcast_ref::<ErrorContext>() {
                // Blocking takes precedence over notification
                if matches!(ctx.severity, ErrorSeverity::Blocking) {
                    severity = ErrorSeverity::Blocking;
                }

                // First user message wins
                if user_message.is_none() && ctx.user_message.is_some() {
                    user_message = ctx.user_message.clone();
                }

                // Any request for details enables it
                if ctx.show_details {
                    show_details = true;
                }
            }

            match current.source() {
                Some(source) => current = source,
                None => break,
            }
        }

        let message = user_message.unwrap_or_else(|| format!("{}", err));

        let mut pane = ErrorPane::new("Error", message, severity.clone());

        if show_details {
            pane = pane.with_details(format!("{:#}", err));
        }

        (pane, severity)
    }
}

pub struct Notification {
    pub pane: ErrorPane,
    pub dismiss_at: Instant,
}

impl Notification {
    pub fn new(pane: ErrorPane) -> Self {
        Self {
            pane,
            dismiss_at: Instant::now() + Duration::from_secs(5),
        }
    }

    pub fn is_expired(&self) -> bool {
        Instant::now() >= self.dismiss_at
    }
}
