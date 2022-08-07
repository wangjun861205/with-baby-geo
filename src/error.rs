use std::fmt::Display;

use actix_web::http::StatusCode;
use actix_web::HttpResponse;
use actix_web::Responder;
use actix_web::ResponseError;

#[derive(Debug)]
pub(crate) struct Error(anyhow::Error);

impl From<anyhow::Error> for Error {
    fn from(e: anyhow::Error) -> Self {
        Self(e)
    }
}

impl Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl ResponseError for Error {
    fn error_response(&self) -> HttpResponse<actix_web::body::BoxBody> {
        HttpResponse::new(StatusCode::INTERNAL_SERVER_ERROR)
    }
}
