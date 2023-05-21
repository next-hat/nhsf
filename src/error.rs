use ntex::{http::StatusCode, web::error::BlockingError};

pub struct CliError {
  pub(crate) msg: String,
  pub(crate) code: i32,
}

impl CliError {
  pub fn new(msg: String, code: i32) -> Self {
    Self { msg, code }
  }

  pub fn exit(&self) -> ! {
    eprintln!("{}", self.msg);
    std::process::exit(self.code);
  }
}

pub type CliResult<T> = Result<T, CliError>;

#[derive(Debug)]
pub struct CliIoError {
  context: Option<String>,
  inner: std::io::Error,
}

impl std::fmt::Display for CliIoError {
  fn fmt(
    &self,
    f: &mut std::fmt::Formatter<'_>,
  ) -> Result<(), std::fmt::Error> {
    use std::io::ErrorKind::*;

    let mut message;
    let message = if self.inner.raw_os_error().is_some() {
      // These are errors that come directly from the OS.
      // We want to normalize their messages across systems,
      // and we want to strip the "(os error X)" suffix.
      match self.inner.kind() {
        NotFound => "No such file or directory",
        PermissionDenied => "Permission denied",
        ConnectionRefused => "Connection refused",
        ConnectionReset => "Connection reset",
        ConnectionAborted => "Connection aborted",
        NotConnected => "Not connected",
        AddrInUse => "Address in use",
        AddrNotAvailable => "Address not available",
        BrokenPipe => "Broken pipe",
        AlreadyExists => "Already exists",
        WouldBlock => "Would block",
        InvalidInput => "Invalid input",
        InvalidData => "Invalid data",
        TimedOut => "Timed out",
        WriteZero => "Write zero",
        Interrupted => "Interrupted",
        UnexpectedEof => "Unexpected end of file",
        _ => {
          // TODO: When the new error variants
          // (https://github.com/rust-lang/rust/issues/86442)
          // are stabilized, we should add them to the match statement.
          message = strip_errno(&self.inner);
          capitalize(&mut message);
          &message
        }
      }
    } else {
      // These messages don't need as much normalization, and the above
      // messages wouldn't always be a good substitute.
      // For example, ErrorKind::NotFound doesn't necessarily mean it was
      // a file that was not found.
      // There are also errors with entirely custom messages.
      message = self.inner.to_string();
      capitalize(&mut message);
      &message
    };
    if let Some(ctx) = &self.context {
      write!(f, "{ctx}: {message}")
    } else {
      write!(f, "{message}")
    }
  }
}

impl std::error::Error for CliIoError {}

/// Capitalize the first character of an ASCII string.
fn capitalize(text: &mut str) {
  if let Some(first) = text.get_mut(..1) {
    first.make_ascii_uppercase();
  }
}

/// Strip the trailing " (os error XX)" from io error strings.
fn strip_errno(err: &std::io::Error) -> String {
  let mut msg = err.to_string();
  if let Some(pos) = msg.find(" (os error ") {
    msg.truncate(pos);
  }
  msg
}

/// Enables the conversion from [`std::io::Error`] to [`UError`] and from [`std::io::Result`] to
/// [`UResult`].
pub trait FromIo<T> {
  fn map_err_context(self, context: impl FnOnce() -> String) -> T;
}

impl FromIo<Box<CliIoError>> for std::io::Error {
  fn map_err_context(
    self,
    context: impl FnOnce() -> String,
  ) -> Box<CliIoError> {
    Box::new(CliIoError {
      context: Some((context)()),
      inner: self,
    })
  }
}

impl<T> FromIo<Box<CliIoError>> for BlockingError<T>
where
  T: std::fmt::Debug + std::fmt::Display + Sync + Send + 'static,
{
  fn map_err_context(
    self,
    context: impl FnOnce() -> String,
  ) -> Box<CliIoError> {
    Box::new(CliIoError {
      context: Some((context)()),
      inner: std::io::Error::new(std::io::ErrorKind::Other, self),
    })
  }
}

impl FromIo<Box<CliIoError>> for std::io::ErrorKind {
  fn map_err_context(
    self,
    context: impl FnOnce() -> String,
  ) -> Box<CliIoError> {
    Box::new(CliIoError {
      context: Some((context)()),
      inner: std::io::Error::new(self, ""),
    })
  }
}

impl From<std::io::Error> for CliIoError {
  fn from(f: std::io::Error) -> Self {
    Self {
      context: None,
      inner: f,
    }
  }
}

impl From<CliIoError> for CliError {
  fn from(f: CliIoError) -> Self {
    Self {
      msg: f.to_string(),
      code: f.inner.raw_os_error().unwrap_or(1),
    }
  }
}

impl From<Box<CliIoError>> for CliError {
  fn from(f: Box<CliIoError>) -> Self {
    Self {
      msg: f.to_string(),
      code: f.inner.raw_os_error().unwrap_or(1),
    }
  }
}

impl FromIo<Box<CliIoError>> for serde_yaml::Error {
  fn map_err_context(
    self,
    context: impl FnOnce() -> String,
  ) -> Box<CliIoError> {
    Box::new(CliIoError {
      context: Some((context)()),
      inner: std::io::Error::new(std::io::ErrorKind::Other, self),
    })
  }
}

/// Http response error
#[derive(Clone, Debug)]
pub struct HttpError {
  pub(crate) msg: String,
  pub(crate) status: StatusCode,
}

pub type HttpResult = Result<ntex::web::HttpResponse, HttpError>;

impl std::fmt::Display for HttpError {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    write!(f, "[{}]{}", self.status, self.msg)
  }
}

impl std::error::Error for HttpError {}

impl ntex::web::WebResponseError for HttpError {
  // builds the actual response to send back when an error occurs
  fn error_response(
    &self,
    _: &ntex::web::HttpRequest,
  ) -> ntex::web::HttpResponse {
    let err_json = serde_json::json!({ "msg": self.msg });
    ntex::web::HttpResponse::build(self.status).json(&err_json)
  }
}

pub trait AsHttpError {
  fn set_status(&mut self, status: StatusCode) -> HttpError;
}

impl AsHttpError for CliIoError {
  fn set_status(&mut self, status: StatusCode) -> HttpError {
    HttpError {
      msg: self.to_string(),
      status,
    }
  }
}
