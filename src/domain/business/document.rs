use serde_json::{json, Value};

use crate::domain::request::document::thinktank;

/// 每一个切片都当作一个文档对象
pub struct Document(Value);

impl From<thinktank::UploadDomainRequest> for Document {
    fn from(value: thinktank::UploadDomainRequest) -> Self {
        let mut payload = json!({
            "fileuuid": value.uuid,
            "filename": value.name.name(),
            "filedate": value.date,
            "filehead": value.head,
            "filehold": value.hold,
            "filestem": value.stem,
        });
        value
            .area
            .split("/")
            .filter(|part| !part.is_empty())
            .for_each(|key| {
                payload[key] = json!(1);
            });
        Self(payload)
    }
}

impl Document {
    pub fn update(&mut self, value: Value) {
        if let (Some(source), Some(target)) = (self.0.as_object_mut(), value.as_object()) {
            source.extend(target.clone());
        }
    }

    /// 因为一个文档对应多个切片, 因此这些切片需要共享文档的元数据
    pub fn value(&self) -> Value {
        self.0.clone()
    }
}
