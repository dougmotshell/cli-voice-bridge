//! Codex CLI — hooks em `PascalCase`, mais o `notify` em `kebab-case`.
//!
//! Verificado na versão 0.147.0. Mapa em `docs/specs/event-normalization.md`.
//!
//! Duas peculiaridades que custam tempo se esquecidas:
//!
//! - O `notify` do `config.toml` é o **único** lugar do sistema com
//!   `kebab-case` (`last-assistant-message`), e o campo pode faltar.
//! - O Codex guarda um `trusted_hash` do comando de hook em `config.toml`.
//!   Mudar o comando deixa o hook inerte até a pessoa confirmar numa sessão —
//!   e isso se parece exatamente com "o evento não dispara".
//!
//! TODO: confirmar se `PreToolUse`/`PostToolUse` valem só para a ferramenta
//! Bash nesta versão. Há relato de terceiros, sem confirmação — usar a skill
//! `map-cli-events`.

use cvb_core::Momento;
use serde_json::Value;

use crate::normalize::{encurtar, primeiro_texto};

pub fn mapear(evento: &str, p: &Value) -> (Momento, String) {
    match evento {
        // --- via `notify` ---
        "agent-turn-complete" => (
            Momento::TurnoConcluido,
            primeiro_texto(p, &["last-assistant-message"])
                .map(|t| encurtar(&t, 200))
                .unwrap_or_default(),
        ),

        // --- via hooks ---
        "PermissionRequest" => (
            Momento::DecisaoNecessaria,
            match primeiro_texto(p, &["tool_name"]) {
                Some(f) => format!("quer usar {f}"),
                None => "pede uma decisão".into(),
            },
        ),

        "Stop" => (
            Momento::TurnoConcluido,
            primeiro_texto(p, &["last_assistant_message"])
                .map(|t| encurtar(&t, 200))
                .unwrap_or_default(),
        ),

        "SubagentStart" => (
            Momento::SubagenteIniciado,
            primeiro_texto(p, &["agent_type", "agent_name"])
                .unwrap_or_else(|| "um subagente".into()),
        ),
        "SubagentStop" => (
            Momento::SubagenteConcluido,
            primeiro_texto(p, &["agent_type", "agent_name"])
                .unwrap_or_else(|| "o subagente".into()),
        ),

        "PreToolUse" => (
            Momento::FerramentaIniciada,
            primeiro_texto(p, &["tool_name"]).unwrap_or_default(),
        ),
        "PostToolUse" => (
            Momento::FerramentaConcluida,
            primeiro_texto(p, &["tool_name"]).unwrap_or_default(),
        ),

        // Serve para cortar a fala em curso: se a pessoa está digitando, ela já
        // voltou (`docs/specs/speech-output.md`). Não é para ser falado.
        "UserPromptSubmit" => (Momento::TextoDeMensagem, String::new()),

        "PreCompact" | "PostCompact" => (Momento::ContextoCompactando, String::new()),

        "SessionStart" => (Momento::SessaoIniciada, String::new()),
        "SessionEnd" => (Momento::SessaoEncerrada, String::new()),

        desconhecido => (
            Momento::Erro,
            format!("evento não mapeado do Codex: {desconhecido}"),
        ),
    }
}

#[cfg(test)]
mod testes {
    use super::*;

    #[test]
    fn notify_usa_kebab_case() {
        // O único lugar do sistema com essa grafia. Fácil de escrever errado.
        let (m, t) = mapear(
            "agent-turn-complete",
            &serde_json::json!({"last-assistant-message": "pronto"}),
        );
        assert_eq!(m, Momento::TurnoConcluido);
        assert_eq!(t, "pronto");
    }

    #[test]
    fn notify_sem_a_mensagem_ainda_vira_turno_concluido() {
        // O campo pode faltar; isso não pode virar erro.
        let (m, t) = mapear("agent-turn-complete", &serde_json::json!({"turn-id": "1"}));
        assert_eq!(m, Momento::TurnoConcluido);
        assert!(t.is_empty());
    }
}
