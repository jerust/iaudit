use serde::Deserialize;

#[derive(Deserialize)]
pub struct ItoolsSettings {
    pub proxy_timeout: u64,
    pub proxy_address: String,
    pub word_to_pdf: String,
    pub pdf_to_html: String,
    pub excel_reader: String,
    pub docx_reader: String,
    pub pdf_reader: String,
    pub splitter: String,
    pub reranker: String,
    pub embedding: String,
}

impl ItoolsSettings {
    pub fn word_to_pdf_proxy(&self) -> String {
        format!("{}{}", self.proxy_address, self.word_to_pdf)
    }

    pub fn pdf_to_html_proxy(&self) -> String {
        format!("{}{}", self.proxy_address, self.pdf_to_html)
    }

    pub fn excel_reader_proxy(&self) -> String {
        format!("{}{}", self.proxy_address, self.excel_reader)
    }

    pub fn docx_reader_proxy(&self) -> String {
        format!("{}{}", self.proxy_address, self.docx_reader)
    }

    pub fn pdf_reader_proxy(&self) -> String {
        format!("{}{}", self.proxy_address, self.pdf_reader)
    }

    pub fn splitter_proxy(&self) -> String {
        format!("{}{}", self.proxy_address, self.splitter)
    }

    pub fn reranker_proxy(&self) -> String {
        format!("{}{}", self.proxy_address, self.reranker)
    }

    pub fn embedding_proxy(&self) -> String {
        format!("{}{}", self.proxy_address, self.embedding)
    }
}
