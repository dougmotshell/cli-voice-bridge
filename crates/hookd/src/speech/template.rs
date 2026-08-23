//! Momento vira frase curta em pt-BR.
//!
//! O molde de cada momento é sobrescrevível em `[momentos."<nome>"] molde`
//! (`docs/pt-BR/specs/configuration.md`). Os marcadores substituídos são
//! `{cli}`, `{texto}` e `{ferramenta}`.

use cvb_core::config::Config;
use cvb_core::{Evento, Momento};

/// Frase pronta para sintetizar, ou `None` quando o momento não vira fala.
pub fn frase(evento: &Evento, config: &Config) -> Option<String> {
    // Modo discreto fala a categoria e nunca o conteúdo: use quando houver
    // outras pessoas ouvindo (`docs/pt-BR/specs/speech-output.md`).
    if config.privacidade.modo_discreto {
        return categoria(evento.momento).map(|c| c.to_string());
    }

    let molde = config
        .momentos
        .get(evento.momento.nome())
        .and_then(|m| m.molde.clone())
        .unwrap_or_else(|| molde_padrao(evento.momento).to_string());

    if molde.is_empty() {
        return None;
    }

    let preenchida = molde
        .replace("{cli}", evento.origem.falado())
        .replace("{ferramenta}", &evento.texto)
        .replace("{texto}", &evento.texto);

    let limpa = preenchida.split_whitespace().collect::<Vec<_>>().join(" ");
    if limpa.is_empty() {
        None
    } else {
        Some(limpa)
    }
}

/// O que se fala no modo discreto: a categoria, sem nenhum conteúdo.
fn categoria(momento: Momento) -> Option<&'static str> {
    Some(match momento {
        Momento::DecisaoNecessaria => "O agente precisa de uma decisão.",
        Momento::EntradaNecessaria => "O agente está esperando você.",
        Momento::TurnoConcluido => "O agente terminou.",
        Momento::TurnoFalhou | Momento::Erro => "O agente encontrou um erro.",
        Momento::SubagenteConcluido => "Um subagente terminou.",
        Momento::SubagenteIniciado => "Um subagente começou.",
        Momento::TarefaConcluida => "Uma tarefa foi concluída.",
        Momento::FerramentaFalhou => "Uma ferramenta falhou.",
        _ => return None,
    })
}

/// Molde embutido. Vazio significa "este momento não vira fala por padrão".
fn molde_padrao(momento: Momento) -> &'static str {
    match momento {
        Momento::DecisaoNecessaria => "{cli} {texto}. Autorizo?",
        Momento::EntradaNecessaria => "{cli} {texto}.",
        Momento::TurnoConcluido => "{cli} terminou. {texto}",
        Momento::TurnoFalhou => "{cli} parou com erro.",
        Momento::Erro => "{cli}: {texto}",
        Momento::SubagenteIniciado => "{cli} abriu o subagente {texto}.",
        Momento::SubagenteConcluido => "{cli}: {texto}",
        Momento::TarefaConcluida => "{cli} concluiu {texto}.",
        Momento::FerramentaFalhou => "No {cli}, a ferramenta {ferramenta} falhou.",
        _ => "",
    }
}

#[cfg(test)]
mod testes {
    use super::*;
    use cvb_core::{Origem, Transporte};

    fn evento(momento: Momento, texto: &str) -> Evento {
        let mut e = Evento::novo(momento, Origem::Claude, Transporte::Hook);
        e.texto = texto.to_string();
        e
    }

    #[test]
    fn decisao_vira_pergunta_com_o_nome_do_cli() {
        let f = frase(
            &evento(Momento::DecisaoNecessaria, "quer usar Bash"),
            &Config::default(),
        );
        assert_eq!(f.as_deref(), Some("Claude quer usar Bash. Autorizo?"));
    }

    #[test]
    fn momento_silencioso_nao_vira_fala() {
        assert!(frase(
            &evento(Momento::FerramentaIniciada, "Bash"),
            &Config::default()
        )
        .is_none());
        assert!(frase(&evento(Momento::TextoDeMensagem, "oi"), &Config::default()).is_none());
    }

    #[test]
    fn modo_discreto_nunca_diz_o_conteudo() {
        let mut c = Config::default();
        c.privacidade.modo_discreto = true;
        let f = frase(
            &evento(Momento::DecisaoNecessaria, "rodar rm -rf /dados"),
            &c,
        )
        .unwrap();
        assert!(!f.contains("rm -rf"), "vazou o conteúdo: {f}");
        assert!(f.contains("decisão"));
    }

    #[test]
    fn molde_da_configuracao_vence_o_embutido() {
        let mut c = Config::default();
        c.momentos.insert(
            "turn.finished".into(),
            cvb_core::config::ConfigMomento {
                molde: Some("acabou no {cli}".into()),
                ..Default::default()
            },
        );
        let f = frase(&evento(Momento::TurnoConcluido, "ignorado"), &c);
        assert_eq!(f.as_deref(), Some("acabou no Claude"));
    }

    #[test]
    fn texto_vazio_nao_deixa_espaco_duplo() {
        let f = frase(&evento(Momento::TurnoConcluido, ""), &Config::default()).unwrap();
        assert!(!f.contains("  "), "{f}");
    }
}
