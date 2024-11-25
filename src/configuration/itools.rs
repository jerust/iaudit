use serde::Deserialize;

#[derive(Deserialize)]
pub struct ItoolsSettings {
    pub convert_timeout_sec: u64,
    pub convert_word_to_pdf: String,
    pub convert_pdf_to_html: String,
    pub extract_pdf_slicing: String,
    pub extract_xls_slicing: String,
    pub vector_embedding_zh: String,
}
