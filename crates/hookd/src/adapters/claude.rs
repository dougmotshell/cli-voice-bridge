//! Claude Code — evento em `PascalCase`, payload em `snake_case`.
//!
//! Verificado na versão 2.1.241. Mapa em `docs/specs/event-normalization.md`.
//!
//! TODO: o Claude tem cerca de 30 eventos e aqui está mapeada pouco mais da
//! metade. Os que faltam estão listados no spec, em *Questões em aberto*.

use cvb_core::Momento;
use serde_json::Value;

use crate::normalize::{encurtar, primeiro_texto};

pub fn mapear(evento: &str, p: &Value) -> (Momento, String) {
    match evento {
        "PermissionRequest" => (
            Momento::DecisaoNecessaria,
            match primeiro_texto(p, &["tool_name"]) {
                Some(f) => format!("quer usar {f}"),
                None => "pede uma decisão".into(),
            },
        ),

        "Elicitation" => (
            Momento::DecisaoNecessaria,
            match primeiro_texto(p, &["mcp_server_name"]) {
                Some(s) => format!("o servidor {s} pede uma resposta"),
                None => "pede uma resposta".into(),
            },
        ),

        // `Notification` é o mesmo evento para coisas bem diferentes: o que
        // separa é o `notification_type`. Por isso o hook deve usar `matcher`
        // em vez de assinar tudo e filtrar aqui.
        "Notification" => match primeiro_texto(p, &["notification_type"]).as_deref() {
            Some("idle_prompt") => (Momento::EntradaNecessaria, "está esperando você".into()),
            Some("permission_prompt") => (Momento::DecisaoNecessaria, "pede permissão".into()),
            Some(outro) => (Momento::SessaoIniciada, outro.to_string()),
            None => (Momento::SessaoIniciada, String::new()),
        },

        "Stop" => (
            Momento::TurnoConcluido,
            texto_do_assistente(p, &["last_assistant_message"]),
        ),
        "StopFailure" => (Momento::TurnoFalhou, "o turno terminou com erro".into()),

        "SubagentStart" => (
            Momento::SubagenteIniciado,
            primeiro_texto(p, &["agent_type"]).unwrap_or_else(|| "um subagente".into()),
        ),
        "SubagentStop" => {
            let tipo = primeiro_texto(p, &["agent_type"]).unwrap_or_else(|| "o subagente".into());
            let resumo = texto_do_assistente(p, &["last_assistant_message"]);
            (
                Momento::SubagenteConcluido,
                if resumo.is_empty() {
                    format!("{tipo} terminou")
                } else {
                    format!("{tipo} terminou: {resumo}")
                },
            )
        }

        "TaskCreated" => (
            Momento::TarefaCriada,
            primeiro_texto(p, &["task_name"]).unwrap_or_default(),
        ),
        "TaskCompleted" => (
            Momento::TarefaConcluida,
            primeiro_texto(p, &["task_name"]).unwrap_or_default(),
        ),

        "PostToolUseFailure" => (
            Momento::FerramentaFalhou,
            primeiro_texto(p, &["tool_name"]).unwrap_or_default(),
        ),
        "PermissionDenied" => (
            Momento::Erro,
            match primeiro_texto(p, &["tool_name"]) {
                Some(f) => format!("permissão negada para {f}"),
                None => "permissão negada".into(),
            },
        ),

        "PreToolUse" => (
            Momento::FerramentaIniciada,
            primeiro_texto(p, &["tool_name"]).unwrap_or_default(),
        ),
        "PostToolUse" => (
            Momento::FerramentaConcluida,
            primeiro_texto(p, &["tool_name"]).unwrap_or_default(),
        ),

        "MessageDisplay" => (
            Momento::TextoDeMensagem,
            primeiro_texto(p, &["message_text"]).unwrap_or_default(),
        ),

        "PreCompact" | "PostCompact" => (Momento::ContextoCompactando, String::new()),

        "SessionStart" => (
            Momento::SessaoIniciada,
            primeiro_texto(p, &["session_start_reason"]).unwrap_or_default(),
        ),
        "SessionEnd" => (
            Momento::SessaoEncerrada,
            primeiro_texto(p, &["session_end_reason"]).unwrap_or_default(),
        ),

        desconhecido => (
            Momento::Erro,
            format!("evento não mapeado do Claude: {desconhecido}"),
        ),
    }
}

/// A mensagem do assistente vem em parágrafos; falar tudo é inútil.
fn texto_do_assistente(p: &Value, chaves: &[&str]) -> String {
    primeiro_texto(p, chaves)
        .map(|t| encurtar(&t, 200))
        .unwrap_or_default()
}

#[cfg(test)]
mod testes {
    use super::*;

    #[test]
    fn permissao_vira_decisao_necessaria() {
        let (m, t) = mapear(
            "PermissionRequest",
            &serde_json::json!({"tool_name": "Bash"}),
        );
        assert_eq!(m, Momento::DecisaoNecessaria);
        assert!(t.contains("Bash"));
    }

    #[test]
    fn notification_ocioso_e_diferente_de_permissao() {
        let (ocioso, _) = mapear(
            "Notification",
            &serde_json::json!({"notification_type": "idle_prompt"}),
        );
        let (perm, _) = mapear(
            "Notification",
            &serde_json::json!({"notification_type": "permission_prompt"}),
        );
        assert_eq!(ocioso, Momento::EntradaNecessaria);
        assert_eq!(perm, Momento::DecisaoNecessaria);
    }

    #[test]
    fn stop_encurta_a_mensagem_do_assistente() {
        let longa = "palavra ".repeat(200);
        let (m, t) = mapear(
            "Stop",
            &serde_json::json!({"last_assistant_message": longa}),
        );
        assert_eq!(m, Momento::TurnoConcluido);
        assert!(t.chars().count() <= 201);
    }
}
