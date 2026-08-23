//! GitHub Copilot CLI — `camelCase` no evento e no payload.
//!
//! Verificado na versão 1.0.80. Mapa em `docs/pt-BR/specs/event-normalization.md`.
//!
//! Duas peculiaridades:
//!
//! - Os nomes de evento aceitam **duas grafias** (`preToolUse` e `PreToolUse`).
//!   Ambas são tratadas aqui, porque qual delas o CLI emite varia.
//! - O Copilot **não tem** evento de texto exibido. Narração contínua nele só
//!   sai por PTY ou `--output-format json`.

use cvb_core::Momento;
use serde_json::Value;

use crate::normalize::{encurtar, primeiro_texto};

pub fn mapear(evento: &str, p: &Value) -> (Momento, String) {
    // Normaliza a grafia antes de comparar: os dois estilos significam o mesmo.
    let chave = evento
        .strip_prefix(|c: char| c.is_ascii_uppercase())
        .map(|resto| {
            let inicial = evento.chars().next().unwrap_or('x').to_ascii_lowercase();
            format!("{inicial}{resto}")
        })
        .unwrap_or_else(|| evento.to_string());

    match chave.as_str() {
        "permissionRequest" => (
            Momento::DecisaoNecessaria,
            match primeiro_texto(p, &["toolName"]) {
                Some(f) => format!("quer usar {f}"),
                None => "pede uma decisão".into(),
            },
        ),

        "notification" => {
            let texto = primeiro_texto(p, &["message", "title"]).unwrap_or_default();
            match primeiro_texto(p, &["notification_type", "notificationType"]).as_deref() {
                Some("idle_prompt") => (Momento::EntradaNecessaria, texto),
                _ => (Momento::DecisaoNecessaria, texto),
            }
        }

        "agentStop" | "stop" => (
            Momento::TurnoConcluido,
            primeiro_texto(p, &["stopReason"]).unwrap_or_default(),
        ),

        "subagentStart" => (
            Momento::SubagenteIniciado,
            primeiro_texto(p, &["agentName"]).unwrap_or_else(|| "um subagente".into()),
        ),
        "subagentStop" => {
            let nome = primeiro_texto(p, &["agentName", "agentType"])
                .unwrap_or_else(|| "o subagente".into());
            let resumo = primeiro_texto(p, &["response"])
                .map(|t| encurtar(&t, 200))
                .unwrap_or_default();
            (
                Momento::SubagenteConcluido,
                if resumo.is_empty() {
                    format!("{nome} terminou")
                } else {
                    format!("{nome} terminou: {resumo}")
                },
            )
        }

        // A ferramenta `ask_user` é como o agente faz perguntas. Classificar
        // como `tool.started` seria silenciar exatamente o momento em que a
        // pessoa é necessária.
        "preToolUse" => match primeiro_texto(p, &["toolName"]).as_deref() {
            Some("ask_user") => (Momento::EntradaNecessaria, "tem uma pergunta".into()),
            Some(f) => (Momento::FerramentaIniciada, f.to_string()),
            None => (Momento::FerramentaIniciada, String::new()),
        },
        "postToolUse" => (
            Momento::FerramentaConcluida,
            primeiro_texto(p, &["toolName"]).unwrap_or_default(),
        ),
        "postToolUseFailure" => (
            Momento::FerramentaFalhou,
            primeiro_texto(p, &["toolName"]).unwrap_or_default(),
        ),

        "errorOccurred" => (
            Momento::Erro,
            primeiro_texto(p, &["errorContext"]).unwrap_or_else(|| "erro na sessão".into()),
        ),

        "preCompact" => (Momento::ContextoCompactando, String::new()),

        "sessionStart" => (
            Momento::SessaoIniciada,
            primeiro_texto(p, &["source"]).unwrap_or_default(),
        ),
        "sessionEnd" => (
            Momento::SessaoEncerrada,
            primeiro_texto(p, &["reason"]).unwrap_or_default(),
        ),

        _ => (
            Momento::Erro,
            format!("evento não mapeado do Copilot: {evento}"),
        ),
    }
}

#[cfg(test)]
mod testes {
    use super::*;

    #[test]
    fn as_duas_grafias_dao_no_mesmo() {
        let p = serde_json::json!({"toolName": "bash"});
        assert_eq!(mapear("preToolUse", &p).0, mapear("PreToolUse", &p).0);
        assert_eq!(mapear("agentStop", &p).0, mapear("AgentStop", &p).0);
    }

    #[test]
    fn ask_user_e_entrada_necessaria_nao_ferramenta() {
        let (m, _) = mapear("preToolUse", &serde_json::json!({"toolName": "ask_user"}));
        assert_eq!(m, Momento::EntradaNecessaria);

        let (m, _) = mapear("preToolUse", &serde_json::json!({"toolName": "bash"}));
        assert_eq!(m, Momento::FerramentaIniciada);
    }
}
