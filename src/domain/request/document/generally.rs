use std::borrow::Cow;
use std::io;
use std::ops::Deref;
use std::path::Path;

use actix_multipart::form::tempfile::TempFile;
use tokio::fs;

use crate::blunder::document::ParseError;

#[derive(Clone, Debug)]
pub enum Extension {
    Doc,
    Pdf,
    Xls,
    Docx,
    Xlsx,
}

impl TryFrom<&str> for Extension {
    type Error = ParseError;

    fn try_from(s: &str) -> Result<Self, Self::Error> {
        match s.to_lowercase().as_str() {
            "doc" => Ok(Extension::Doc),
            "pdf" => Ok(Extension::Pdf),
            "xls" => Ok(Extension::Xls),
            "docx" => Ok(Extension::Docx),
            "xlsx" => Ok(Extension::Xlsx),
            _ => Err(ParseError::InvalidExtension),
        }
    }
}

#[derive(Debug)]
pub struct DocumentName(String, Extension);

impl DocumentName {
    pub fn parse(s: String) -> Result<Self, ParseError> {
        let extension = s
            .to_lowercase()
            .rsplit('.')
            .next()
            .ok_or_else(|| ParseError::MissingExtension)?
            .try_into()?;
        Ok(Self(s, extension))
    }

    pub fn name(&self) -> Cow<str> {
        Cow::Borrowed(&self.0)
    }

    pub fn extension(&self) -> Cow<Extension> {
        Cow::Borrowed(&self.1)
    }
}

pub struct DocumentFile(DocumentName, TempFile);

impl Deref for DocumentFile {
    type Target = DocumentName;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DocumentFile {
    pub fn parse(tempfile: TempFile) -> Result<Self, ParseError> {
        let filename = tempfile
            .file_name
            .as_ref()
            .ok_or_else(|| ParseError::MissingFileName)?
            .to_owned();
        Ok(Self(DocumentName::parse(filename)?, tempfile))
    }

    pub async fn persist<T: AsRef<Path>>(&self, target: T) -> Result<(), io::Error> {
        let source = self.1.file.path();
        fs::copy(source, target).await?;
        fs::remove_file(source).await?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // 判断文件名称是否匹配特定的扩展名
    fn has_valid_extension<F: AsRef<str>>(filename: F) -> bool {
        matches!(
            filename.as_ref().to_lowercase().split('.').last(),
            Some("docx" | "doc" | "pdf" | "xls" | "xlsx")
        )
    }

    #[test]
    fn check_if_has_valid_extension() {
        let test_cases = vec!["doc", "pdf", "xls", "docx", "xlsx", "ppt", "csv"];
        test_cases.into_iter().for_each(|case| {
            let valid = has_valid_extension(case);
            println!("{} {}", case, valid)
        });
    }

    #[test]
    fn build_extension_from_str() {
        let test_cases = vec!["doc", "pdf", "xls", "docx", "xlsx", "ppt", "csv"];
        test_cases.into_iter().for_each(|case| {
            let extension: Result<Extension, ParseError> = case.try_into();
            println!("{}: {}", case, extension.is_ok())
        });
    }

    #[test]
    fn check_tempfile() {
        let tempfile = TempFile {
            file_name: Some("example.docx".to_string()),
            file: tempfile::NamedTempFile::new().unwrap(),
            content_type: None,
            size: 100,
        };
        let document = DocumentFile::parse(tempfile).unwrap();
        println!("{}", document.name());
    }
}
