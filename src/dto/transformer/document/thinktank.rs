use actix_multipart::form::text::Text;
use serde::de::DeserializeOwned;

use crate::blunder::document::ParseError;
use crate::domain::request::document::generally::{DocumentFile, DocumentName};
use crate::domain::request::document::thinktank::UploadDomainRequest;
use crate::dto::request::document::thinktank::UploadRequest;

fn inner<T: DeserializeOwned>(text: Text<T>) -> T {
    text.into_inner()
}

impl TryFrom<UploadRequest> for UploadDomainRequest {
    type Error = ParseError;

    fn try_from(value: UploadRequest) -> Result<Self, Self::Error> {
        Ok(Self {
            file: DocumentFile::parse(value.file)?,
            name: DocumentName::parse(inner(value.name))?,
            uuid: inner(value.uuid),
            date: inner(value.date),
            head: inner(value.title),
            hold: inner(value.owner),
            area: inner(value.range),
            stem: inner(value.source),
        })
    }
}
