use dotenv::dotenv;
use once_cell::sync::Lazy;
use reqwest::Client;
use serde::{Deserialize, Serialize};
use serde_json::json;
use std::env;

const MAX_WORDS_FOR_SUMMARY: usize = 500;
const OPENAI_API_ENDPOINT: &str = "https://api.openai.com/v1/chat/completions";

// Carrega a API key uma única vez durante a execução do programa
static API_KEY: Lazy<String> = Lazy::new(|| {
    dotenv().ok(); // Carrega o arquivo .env se existir
    env::var("API_KEY").unwrap_or_else(|_| {
        eprintln!("⚠️ API_KEY não encontrada no ambiente");
        String::new()
    })
});

pub struct PdfContent {
    pub text: String,
    pub filename: String,
    pub size: usize,
}

impl PdfContent {
    pub fn new(text: String, filename: String) -> Self {
        let size = text.len();
        Self {
            text,
            filename,
            size,
        }
    }

    // Retorna um resumo básico do conteúdo
    pub fn get_summary(&self) -> String {
        format!(
            "📑 Arquivo: {}\n📊 Tamanho: {} bytes\n\n📄 Conteúdo:\n{}\n",
            self.filename, self.size, self.text
        )
    }

    // Limita o texto a 500 palavras
    fn get_limited_text(&self) -> String {
        self.text
            .split_whitespace()
            .take(MAX_WORDS_FOR_SUMMARY)
            .collect::<Vec<_>>()
            .join(" ")
    }

    // Gera um resumo do conteúdo usando GPT
    pub async fn generate_summary(&self) -> Result<String, Box<dyn std::error::Error>> {
        // Validação da API key
        if API_KEY.is_empty() {
            return Err("API key não configurada. Defina a variável de ambiente API_KEY.".into());
        }

        let client = Client::new();
        let limited_text = self.get_limited_text();

        let body = json!({
            "model": "gpt-4o-mini",
            "messages": [
                {
                    "role": "user",
                    "content": format!("Resuma o seguinte texto em português, mantendo os pontos principais: {}", limited_text)
                }
            ],
            "max_tokens": 200
        });

        let resp = client
            .post(OPENAI_API_ENDPOINT)
            .bearer_auth(&*API_KEY)
            .json(&body)
            .send()
            .await;

        let response = match resp {
            Ok(r) => r,
            Err(e) => {
                return Err(format!("Erro ao enviar requisição para a OpenAI: {}", e).into());
            }
        };

        if !response.status().is_success() {
            let status = response.status();
            let text = response.text().await.unwrap_or_default();
            return Err(format!("Erro HTTP {}: {}", status, text).into());
        }

        let parsed = response.json::<ChatGPTResponse>().await;
        let chat_response = match parsed {
            Ok(r) => r,
            Err(e) => {
                return Err(format!("Erro ao interpretar resposta da OpenAI: {}", e).into());
            }
        };

        if let Some(choice) = chat_response.choices.first() {
            Ok(format!(
                "Resumo do arquivo {}:\n\n{}",
                self.filename, choice.message.content
            ))
        } else {
            Err("Nenhum resumo foi gerado pela API.".into())
        }
    }
}

// Função que processa o conteúdo do PDF
pub fn process_pdf_content(content: String, filename: String) -> PdfContent {
    // Por enquanto apenas criamos o objeto PdfContent
    // Aqui você pode adicionar mais processamento conforme necessário
    PdfContent::new(content, filename)
}

// Função que processa o conteúdo do PDF com resumo
pub async fn process_pdf_content_with_summary(
    content: String,
    filename: String,
) -> Result<String, Box<dyn std::error::Error>> {
    let pdf_content = PdfContent::new(content, filename);
    match pdf_content.generate_summary().await {
        Ok(summary) => Ok(summary),
        Err(e) => Err(format!("Erro ao gerar resumo: {}", e).into()),
    }
}

// Função auxiliar para verificar se a API key está configurada
pub fn check_api_key() -> bool {
    !API_KEY.is_empty()
}

// Função para teste da API key
pub fn get_api_key_status() -> String {
    if check_api_key() {
        let masked_key = API_KEY.chars().take(8).collect::<String>() + "...";
        format!("✅ API key configurada: {}", masked_key)
    } else {
        "❌ API key não encontrada ou vazia".to_string()
    }
}

#[derive(Debug, Serialize, Deserialize)]
struct ChatGPTResponse {
    choices: Vec<Choice>,
}

#[derive(Debug, Serialize, Deserialize)]
struct Choice {
    message: Message,
}

#[derive(Debug, Serialize, Deserialize)]
struct Message {
    content: String,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_api_key_loading() {
        assert!(check_api_key(), "API key deveria estar configurada");
    }
}
