use std::fmt;

#[derive(Debug)]
pub enum Error {
    Invalid(String),
    NotFound(String),
    Io(std::io::Error),
    Json(serde_json::Error),
    Reqwest(reqwest::Error),
    Serenity(serenity::Error),
    Sqlx(sqlx::Error),
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            Self::Invalid(ref err) | Self::NotFound(ref err) => write!(f, "{err}"),
            Self::Io(ref err) => write!(f, "{err}"),
            Self::Json(ref err) => write!(f, "{err}"),
            Self::Reqwest(ref err) => write!(f, "{err}"),
            Self::Serenity(ref err) => write!(f, "{err}"),
            Self::Sqlx(ref err) => write!(f, "{err}"),
        }
    }
}

impl From<std::io::Error> for Error {
    fn from(err: std::io::Error) -> Self {
        Self::Io(err)
    }
}

impl From<serde_json::Error> for Error {
    fn from(err: serde_json::Error) -> Self {
        Self::Json(err)
    }
}

impl From<reqwest::Error> for Error {
    fn from(err: reqwest::Error) -> Self {
        Self::Reqwest(err)
    }
}

impl From<serenity::Error> for Error {
    fn from(err: serenity::Error) -> Self {
        Self::Serenity(err)
    }
}

impl From<sqlx::Error> for Error {
    fn from(err: sqlx::Error) -> Self {
        Self::Sqlx(err)
    }
}
