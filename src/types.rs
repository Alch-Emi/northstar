use std::ops;
use anyhow::*;
use mime::Mime;
use percent_encoding::percent_decode_str;
use tokio::{io::AsyncRead, fs::File};
use uriparse::URIReference;

pub struct Request {
    uri: URIReference<'static>,
    input: Option<String>,
}

impl Request {
    pub fn from_uri(mut uri: URIReference<'static>) -> Result<Self> {
        uri.normalize();

        let input = match uri.query() {
            None => None,
            Some(query) => {
                let input = percent_decode_str(query.as_str())
                    .decode_utf8()?
                    .into_owned();
                Some(input)
            }
        };

        Ok(Self {
            uri,
            input,
        })
    }

    pub fn uri(&self) -> &URIReference {
        &self.uri
    }

    pub fn path_segments(&self) -> Vec<String> {
        self.uri()
            .path()
            .segments()
            .iter()
            .map(|segment| percent_decode_str(segment.as_str()).decode_utf8_lossy().into_owned())
            .collect::<Vec<String>>()
    }

    pub fn input(&self) -> Option<&str> {
        self.input.as_deref()
    }
}

impl ops::Deref for Request {
    type Target = URIReference<'static>;

    fn deref(&self) -> &Self::Target {
        &self.uri
    }
}

#[derive(Debug,Clone)]
pub struct ResponseHeader {
    pub status: Status,
    pub meta: Meta,
}

impl ResponseHeader {
    pub fn input(prompt: impl AsRef<str> + Into<String>) -> Result<Self> {
        Ok(Self {
            status: Status::INPUT,
            meta: Meta::new(prompt)?,
        })
    }

    pub fn success(mime: &Mime) -> Result<Self> {
        Ok(Self {
            status: Status::SUCCESS,
            meta: Meta::new(mime.to_string())?,
        })
    }

    pub fn server_error(reason: impl AsRef<str> + Into<String>) -> Result<Self> {
        Ok(Self {
            status: Status::PERMANENT_FAILURE,
            meta: Meta::new(reason)?,
        })
    }

    pub fn not_found() -> Result<Self> {
        Ok(Self {
            status: Status::NOT_FOUND,
            meta: Meta::new("Not found")?,
        })
    }

    pub fn status(&self) -> &Status {
        &self.status
    }

    pub fn meta(&self) -> &Meta {
        &self.meta
    }
}

#[derive(Debug,Copy,Clone,PartialEq,Eq)]
pub struct Status(u8);

impl Status {
    pub const INPUT: Self = Self(10);
    pub const SENSITIVE_INPUT: Self = Self(11);
    pub const SUCCESS: Self = Self(20);
    pub const REDIRECT_TEMPORARY: Self = Self(30);
    pub const REDIRECT_PERMANENT: Self = Self(31);
    pub const TEMPORARY_FAILURE: Self = Self(40);
    pub const SERVER_UNAVAILABLE: Self = Self(41);
    pub const CGI_ERROR: Self = Self(42);
    pub const PROXY_ERROR: Self = Self(43);
    pub const SLOW_DOWN: Self = Self(44);
    pub const PERMANENT_FAILURE: Self = Self(50);
    pub const NOT_FOUND: Self = Self(51);
    pub const GONE: Self = Self(52);
    pub const PROXY_REQUEST_REFUSED: Self = Self(53);
    pub const BAD_REQUEST: Self = Self(59);
    pub const CLIENT_CERTIFICATE_REQUIRED: Self = Self(60);

    pub fn code(&self) -> u8 {
        self.0
    }

    pub fn is_success(&self) -> bool {
        self.category().is_success()
    }

    pub fn category(&self) -> StatusCategory {
        let class = self.0 / 10;

        match class {
            1 => StatusCategory::Input,
            2 => StatusCategory::Success,
            3 => StatusCategory::Redirect,
            4 => StatusCategory::TemporaryFailure,
            5 => StatusCategory::PermanentFailure,
            6 => StatusCategory::ClientCertificateRequired,
            _ => StatusCategory::PermanentFailure,
        }
    }
}

#[derive(Copy,Clone,PartialEq,Eq)]
pub enum StatusCategory {
    Input,
    Success,
    Redirect,
    TemporaryFailure,
    PermanentFailure,
    ClientCertificateRequired,
}

impl StatusCategory {
    pub fn is_input(&self) -> bool {
        *self == Self::Input
    }

    pub fn is_success(&self) -> bool {
        *self == Self::Success
    }

    pub fn redirect(&self) -> bool {
        *self == Self::Redirect
    }

    pub fn is_temporary_failure(&self) -> bool {
        *self == Self::TemporaryFailure
    }

    pub fn is_permanent_failure(&self) -> bool {
        *self == Self::PermanentFailure
    }

    pub fn is_client_certificate_required(&self) -> bool {
        *self == Self::ClientCertificateRequired
    }
}

#[derive(Debug,Clone,PartialEq,Eq,Default)]
pub struct Meta(String);

impl Meta {
    pub fn new(meta: impl AsRef<str> + Into<String>) -> Result<Self> {
        ensure!(!meta.as_ref().contains("\n"), "Meta must not contain newlines");

        Ok(Self(meta.into()))
    }

    pub fn empty() -> Self {
        Self::default()
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }

    pub fn to_mime(&self) -> Result<Mime> {
        let mime = self.as_str().parse::<Mime>()?;
        Ok(mime)
    }
}

pub struct Response {
    header: ResponseHeader,
    body: Option<Body>,
}

impl Response {
    pub fn new(header: ResponseHeader) -> Self {
        Self {
            header,
            body: None,
        }
    }

    pub fn input(prompt: impl AsRef<str> + Into<String>) -> Result<Self> {
        let header = ResponseHeader::input(prompt)?;
        Ok(Self::new(header))
    }

    pub fn success(mime: &Mime) -> Result<Self> {
        let header = ResponseHeader::success(&mime)?;
        Ok(Self::new(header))
    }

    pub fn server_error(reason: impl AsRef<str> + Into<String>) -> Result<Self>  {
        let header = ResponseHeader::server_error(reason)?;
        Ok(Self::new(header))
    }

    pub fn not_found() -> Result<Self> {
        let header = ResponseHeader::not_found()?;
        Ok(Self::new(header))
    }

    pub fn with_body(mut self, body: impl Into<Body>) -> Self {
        self.body = Some(body.into());
        self
    }

    pub fn header(&self) -> &ResponseHeader {
        &self.header
    }

    pub fn take_body(&mut self) -> Option<Body> {
        self.body.take()
    }
}

pub enum Body {
    Bytes(Vec<u8>),
    Reader(Box<dyn AsyncRead + Send + Sync + Unpin>),
}

impl From<Vec<u8>> for Body {
    fn from(bytes: Vec<u8>) -> Self {
        Self::Bytes(bytes)
    }
}

impl<'a> From<&'a [u8]> for Body {
    fn from(bytes: &[u8]) -> Self {
        Self::Bytes(bytes.to_owned())
    }
}

impl From<String> for Body {
    fn from(text: String) -> Self {
        Self::Bytes(text.into_bytes())
    }
}

impl<'a> From<&'a str> for Body {
    fn from(text: &str) -> Self {
        Self::Bytes(text.to_owned().into_bytes())
    }
}

impl From<File> for Body {
    fn from(file: File) -> Self {
        Self::Reader(Box::new(file))
    }
}