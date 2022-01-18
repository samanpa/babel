#[derive(Debug)]
pub struct Error {
    msg: String,
}

impl std::fmt::Display for Error {
    fn fmt(&self, fmt: &mut std::fmt::Formatter) -> std::result::Result<(), std::fmt::Error> {
        write!(fmt, "{}", self.msg)?;
        Ok(())
    }
}

impl From<String> for Error {
    fn from(e: String) -> Self {
        Error::new(e)
    }
}

impl std::error::Error for Error {
    fn cause(&self) -> Option<&dyn std::error::Error> {
        None
    }
}

impl Error {
    pub fn new<T>(msg: T) -> Self
    where
        T: Into<String>,
    {
        Error { msg: msg.into() }
    }
}
