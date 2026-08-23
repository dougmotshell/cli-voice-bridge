//! `cvb-hook` — o cliente de hook.
//!
//! Roda **em série com o agente de IA**, centenas de vezes por sessão. Tudo
//! aqui é medido nesse orçamento: ler o payload, abrir o socket, despejar, sair.
//! Nada de carregar modelo, abrir rede ou fazer parsing além do necessário para
//! rotear (ADR-0001).
//!
//! **Nunca falha visivelmente.** Daemon fora do ar, socket ausente, payload
//! estranho — tudo sai com código 0 e em silêncio. Um hook que quebra é um
//! agente de IA que trava, e isso é pior que ficar sem voz.
//!
//! Uso:
//!
//! ```text
//! cvb-hook --origem claude --transporte hook            # payload no stdin
//! cvb-hook --origem codex --transporte notify '<json>'  # payload no argumento
//! ```
//!
//! O Codex passa o JSON como argumento final em vez de pelo stdin — daí a
//! segunda forma. Ver `docs/specs/capture-transports.md`.

use std::io::Read;
use std::str::FromStr;

use cvb_core::caminhos;
use cvb_core::ipc;
use cvb_core::{Origem, Requisicao, Transporte, VERSAO_PROTOCOLO};

fn main() {
    // O código de saída é sempre 0, aconteça o que acontecer.
    if let Err(e) = executar() {
        if std::env::var_os("CVB_DEBUG").is_some() {
            eprintln!("cvb-hook: {e}");
        }
    }
}

struct Argumentos {
    origem: Origem,
    transporte: Transporte,
    evento: Option<String>,
    payload_literal: Option<String>,
}

fn executar() -> Result<(), String> {
    let args = analisar(std::env::args().skip(1))?;

    let bruto = match args.payload_literal {
        Some(texto) => texto,
        None => ler_stdin()?,
    };

    let payload: serde_json::Value =
        serde_json::from_str(bruto.trim()).unwrap_or(serde_json::Value::Null);

    // O nome do evento pode vir do argumento ou do próprio payload — cada CLI
    // usa uma chave diferente, e o daemon é quem sabe qual (ADR-0001: aqui não
    // se interpreta, só se roteia).
    let evento = args
        .evento
        .or_else(|| nome_do_evento_no_payload(&payload))
        .unwrap_or_default();

    let mut conexao = ipc::conectar(&caminhos::endereco_daemon())
        .map_err(|e| format!("daemon indisponível: {e}"))?;

    let ola = Requisicao::Ola {
        versao: VERSAO_PROTOCOLO,
        cliente: "hookc".into(),
    };
    ipc::enviar_linha(&mut conexao, &ola).map_err(|e| e.to_string())?;

    let msg = Requisicao::EventoBruto {
        origem: args.origem,
        transporte: args.transporte,
        evento,
        payload,
    };
    ipc::enviar_linha(&mut conexao, &msg).map_err(|e| e.to_string())?;

    // Não esperamos resposta: o agente está bloqueado enquanto estivermos vivos.
    // Decisão de permissão por voz vai precisar esperar, e é a única exceção —
    // ver docs/specs/speech-input.md quando for implementada.
    Ok(())
}

fn analisar<I: Iterator<Item = String>>(mut it: I) -> Result<Argumentos, String> {
    let mut origem = None;
    let mut transporte = None;
    let mut evento = None;
    let mut literal = None;

    while let Some(arg) = it.next() {
        match arg.as_str() {
            "--origem" => {
                origem = Some(Origem::from_str(&it.next().unwrap_or_default())?);
            }
            "--transporte" => {
                transporte = Some(Transporte::from_str(&it.next().unwrap_or_default())?);
            }
            "--evento" => {
                evento = it.next();
            }
            outro if outro.starts_with("--") => {
                return Err(format!("argumento desconhecido: {outro}"));
            }
            // Posicional: é o JSON que o Codex acrescenta ao `notify`.
            outro => literal = Some(outro.to_string()),
        }
    }

    Ok(Argumentos {
        origem: origem.ok_or("falta --origem")?,
        transporte: transporte.unwrap_or(Transporte::Hook),
        evento,
        payload_literal: literal,
    })
}

fn ler_stdin() -> Result<String, String> {
    let mut buf = String::new();
    std::io::stdin()
        .read_to_string(&mut buf)
        .map_err(|e| format!("stdin: {e}"))?;
    Ok(buf)
}

/// Onde cada CLI guarda o nome do evento dentro do próprio payload.
///
/// Três dialetos, três chaves. O daemon reconhece as três, mas adiantar aqui
/// evita uma ida e volta quando o hook não foi configurado com `--evento`.
fn nome_do_evento_no_payload(payload: &serde_json::Value) -> Option<String> {
    for chave in ["hook_event_name", "hookEventName", "type"] {
        if let Some(v) = payload.get(chave).and_then(|v| v.as_str()) {
            return Some(v.to_string());
        }
    }
    None
}

#[cfg(test)]
mod testes {
    use super::*;

    #[test]
    fn analisa_forma_do_claude() {
        let args = analisar(
            ["--origem", "claude", "--transporte", "hook"]
                .into_iter()
                .map(String::from),
        )
        .expect("analisa");
        assert_eq!(args.origem, Origem::Claude);
        assert_eq!(args.transporte, Transporte::Hook);
        assert!(args.payload_literal.is_none());
    }

    #[test]
    fn analisa_forma_do_notify_do_codex() {
        // O Codex acrescenta o JSON como argumento final.
        let args = analisar(
            [
                "--origem",
                "codex",
                "--transporte",
                "notify",
                r#"{"type":"agent-turn-complete"}"#,
            ]
            .into_iter()
            .map(String::from),
        )
        .expect("analisa");
        assert_eq!(args.transporte, Transporte::Notify);
        assert!(args.payload_literal.is_some());
    }

    #[test]
    fn descobre_o_evento_nos_tres_dialetos() {
        let claude = serde_json::json!({"hook_event_name": "Stop"});
        let copilot = serde_json::json!({"hookEventName": "agentStop"});
        let codex = serde_json::json!({"type": "agent-turn-complete"});
        assert_eq!(nome_do_evento_no_payload(&claude).as_deref(), Some("Stop"));
        assert_eq!(
            nome_do_evento_no_payload(&copilot).as_deref(),
            Some("agentStop")
        );
        assert_eq!(
            nome_do_evento_no_payload(&codex).as_deref(),
            Some("agent-turn-complete")
        );
    }
}
