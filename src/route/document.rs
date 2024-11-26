use actix_web::web::{post, scope};
use actix_web::Scope;

use crate::handler::document::thinktank;

pub fn register_document_route() -> Scope {
    scope("/iaudit/chatgpt/document/thinktank").route("", post().to(thinktank::upload))
}
