use std::time::{Duration, Instant};

use crate::pane::ErrorPane;

#[derive(Debug, Clone, Copy)]
pub enum ErrorSeverity {
    Blocking,     // Stops app, requires acknowledgment
    Notification, // Temporary overlay, auto-dismisses
}

#[derive(Debug, Clone)]
pub struct AppError {
    pub severity: ErrorSeverity,
    pub message: String,
    pub details: Option<String>,
}

impl AppError {
    pub fn new(message: impl Into<String>) -> Self {
        Self {
            severity: ErrorSeverity::Notification,
            message: message.into(),
            details: None,
        }
    }

    pub fn blocking(mut self) -> Self {
        self.severity = ErrorSeverity::Blocking;
        self
    }

    pub fn with_message(mut self, message: impl Into<String>) -> Self {
        self.message = message.into();
        self
    }

    pub fn with_details(mut self, details: impl Into<String>) -> Self {
        self.details = Some(details.into());
        self
    }

    pub fn severity(&self) -> &ErrorSeverity {
        &self.severity
    }

    pub fn message(&self) -> &str {
        &self.message
    }

    pub fn details(&self) -> Option<&str> {
        self.details.as_deref()
    }
}

impl std::fmt::Display for AppError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.message)
    }
}

impl std::error::Error for AppError {}

// Common From implementations for standard error types
impl From<std::io::Error> for AppError {
    fn from(err: std::io::Error) -> Self {
        Self::new(format!("{}", err))
            .with_details(format!("{:?}", err))
    }
}

impl From<reqwest::Error> for AppError {
    fn from(err: reqwest::Error) -> Self {
        Self::new(format!("{}", err))
            .with_details(format!("{:?}", err))
    }
}

impl From<serde_json::Error> for AppError {
    fn from(err: serde_json::Error) -> Self {
        Self::new(format!("{}", err))
            .with_details(format!("{:?}", err))
    }
}

impl From<std::env::VarError> for AppError {
    fn from(err: std::env::VarError) -> Self {
        Self::new(format!("{}", err))
            .with_details(format!("{:?}", err))
    }
}

impl From<uuid::Error> for AppError {
    fn from(err: uuid::Error) -> Self {
        Self::new(format!("{}", err))
            .with_details(format!("{:?}", err))
    }
}

impl From<tokio::task::JoinError> for AppError {
    fn from(err: tokio::task::JoinError) -> Self {
        Self::new(format!("{}", err))
            .with_details(format!("{:?}", err))
    }
}

pub type Result<T> = std::result::Result<T, AppError>;

pub trait ErrorExt<T> {
    fn blocking(self) -> Result<T>;
    fn with_message(self, msg: impl Into<String>) -> Result<T>;
    fn with_details(self) -> Result<T>;
}

impl<T, E> ErrorExt<T> for std::result::Result<T, E>
where
    E: std::fmt::Display + std::fmt::Debug + 'static,
{
    fn blocking(self) -> Result<T> {
        self.map_err(|e| AppError::new(format!("{}", e))
            .with_details(format!("{:?}", e))
            .blocking())
    }

    fn with_message(self, msg: impl Into<String>) -> Result<T> {
        self.map_err(|e| {
            // Check if e is already an AppError to preserve its severity
            let severity = if let Some(app_err) = (&e as &dyn std::any::Any).downcast_ref::<AppError>() {
                app_err.severity.clone()
            } else {
                ErrorSeverity::Notification
            };

            AppError {
                severity,
                message: msg.into(),
                details: Some(format!("{:?}", e)),
            }
        })
    }

    fn with_details(self) -> Result<T> {
        self.map_err(|e| {
            // Check if e is already an AppError to preserve its severity
            let severity = if let Some(app_err) = (&e as &dyn std::any::Any).downcast_ref::<AppError>() {
                app_err.severity.clone()
            } else {
                ErrorSeverity::Notification
            };

            let msg = format!("{}", e);
            let details = format!("{:?}", e);

            AppError {
                severity,
                message: msg,
                details: Some(details),
            }
        })
    }
}

pub struct ErrorHandler;

impl ErrorHandler {
    pub fn handle(&self, err: &AppError) -> (ErrorPane, ErrorSeverity) {
        let message = err.message();
        let severity = err.severity().clone();

        let mut pane = ErrorPane::new("Error", message, severity.clone());

        if let Some(details) = err.details() {
            pane = pane.with_details(details);
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
