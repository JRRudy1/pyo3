/// A simple macro for returning an error. Resembles anyhow::bail.
#[macro_export]
macro_rules! bail {
    ($msg: expr) => { return Err($msg.into()); };
    ($fmt: literal $($args: tt)+) => { return Err(format!($fmt $($args)+).into()); };
}

/// A simple macro for checking a condition. Resembles anyhow::ensure.
#[macro_export]
macro_rules! ensure {
    ($condition:expr, $($args: tt)+) => { if !($condition) { bail!($($args)+) } };
}

/// Show warning. If needed, please extend this macro to support arguments.
#[macro_export]
macro_rules! warn {
    ($msg: literal) => {
        println!(concat!("cargo:warning=", $msg));
    };
}

/// A simple error implementation which allows chaining of errors, inspired somewhat by anyhow.
#[derive(Debug)]
pub struct Error {
    value: String,
    source: Option<Box<dyn std::error::Error>>,
}

/// Error report inspired by
/// https://blog.rust-lang.org/inside-rust/2021/07/01/What-the-error-handling-project-group-is-working-towards.html#2-error-reporter
pub struct ErrorReport<'a>(&'a Error);

impl Error {
    pub fn report(&self) -> ErrorReport<'_> {
        ErrorReport(self)
    }
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.value)
    }
}

impl std::error::Error for Error {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        self.source.as_deref()
    }
}

impl std::fmt::Display for ErrorReport<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        use std::error::Error;
        self.0.fmt(f)?;
        let mut source = self.0.source();
        if source.is_some() {
            writeln!(f, "\ncaused by:")?;
            let mut index = 0;
            while let Some(some_source) = source {
                writeln!(f, "  - {}: {}", index, some_source)?;
                source = some_source.source();
                index += 1;
            }
        }
        Ok(())
    }
}

impl From<String> for Error {
    fn from(value: String) -> Self {
        Self {
            value,
            source: None,
        }
    }
}

impl From<&'_ str> for Error {
    fn from(value: &str) -> Self {
        value.to_string().into()
    }
}

impl From<std::convert::Infallible> for Error {
    fn from(_: std::convert::Infallible) -> Self {
        unreachable!()
    }
}

pub type Result<T, E = Error> = std::result::Result<T, E>;

pub trait Context<T> {
    fn context(self, message: impl Into<String>) -> Result<T>;
    fn with_context(self, message: impl FnOnce() -> String) -> Result<T>;
}

impl<T, E> Context<T> for Result<T, E>
where
    E: std::error::Error + 'static,
{
    fn context(self, message: impl Into<String>) -> Result<T> {
        self.map_err(|error| Error {
            value: message.into(),
            source: Some(Box::new(error)),
        })
    }

    fn with_context(self, message: impl FnOnce() -> String) -> Result<T> {
        self.map_err(|error| Error {
            value: message(),
            source: Some(Box::new(error)),
        })
    }
}
