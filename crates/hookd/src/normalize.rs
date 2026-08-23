//! Payload cru de um CLI → momento canônico.
//!
//! A fonte da verdade do mapa é `docs/pt-BR/specs/event-normalization.md`. Se este
//! arquivo e o spec discordarem, o spec está certo e o código está errado.
//!
//! **Regra de ouro:** evento desconhecido nunca é descartado. Vira
//! [`Momento::Erro`] carregando o nome cru, porque é assim que uma renomeação
//! silenciosa por um fornecedor aparece em vez de sumir (ADR-0007).

use cvb_core::{Evento, Origem, Transporte};
use serde_json::Value;

use crate::adapters;

pub fn normalizar(origem: Origem, transporte: Transporte, evento: &str, payload: &Value) -> Evento {
    let (momento, texto) = match origem {
        Origem::Claude => adapters::claude::mapear(evento, payload),
        Origem::Codex => adapters::codex::mapear(evento, payload),
        Origem::Copilot => adapters::copilot::mapear(evento, payload),
    };

    let mut e = Evento::novo(momento, origem, transporte);
    e.sessao_id =
        primeiro_texto(payload, &["session_id", "sessionId", "thread-id"]).unwrap_or_default();
    e.projeto = primeiro_texto(payload, &["cwd"]);
    e.texto = texto;
    e.detalhe = payload.clone();
    e
}

/// Procura a primeira chave presente, na ordem dada.
///
/// Existe porque os três CLIs usam grafias diferentes para o mesmo campo:
/// `session_id` no Claude, `sessionId` no Copilot, `thread-id` no `notify` do
/// Codex. Não é redundância — é o mapa dos dialetos.
pub fn primeiro_texto(payload: &Value, chaves: &[&str]) -> Option<String> {
    chaves
        .iter()
        .find_map(|c| payload.get(*c).and_then(|v| v.as_str()))
        .map(str::to_string)
}

/// Encurta um texto longo para caber numa fala.
///
/// Corte grosseiro e provisório. TODO: substituir pelo resumidor de verdade —
/// ver `docs/pt-BR/specs/speech-output.md`, que ainda não decidiu qual.
pub fn encurtar(texto: &str, maximo: usize) -> String {
    let limpo = texto.split_whitespace().collect::<Vec<_>>().join(" ");
    if limpo.chars().count() <= maximo {
        return limpo;
    }
    let corte: String = limpo.chars().take(maximo).collect();
    match corte.rfind(' ') {
        Some(i) if i > maximo / 2 => format!("{}…", &corte[..i]),
        _ => format!("{corte}…"),
    }
}

#[cfg(test)]
mod testes {
    use super::*;
    use cvb_core::Momento;

    #[test]
    fn evento_desconhecido_vira_erro_com_o_nome_cru() {
        let e = normalizar(
            Origem::Claude,
            Transporte::Hook,
            "EventoQueAindaNaoExiste",
            &serde_json::json!({}),
        );
        assert_eq!(e.momento, Momento::Erro);
        assert!(e.texto.contains("EventoQueAindaNaoExiste"));
    }

    #[test]
    fn sessao_e_lida_nos_tres_dialetos() {
        let claude = serde_json::json!({"session_id": "a"});
        let copilot = serde_json::json!({"sessionId": "b"});
        let codex = serde_json::json!({"thread-id": "c"});
        let chaves = ["session_id", "sessionId", "thread-id"];
        assert_eq!(primeiro_texto(&claude, &chaves).as_deref(), Some("a"));
        assert_eq!(primeiro_texto(&copilot, &chaves).as_deref(), Some("b"));
        assert_eq!(primeiro_texto(&codex, &chaves).as_deref(), Some("c"));
    }

    #[test]
    fn encurtar_respeita_a_fronteira_de_palavra() {
        let curto = encurtar("uma frase curta", 40);
        assert_eq!(curto, "uma frase curta");
        let longo = encurtar("uma frase razoavelmente longa que precisa caber", 20);
        assert!(longo.ends_with('…'));
        assert!(longo.chars().count() <= 21);
    }
}
