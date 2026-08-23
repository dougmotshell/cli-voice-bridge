//! Protocolo entre o daemon e os clientes locais.
//!
//! Uma mensagem JSON por linha, nos dois sentidos. Versionado no handshake:
//! cliente de versão incompatível recebe recusa explicativa em vez de
//! comportamento indefinido (ADR-0008).

use serde::{Deserialize, Serialize};

use crate::momento::{Evento, Origem, Transporte};

/// Versão do protocolo. Suba ao mudar o formato de forma incompatível.
pub const VERSAO_PROTOCOLO: u32 = 1;

/// O que um cliente pede ao daemon.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "tipo", rename_all = "snake_case")]
pub enum Requisicao {
    /// Handshake. Primeira mensagem de qualquer conexão.
    Ola {
        versao: u32,
        cliente: String,
    },

    /// Um payload cru vindo de um CLI, ainda por normalizar.
    ///
    /// É o que o `hookc` manda. A normalização acontece no daemon, não no
    /// cliente, porque o cliente precisa ser quase gratuito (ADR-0001).
    EventoBruto {
        origem: Origem,
        transporte: Transporte,
        /// Nome do evento como o CLI o chama, na grafia dele.
        evento: String,
        payload: serde_json::Value,
    },

    /// Estado do daemon, para `cvb daemon status` e para a GUI.
    Status,

    /// Fala um texto arbitrário. É o `cvb say`.
    Falar {
        texto: String,
    },

    /// Cala por um tempo. `None` significa até `Retomar`.
    Silenciar {
        segundos: Option<u64>,
    },

    Retomar,

    /// Assina o fluxo de momentos. É o `cvb events --follow` e o painel da GUI.
    Assinar,

    /// Lista as vozes cadastradas no `voice-clone`.
    Vozes,
}

/// O que o daemon responde.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "tipo", rename_all = "snake_case")]
pub enum Resposta {
    Ok,
    Erro {
        mensagem: String,
    },
    VersaoIncompativel {
        esperada: u32,
        recebida: u32,
    },
    Status {
        versao: String,
        sessoes: usize,
        fila: usize,
        silenciado: bool,
    },
    Vozes {
        vozes: Vec<String>,
    },
    /// Falou — e por qual caminho. `cvb say` mostra isso, porque "falou" e
    /// "falou pela voz de emergência" são resultados bem diferentes.
    Falado {
        como: String,
    },
    /// Um momento normalizado, empurrado para quem assinou.
    Momento {
        evento: Evento,
    },
}

#[cfg(test)]
mod testes {
    use super::*;

    #[test]
    fn requisicao_vai_e_volta() {
        let r = Requisicao::EventoBruto {
            origem: Origem::Claude,
            transporte: Transporte::Hook,
            evento: "PermissionRequest".into(),
            payload: serde_json::json!({"tool_name": "Bash"}),
        };
        let linha = serde_json::to_string(&r).expect("serializa");
        assert!(linha.contains("\"tipo\":\"evento_bruto\""));
        let volta: Requisicao = serde_json::from_str(&linha).expect("desserializa");
        match volta {
            Requisicao::EventoBruto { evento, .. } => assert_eq!(evento, "PermissionRequest"),
            outro => panic!("variante errada: {outro:?}"),
        }
    }
}
