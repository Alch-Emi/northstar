use anyhow::*;
use crate::Mime;
use crate::util::Cowy;
use crate::types::{Status, Meta};

#[derive(Debug,Clone)]
pub struct ResponseHeader {
    pub status: Status,
    pub meta: Meta,
}

impl ResponseHeader {
    pub fn input(prompt: impl Cowy<str>) -> Result<Self> {
        Ok(Self {
            status: Status::INPUT,
            meta: Meta::new(prompt).context("Invalid input prompt")?,
        })
    }

    pub fn input_lossy(prompt: impl Cowy<str>) -> Self {
        Self {
            status: Status::INPUT,
            meta: Meta::new_lossy(prompt),
        }
    }

    pub fn success(mime: &Mime) -> Self {
        Self {
            status: Status::SUCCESS,
            meta: Meta::new_lossy(mime.to_string()),
        }
    }

    pub fn server_error(reason: impl Cowy<str>) -> Result<Self> {
        Ok(Self {
            status: Status::PERMANENT_FAILURE,
            meta: Meta::new(reason).context("Invalid server error reason")?,
        })
    }

    pub fn server_error_lossy(reason: impl Cowy<str>) -> Self {
        Self {
            status: Status::PERMANENT_FAILURE,
            meta: Meta::new_lossy(reason),
        }
    }

    pub fn not_found() -> Self {
        Self {
            status: Status::NOT_FOUND,
            meta: Meta::new_lossy("Not found"),
        }
    }

    pub fn client_certificate_required() -> Self {
        Self {
            status: Status::CLIENT_CERTIFICATE_REQUIRED,
            meta: Meta::new_lossy("No certificate provided"),
        }
    }

    pub fn certificate_not_authorized() -> Self {
        Self {
            status: Status::CERTIFICATE_NOT_AUTHORIZED,
            meta: Meta::new_lossy("Your certificate is not authorized to view this content"),
        }
    }

    pub fn status(&self) -> &Status {
        &self.status
    }

    pub fn meta(&self) -> &Meta {
        &self.meta
    }
}
