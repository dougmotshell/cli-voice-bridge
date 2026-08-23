//! Configuração: camadas, precedência e recarga.
//!
//! O contrato completo está em `docs/pt-BR/specs/configuration.md`. O que este módulo
//! garante é a **precedência**, que é a única parte já fechada:
//!
//! 1. padrões embutidos → 2. arquivo da pessoa → 3. arquivo do projeto →
//! 4. perfil ativo → 5. variáveis `CVB_*` → 6. argumentos de linha de comando
//!
//! **Configuração de projeto não manda no seu microfone.** Um repositório
//! clonado não pode ligar o que a pessoa desligou por segurança — ver
//! [`Config::mesclar_projeto`].

use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};

use crate::caminhos;
use crate::momento::Momento;

/// Nome do arquivo de configuração dentro de um projeto.
pub const ARQUIVO_DE_PROJETO: &str = ".cli-voice-bridge.toml";

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(default, deny_unknown_fields)]
pub struct Config {
    pub geral: Geral,
    pub voice_clone: VoiceClone,
    /// Chaveado por nome de CLI: `claude`, `codex`, `copilot`.
    pub cli: BTreeMap<String, ConfigCli>,
    /// Chaveado pelo nome estável do momento: `decision.needed` etc.
    pub momentos: BTreeMap<String, ConfigMomento>,
    pub escuta: Escuta,
    pub privacidade: Privacidade,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default, deny_unknown_fields)]
pub struct Geral {
    /// Nome de voz cadastrado no `voice-clone`.
    pub voz: String,
    pub idioma: String,
    pub perfil: String,
    /// Teto de duração de uma fala. Acima disso, corta e diz que tem mais.
    pub segundos_max_por_fala: u64,
    /// Comando que reproduz um WAV. Vazio = descobrir pela plataforma.
    pub reprodutor: Vec<String>,
}

impl Default for Geral {
    fn default() -> Self {
        Geral {
            voz: String::new(),
            idioma: "pt-BR".into(),
            perfil: "padrao".into(),
            segundos_max_por_fala: 12,
            reprodutor: Vec::new(),
        }
    }
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(default, deny_unknown_fields)]
pub struct VoiceClone {
    /// Raiz do projeto `voice-clone`. Nunca embutida no código (ADR-0003).
    pub raiz: String,
    /// Interpretador. Vazio = `<raiz>/.venv/bin/python` (ou `Scripts` no Windows).
    pub python: String,
}

impl VoiceClone {
    pub fn raiz_resolvida(&self) -> Option<PathBuf> {
        let bruto = if self.raiz.is_empty() {
            std::env::var("CVB_VOICE_CLONE").ok()?
        } else {
            self.raiz.clone()
        };
        Some(expandir_til(&bruto))
    }

    pub fn python_resolvido(&self) -> Option<PathBuf> {
        if !self.python.is_empty() {
            return Some(expandir_til(&self.python));
        }
        let raiz = self.raiz_resolvida()?;
        let candidato = if cfg!(windows) {
            raiz.join(".venv/Scripts/python.exe")
        } else {
            raiz.join(".venv/bin/python")
        };
        Some(candidato)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default, deny_unknown_fields)]
pub struct ConfigCli {
    pub ativo: bool,
    pub transportes: Vec<String>,
    /// `voz`, `texto` ou `ambos`.
    pub resposta: String,
}

impl Default for ConfigCli {
    fn default() -> Self {
        ConfigCli {
            ativo: true,
            transportes: vec!["hook".into()],
            resposta: "texto".into(),
        }
    }
}

/// Quando um momento vira fala.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum QuandoFalar {
    /// Sempre que acontecer.
    Sempre,
    /// Só quando a pessoa não está olhando. É o que mais reduz ruído.
    Ausente,
    Nunca,
}

/// Tudo opcional de propósito: ausente significa "use o padrão do momento",
/// que é diferente de "não fale".
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(default, deny_unknown_fields)]
pub struct ConfigMomento {
    pub falar: Option<QuandoFalar>,
    /// Corta a fala em curso.
    pub interrompe: Option<bool>,
    /// Molde da frase. `{cli}`, `{texto}` e `{ferramenta}` são substituídos.
    pub molde: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default, deny_unknown_fields)]
pub struct Escuta {
    pub acionamento: String,
    pub atalho: String,
    pub motor: String,
    /// Comando destrutivo nunca é autorizado por um único "sim" falado.
    pub confirmar_destrutivo: bool,
}

impl Default for Escuta {
    fn default() -> Self {
        Escuta {
            acionamento: "push-to-talk".into(),
            atalho: "Ctrl+Alt+Space".into(),
            motor: String::new(),
            confirmar_destrutivo: true,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default, deny_unknown_fields)]
pub struct Privacidade {
    /// Fala a categoria, nunca o conteúdo.
    pub modo_discreto: bool,
    /// Marcadores extras de segredo, além dos embutidos.
    pub redigir: Vec<String>,
    pub retencao_log_dias: u64,
}

impl Default for Privacidade {
    fn default() -> Self {
        Privacidade {
            modo_discreto: false,
            redigir: Vec::new(),
            retencao_log_dias: 7,
        }
    }
}

#[derive(Debug)]
pub enum ErroConfig {
    Leitura { caminho: PathBuf, erro: String },
    Sintaxe { caminho: PathBuf, erro: String },
}

impl std::fmt::Display for ErroConfig {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ErroConfig::Leitura { caminho, erro } => {
                write!(f, "não consegui ler {}: {erro}", caminho.display())
            }
            ErroConfig::Sintaxe { caminho, erro } => {
                write!(f, "{}: {erro}", caminho.display())
            }
        }
    }
}

impl std::error::Error for ErroConfig {}

impl Config {
    /// Lê a configuração da pessoa, ou devolve os padrões se ela não existir.
    ///
    /// Arquivo ausente **não é erro**: os padrões embutidos bastam para começar.
    /// Arquivo presente e inválido **é** erro — silenciar isso faria a pessoa
    /// achar que configurou algo que não vale.
    pub fn carregar() -> Result<Config, ErroConfig> {
        Config::carregar_de(&caminhos::arquivo_config())
    }

    pub fn carregar_de(caminho: &Path) -> Result<Config, ErroConfig> {
        if !caminho.exists() {
            return Ok(Config::default());
        }
        let texto = std::fs::read_to_string(caminho).map_err(|e| ErroConfig::Leitura {
            caminho: caminho.to_path_buf(),
            erro: e.to_string(),
        })?;
        toml::from_str(&texto).map_err(|e| ErroConfig::Sintaxe {
            caminho: caminho.to_path_buf(),
            erro: e.to_string(),
        })
    }

    /// Mescla a configuração de um projeto por cima desta.
    ///
    /// **O que um projeto não pode fazer:** ligar o microfone, mudar o
    /// acionamento da escuta ou desligar a confirmação de comando destrutivo.
    /// Repositório clonado não manda na segurança de quem clonou
    /// (`docs/pt-BR/specs/configuration.md`).
    pub fn mesclar_projeto(&mut self, projeto: Config) {
        let escuta_preservada = self.escuta.clone();

        if !projeto.geral.voz.is_empty() {
            self.geral.voz = projeto.geral.voz;
        }
        if projeto.geral.perfil != Geral::default().perfil {
            self.geral.perfil = projeto.geral.perfil;
        }
        for (nome, c) in projeto.cli {
            self.cli.insert(nome, c);
        }
        for (nome, m) in projeto.momentos {
            self.momentos.insert(nome, m);
        }
        self.privacidade.redigir.extend(projeto.privacidade.redigir);
        // Modo discreto só pode ser LIGADO por um projeto, nunca desligado.
        self.privacidade.modo_discreto |= projeto.privacidade.modo_discreto;

        self.escuta = escuta_preservada;
    }

    /// A política para um momento, já com o padrão embutido aplicado.
    pub fn quando_falar(&self, momento: Momento) -> QuandoFalar {
        self.momentos
            .get(momento.nome())
            .and_then(|m| m.falar)
            .unwrap_or_else(|| match momento.urgencia_padrao() {
                crate::momento::Urgencia::Critica | crate::momento::Urgencia::Alta => {
                    QuandoFalar::Sempre
                }
                crate::momento::Urgencia::Media => QuandoFalar::Ausente,
                _ => QuandoFalar::Nunca,
            })
    }

    pub fn cli_ativo(&self, origem: crate::momento::Origem) -> bool {
        let nome = match origem {
            crate::momento::Origem::Claude => "claude",
            crate::momento::Origem::Codex => "codex",
            crate::momento::Origem::Copilot => "copilot",
        };
        self.cli.get(nome).map(|c| c.ativo).unwrap_or(true)
    }
}

/// Expande `~` no início de um caminho. Só no início — `~` no meio é literal.
pub fn expandir_til(bruto: &str) -> PathBuf {
    let Some(resto) = bruto.strip_prefix('~') else {
        return PathBuf::from(bruto);
    };
    let resto = resto.strip_prefix(['/', '\\']).unwrap_or(resto);
    let casa = std::env::var_os("HOME")
        .or_else(|| std::env::var_os("USERPROFILE"))
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from("."));
    if resto.is_empty() {
        casa
    } else {
        casa.join(resto)
    }
}

#[cfg(test)]
mod testes {
    use super::*;

    #[test]
    fn arquivo_ausente_devolve_padroes_sem_erro() {
        let c = Config::carregar_de(Path::new("/nao/existe/config.toml")).expect("padrões");
        assert_eq!(c.geral.idioma, "pt-BR");
        assert!(c.geral.voz.is_empty());
    }

    #[test]
    fn chave_desconhecida_e_erro_de_sintaxe() {
        // `deny_unknown_fields` é deliberado: chave escrita errada que passa em
        // silêncio faz a pessoa achar que configurou o que não configurou.
        let dir = std::env::temp_dir().join("cvb-teste-config-desconhecida");
        std::fs::create_dir_all(&dir).unwrap();
        let arquivo = dir.join("config.toml");
        std::fs::write(&arquivo, "[geral]\nvozz = \"x\"\n").unwrap();
        assert!(Config::carregar_de(&arquivo).is_err());
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn projeto_nao_mexe_na_escuta_nem_desliga_modo_discreto() {
        let mut pessoa = Config::default();
        pessoa.escuta.confirmar_destrutivo = true;
        pessoa.escuta.atalho = "Ctrl+Alt+Space".into();
        pessoa.privacidade.modo_discreto = true;

        let mut projeto = Config::default();
        projeto.escuta.confirmar_destrutivo = false;
        projeto.escuta.atalho = "F1".into();
        projeto.privacidade.modo_discreto = false;

        pessoa.mesclar_projeto(projeto);

        assert!(
            pessoa.escuta.confirmar_destrutivo,
            "projeto não desliga a confirmação"
        );
        assert_eq!(
            pessoa.escuta.atalho, "Ctrl+Alt+Space",
            "projeto não muda o atalho"
        );
        assert!(
            pessoa.privacidade.modo_discreto,
            "projeto não desliga o modo discreto"
        );
    }

    #[test]
    fn politica_padrao_segue_a_urgencia() {
        let c = Config::default();
        assert_eq!(
            c.quando_falar(Momento::DecisaoNecessaria),
            QuandoFalar::Sempre
        );
        assert_eq!(
            c.quando_falar(Momento::SubagenteConcluido),
            QuandoFalar::Ausente
        );
        assert_eq!(
            c.quando_falar(Momento::FerramentaIniciada),
            QuandoFalar::Nunca
        );
    }

    #[test]
    fn til_so_expande_no_comeco() {
        std::env::set_var("HOME", "/casa/alguem");
        assert_eq!(expandir_til("~/www/x"), PathBuf::from("/casa/alguem/www/x"));
        assert_eq!(expandir_til("/abs/~/x"), PathBuf::from("/abs/~/x"));
    }
}
