//! Cliente do sidecar de síntese.
//!
//! O sidecar é um processo Python de vida longa que carrega o XTTS-v2 uma vez e
//! atende por socket local (ADR-0003). Aqui só se conversa com ele.
//!
//! **Indisponível não é exceção, é rotina.** O sidecar pode não ter subido, pode
//! ter morrido, pode estar carregando o modelo. Quem chama trata o erro caindo
//! para a voz do sistema — nunca ficando mudo.

use std::io::BufRead;
use std::path::{Path, PathBuf};

use crate::caminhos;
use crate::ipc;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize)]
#[serde(tag = "tipo", rename_all = "snake_case")]
enum Pedido<'a> {
    Ping,
    Vozes,
    Sintetizar {
        texto: &'a str,
        voz: &'a str,
        idioma: &'a str,
        saida: &'a str,
    },
}

#[derive(Debug, Deserialize)]
#[serde(tag = "tipo", rename_all = "snake_case")]
enum Retorno {
    Ok {
        #[serde(default)]
        duracao_s: f32,
    },
    Vozes {
        vozes: Vec<String>,
    },
    Erro {
        mensagem: String,
    },
}

#[derive(Debug)]
pub struct ErroSidecar(pub String);

impl std::fmt::Display for ErroSidecar {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.0)
    }
}

pub struct Sidecar {
    endereco: PathBuf,
}

impl Sidecar {
    pub fn novo() -> Sidecar {
        Sidecar {
            endereco: caminhos::endereco_sidecar(),
        }
    }

    pub fn endereco(&self) -> &Path {
        &self.endereco
    }

    /// Está de pé e respondendo?
    pub fn vivo(&self) -> bool {
        matches!(self.conversar(&Pedido::Ping), Ok(Retorno::Ok { .. }))
    }

    pub fn vozes(&self) -> Result<Vec<String>, ErroSidecar> {
        match self.conversar(&Pedido::Vozes)? {
            Retorno::Vozes { vozes } => Ok(vozes),
            Retorno::Erro { mensagem } => Err(ErroSidecar(mensagem)),
            outro => Err(ErroSidecar(format!("resposta inesperada: {outro:?}"))),
        }
    }

    /// Sintetiza `texto` em `saida`. Devolve a duração do áudio, em segundos.
    pub fn sintetizar(
        &self,
        texto: &str,
        voz: &str,
        idioma: &str,
        saida: &Path,
    ) -> Result<f32, ErroSidecar> {
        let pedido = Pedido::Sintetizar {
            texto,
            voz,
            idioma,
            saida: &saida.to_string_lossy(),
        };
        match self.conversar(&pedido)? {
            Retorno::Ok { duracao_s, .. } => Ok(duracao_s),
            Retorno::Erro { mensagem } => Err(ErroSidecar(mensagem)),
            outro => Err(ErroSidecar(format!("resposta inesperada: {outro:?}"))),
        }
    }

    fn conversar(&self, pedido: &Pedido<'_>) -> Result<Retorno, ErroSidecar> {
        let mut conexao = ipc::conectar(&self.endereco).map_err(|e| {
            ErroSidecar(format!(
                "sidecar indisponível em {}: {e}",
                self.endereco.display()
            ))
        })?;

        ipc::enviar_linha(&mut conexao, pedido)
            .map_err(|e| ErroSidecar(format!("envio falhou: {e}")))?;

        let mut leitor = std::io::BufReader::new(conexao);
        let mut linha = String::new();
        leitor
            .read_line(&mut linha)
            .map_err(|e| ErroSidecar(format!("leitura falhou: {e}")))?;

        if linha.trim().is_empty() {
            return Err(ErroSidecar("sidecar fechou sem responder".into()));
        }
        serde_json::from_str(linha.trim())
            .map_err(|e| ErroSidecar(format!("resposta não decodifica: {e}")))
    }
}

impl Default for Sidecar {
    fn default() -> Self {
        Sidecar::novo()
    }
}
