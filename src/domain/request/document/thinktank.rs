use crate::domain::request::document::generally::{DocumentFile, DocumentName};

pub struct UploadDomainRequest {
    pub file: DocumentFile, // 临时文档
    pub name: DocumentName, // 文档名称
    pub uuid: String,       // 文档主键
    pub date: String,       // 文档日期
    pub head: String,       // 文档标题
    pub hold: String,       // 文档所属
    pub area: String,       // 应用范围
    pub stem: String,       // 文档来源
}
