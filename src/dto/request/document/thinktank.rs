use actix_multipart::form::tempfile::TempFile;
use actix_multipart::form::text::Text;
use actix_multipart::form::MultipartForm;

#[derive(MultipartForm)]
pub struct UploadRequest {
    pub file: TempFile,       // 临时文档
    pub name: Text<String>,   // 文档名称
    pub uuid: Text<String>,   // 文档主键
    pub date: Text<String>,   // 文档日期
    pub title: Text<String>,  // 文档标题
    pub owner: Text<String>,  // 文档所属
    pub range: Text<String>,  // 应用范围
    pub source: Text<String>, // 文档来源
}
