use serde::Deserialize;

#[derive(Deserialize)]
pub struct ItoolsSettings {
    pub proxy_route: String,
    pub word_to_pdf: String,
    pub pdf_to_html: String,
    pub docx_reader: String,
    pub pdfx_reader: String,
    pub xlsx_reader: String,
    pub splitting: String,
    pub reranking: String,
    pub embedding: String,
}

impl ItoolsSettings {
    pub fn word_to_pdf_proxy(&self) -> String {
        format!("{}{}", self.proxy_route, self.word_to_pdf)
    }

    pub fn pdf_to_html_proxy(&self) -> String {
        format!("{}{}", self.proxy_route, self.pdf_to_html)
    }

    pub fn docx_reader_proxy(&self) -> String {
        format!("{}{}", self.proxy_route, self.docx_reader)
    }

    pub fn pdfx_reader_proxy(&self) -> String {
        format!("{}{}", self.proxy_route, self.pdfx_reader)
    }

    pub fn xlsx_reader_proxy(&self) -> String {
        format!("{}{}", self.proxy_route, self.xlsx_reader)
    }

    pub fn embedding_proxy(&self) -> String {
        format!("{}{}", self.proxy_route, self.embedding)
    }

    pub fn reranking_proxy(&self) -> String {
        format!("{}{}", self.proxy_route, self.reranking)
    }

    pub fn splitting_proxy(&self) -> String {
        format!("{}{}", self.proxy_route, self.splitting)
    }
}
