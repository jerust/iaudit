use std::fmt;

use actix_web::http::StatusCode;
use actix_web::ResponseError;

use crate::blunder::errchain;

#[derive(thiserror::Error)]
pub enum ParseError {
    #[error("文件名缺失")]
    MissingFileName,

    #[error("扩展名缺失")]
    MissingExtension,

    #[error("扩展名无效")]
    InvalidExtension,
}

impl fmt::Debug for ParseError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        errchain::errorchain(self, f)
    }
}

#[derive(thiserror::Error)]
pub enum DocumentError {
    #[error("文档解析错误: {0}")]
    ValidationError(#[from] ParseError),

    #[error(transparent)]
    UnexpectedError(#[from] anyhow::Error),
}

impl fmt::Debug for DocumentError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        errchain::errorchain(self, f)
    }
}

// impl fmt::Debug for DocumentError {
//     fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
//         match self {
//             DocumentError::ValidationError(parse_error) => {
//                 writeln!(f, "解析错误: {:?}", parse_error)
//             }
//             DocumentError::UnexpectedError(err) => {
//                 writeln!(f, "Unexpected error: {}", err)?;
//                 errchain::errorchain(self, f)
//             }
//         }
//     }
// }

// 如果handler层要返回自定义错误类型给actix-web, 例如: `Result<HttpResponse, DocumentError>`
// 那么必须为自定义错误类型实现`ResponseError`特征
impl ResponseError for DocumentError {
    // 自定义状态码
    fn status_code(&self) -> StatusCode {
        match self {
            DocumentError::ValidationError(_) => StatusCode::BAD_REQUEST,
            DocumentError::UnexpectedError(_) => StatusCode::INTERNAL_SERVER_ERROR,
        }
    }

    // 自定义错误响应
    // fn error_response(&self) -> HttpResponse<BoxBody> {
    //     match self {
    //         // 当遇到客户度错误时, 不想直接给前端返回400状态码, 而是想返回200状态码以及额外的信息, 例如:
    //         // {"code": 400, "error": "xxx"}
    //         DocumentError::ValidationError(error) => {
    //             HttpResponse::Ok().json(DocumentResponse::bad_request(error.to_string()))
    //         }
    //         // 当遇到服务端错误时, 不想直接给前端返回500状态码, 而是想返回200状态码以及额外的信息, 例如:
    //         // {"code": 500, "error": "xxx"}
    //         DocumentError::UnexpectedError(error) => {
    //             HttpResponse::Ok().json(DocumentResponse::internal_server_error(error.to_string()))
    //         }
    //     }
    // }
}
