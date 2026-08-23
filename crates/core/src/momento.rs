//! O vocabulário canônico: momentos, origens, transportes e urgência.
//!
//! Isto é a única coisa que atravessa a fronteira entre os adaptadores e o
//! resto do sistema (ADR-0007). Política, fila, GUI e configuração só conhecem
//! momentos — nunca o payload cru de um CLI.
//!
//! A definição de cada momento está em `docs/specs/event-normalization.md`.

use serde::{Deserialize, Serialize};

/// Qual CLI de IA gerou o acontecimento.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Origem {
    Claude,
    Codex,
    Copilot,
}

impl Origem {
    /// Nome para falar em voz alta. Não é o mesmo que o nome serializado.
    pub fn falado(self) -> &'static str {
        match self {
            Origem::Claude => "Claude",
            Origem::Codex => "Codex",
            Origem::Copilot => "Copilot",
        }
    }
}

impl std::str::FromStr for Origem {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.trim().to_ascii_lowercase().as_str() {
            "claude" => Ok(Origem::Claude),
            "codex" => Ok(Origem::Codex),
            "copilot" => Ok(Origem::Copilot),
            outro => Err(format!("origem desconhecida: {outro}")),
        }
    }
}

/// Por qual caminho o acontecimento chegou.
///
/// Separado de [`Origem`] de propósito: o mesmo momento pode chegar por dois
/// transportes e precisa ser deduplicado. Ver [`Transporte::confianca`].
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum Transporte {
    Hook,
    Notify,
    Pty,
    StreamJson,
    Acp,
}

impl Transporte {
    /// Quem vence na deduplicação: maior número ganha.
    ///
    /// A ordem é `hook > acp > stream-json > notify > pty`, e não é arbitrária:
    /// hook é payload estruturado do próprio CLI, pty é leitura de tela.
    pub fn confianca(self) -> u8 {
        match self {
            Transporte::Hook => 5,
            Transporte::Acp => 4,
            Transporte::StreamJson => 3,
            Transporte::Notify => 2,
            Transporte::Pty => 1,
        }
    }
}

impl std::str::FromStr for Transporte {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.trim().to_ascii_lowercase().as_str() {
            "hook" => Ok(Transporte::Hook),
            "notify" => Ok(Transporte::Notify),
            "pty" => Ok(Transporte::Pty),
            "stream-json" | "stream_json" => Ok(Transporte::StreamJson),
            "acp" => Ok(Transporte::Acp),
            outro => Err(format!("transporte desconhecido: {outro}")),
        }
    }
}

/// Quanto o momento merece interromper a pessoa.
///
/// A ordem importa: `Ord` é usado pela fila de fala para decidir prioridade e
/// corte. Não reordene as variantes sem ler `docs/specs/speech-output.md`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Urgencia {
    Silenciosa,
    Baixa,
    Media,
    Alta,
    Critica,
}

/// O vocabulário fechado de momentos.
///
/// Acrescentar variante aqui é mexer no contrato: a configuração da pessoa fala
/// nestes nomes, e um nome que muda quebra o `config.toml` de quem já usava.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Momento {
    #[serde(rename = "session.started")]
    SessaoIniciada,
    #[serde(rename = "session.ended")]
    SessaoEncerrada,
    #[serde(rename = "turn.finished")]
    TurnoConcluido,
    #[serde(rename = "turn.failed")]
    TurnoFalhou,
    #[serde(rename = "decision.needed")]
    DecisaoNecessaria,
    #[serde(rename = "input.needed")]
    EntradaNecessaria,
    #[serde(rename = "subagent.started")]
    SubagenteIniciado,
    #[serde(rename = "subagent.finished")]
    SubagenteConcluido,
    #[serde(rename = "task.created")]
    TarefaCriada,
    #[serde(rename = "task.completed")]
    TarefaConcluida,
    #[serde(rename = "tool.started")]
    FerramentaIniciada,
    #[serde(rename = "tool.finished")]
    FerramentaConcluida,
    #[serde(rename = "tool.failed")]
    FerramentaFalhou,
    #[serde(rename = "context.compacting")]
    ContextoCompactando,
    #[serde(rename = "message.text")]
    TextoDeMensagem,
    #[serde(rename = "error")]
    Erro,
}

impl Momento {
    /// O nome estável, o mesmo que aparece no `config.toml` da pessoa.
    pub fn nome(self) -> &'static str {
        match self {
            Momento::SessaoIniciada => "session.started",
            Momento::SessaoEncerrada => "session.ended",
            Momento::TurnoConcluido => "turn.finished",
            Momento::TurnoFalhou => "turn.failed",
            Momento::DecisaoNecessaria => "decision.needed",
            Momento::EntradaNecessaria => "input.needed",
            Momento::SubagenteIniciado => "subagent.started",
            Momento::SubagenteConcluido => "subagent.finished",
            Momento::TarefaCriada => "task.created",
            Momento::TarefaConcluida => "task.completed",
            Momento::FerramentaIniciada => "tool.started",
            Momento::FerramentaConcluida => "tool.finished",
            Momento::FerramentaFalhou => "tool.failed",
            Momento::ContextoCompactando => "context.compacting",
            Momento::TextoDeMensagem => "message.text",
            Momento::Erro => "error",
        }
    }

    /// Urgência antes de qualquer configuração da pessoa.
    ///
    /// A tabela é a de `docs/specs/speech-output.md`. Mudança aqui muda o
    /// comportamento padrão de quem nunca configurou nada — mexa junto com o
    /// spec, não sozinho.
    pub fn urgencia_padrao(self) -> Urgencia {
        match self {
            Momento::DecisaoNecessaria | Momento::EntradaNecessaria => Urgencia::Critica,
            Momento::TurnoConcluido | Momento::TurnoFalhou | Momento::Erro => Urgencia::Alta,
            Momento::SubagenteIniciado
            | Momento::SubagenteConcluido
            | Momento::TarefaConcluida
            | Momento::FerramentaFalhou => Urgencia::Media,
            Momento::SessaoIniciada
            | Momento::SessaoEncerrada
            | Momento::TarefaCriada
            | Momento::ContextoCompactando => Urgencia::Baixa,
            Momento::FerramentaIniciada
            | Momento::FerramentaConcluida
            | Momento::TextoDeMensagem => Urgencia::Silenciosa,
        }
    }
}

impl std::fmt::Display for Momento {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.nome())
    }
}

/// Um acontecimento já normalizado, pronto para a política de voz.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Evento {
    pub momento: Momento,
    pub origem: Origem,
    pub transporte: Transporte,
    pub sessao_id: String,
    /// Diretório de trabalho do CLI, para agrupar e para escolher o perfil.
    pub projeto: Option<String>,
    /// O que interessa falar, já extraído. Pode ser vazio.
    pub texto: String,
    /// Payload cru da origem. Só para depuração — nenhum consumidor decide a
    /// partir daqui (ADR-0007).
    pub detalhe: serde_json::Value,
    /// Milissegundos desde a época Unix, carimbados na chegada.
    pub recebido_em_ms: u64,
}

impl Evento {
    pub fn novo(momento: Momento, origem: Origem, transporte: Transporte) -> Self {
        Evento {
            momento,
            origem,
            transporte,
            sessao_id: String::new(),
            projeto: None,
            texto: String::new(),
            detalhe: serde_json::Value::Null,
            recebido_em_ms: agora_ms(),
        }
    }

    /// Chave de deduplicação entre transportes.
    ///
    /// Dois eventos com a mesma chave dentro da janela de tolerância são o
    /// mesmo acontecimento chegando por caminhos diferentes; vence o de maior
    /// [`Transporte::confianca`].
    pub fn chave_dedup(&self) -> (Origem, &str, Momento) {
        (self.origem, self.sessao_id.as_str(), self.momento)
    }

    pub fn urgencia(&self) -> Urgencia {
        self.momento.urgencia_padrao()
    }
}

/// Milissegundos desde a época Unix.
///
/// Relógio de parede, não monotônico: o carimbo é para ordenar e expirar itens
/// da fila, não para medir duração.
pub fn agora_ms() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_millis() as u64)
        .unwrap_or(0)
}

#[cfg(test)]
mod testes {
    use super::*;

    #[test]
    fn nome_do_momento_sobrevive_a_ida_e_volta() {
        // O nome é contrato: é o que a pessoa escreve no config.toml.
        for m in [
            Momento::DecisaoNecessaria,
            Momento::TurnoConcluido,
            Momento::TextoDeMensagem,
            Momento::Erro,
        ] {
            let json = serde_json::to_string(&m).expect("serializa");
            assert_eq!(json, format!("\"{}\"", m.nome()));
            let volta: Momento = serde_json::from_str(&json).expect("desserializa");
            assert_eq!(volta, m);
        }
    }

    #[test]
    fn hook_vence_pty_na_deduplicacao() {
        assert!(Transporte::Hook.confianca() > Transporte::Pty.confianca());
        assert!(Transporte::Acp.confianca() > Transporte::Notify.confianca());
    }

    #[test]
    fn decisao_e_mais_urgente_que_turno_concluido() {
        assert!(
            Momento::DecisaoNecessaria.urgencia_padrao()
                > Momento::TurnoConcluido.urgencia_padrao()
        );
        assert!(
            Momento::TurnoConcluido.urgencia_padrao()
                > Momento::FerramentaIniciada.urgencia_padrao()
        );
    }
}
