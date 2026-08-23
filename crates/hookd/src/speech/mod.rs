//! Da frase ao som: redação, síntese, cache e reprodução.
//!
//! A cadeia é `redigir → sintetizar → tocar`, com um degrau de degradação em
//! cada ponto que pode falhar. O princípio que organiza tudo:
//!
//! > **Nunca ficar mudo.** Sidecar morto, voz inexistente, XTTS quebrado —
//! > qualquer um deles cai para a voz do sistema e avisa por quê. Falar feio é
//! > melhor que não avisar que o agente está travado esperando permissão
//! > (ADR-0003).
//!
//! Fila, prioridade e corte ainda não existem — ver os `TODO:` no fim.

pub mod redact;
pub mod template;

use std::hash::{Hash, Hasher};
use std::path::PathBuf;

use cvb_core::audio::{self as playback, Reprodutor};
use cvb_core::caminhos;
use cvb_core::config::Config;
use cvb_core::sidecar::Sidecar;

use redact::Redator;

/// Como a fala saiu — ou por que não saiu.
#[derive(Debug)]
pub enum Saida {
    /// Sintetizado pelo `voice-clone`, na voz clonada.
    VozClonada { do_cache: bool },
    /// O sidecar não serviu; saiu pela voz do sistema.
    VozDoSistema { motivo: String },
    /// Nem uma nem outra.
    Mudo { motivo: String },
}

impl Saida {
    pub fn falou(&self) -> bool {
        !matches!(self, Saida::Mudo { .. })
    }

    /// Uma linha para o log e para a resposta ao cliente.
    pub fn descricao(&self) -> String {
        match self {
            Saida::VozClonada { do_cache: true } => "voz clonada (cache)".into(),
            Saida::VozClonada { do_cache: false } => "voz clonada".into(),
            Saida::VozDoSistema { motivo } => format!("voz do sistema — {motivo}"),
            Saida::Mudo { motivo } => format!("mudo — {motivo}"),
        }
    }
}

pub struct Voz {
    /// Uma fala por vez. A fila com prioridade e corte ainda não existe (ver os
    /// `TODO:` no fim), mas duas threads falando juntas produziriam dois áudios
    /// sobrepostos — que é pior que esperar.
    falando: std::sync::Mutex<()>,
    redator: Redator,
    sidecar: Sidecar,
    reprodutor: Option<Reprodutor>,
    voz: String,
    idioma: String,
    dir_cache: PathBuf,
}

impl Voz {
    pub fn nova(config: &Config) -> Voz {
        Voz {
            falando: std::sync::Mutex::new(()),
            redator: Redator::novo(&config.privacidade.redigir),
            sidecar: Sidecar::novo(),
            reprodutor: Reprodutor::descobrir(&config.geral.reprodutor),
            voz: config.geral.voz.clone(),
            idioma: config.geral.idioma.clone(),
            dir_cache: caminhos::dir_dados().join("cache-audio"),
        }
    }

    pub fn reprodutor(&self) -> Option<&Reprodutor> {
        self.reprodutor.as_ref()
    }

    pub fn sidecar(&self) -> &Sidecar {
        &self.sidecar
    }

    /// Fala um texto. **Redige antes de qualquer outra coisa.**
    ///
    /// Bloqueia até o áudio terminar, e serializa com as outras chamadas.
    pub fn falar(&self, bruto: &str) -> Saida {
        let texto = self.redator.redigir(bruto);
        if texto.trim().is_empty() {
            return Saida::Mudo {
                motivo: "nada a dizer depois da redação".into(),
            };
        }

        let _fila = self.falando.lock().unwrap_or_else(|e| e.into_inner());

        match self.tentar_voz_clonada(&texto) {
            Ok(saida) => saida,
            Err(motivo) => match playback::falar_com_voz_do_sistema(&texto) {
                Ok(()) => Saida::VozDoSistema { motivo },
                Err(e) => Saida::Mudo {
                    motivo: format!("{motivo}; e a voz do sistema também falhou: {e}"),
                },
            },
        }
    }

    fn tentar_voz_clonada(&self, texto: &str) -> Result<Saida, String> {
        if self.voz.is_empty() {
            return Err("nenhuma voz configurada em `geral.voz`".into());
        }
        let Some(reprodutor) = &self.reprodutor else {
            return Err(format!(
                "nenhum reprodutor de áudio encontrado (procurei por {})",
                Reprodutor::candidatos().join(", ")
            ));
        };

        // Frases fixas — "terminei", "preciso de permissão" — são poucas e se
        // repetem. Sintetizar uma vez troca a maior parte das falas por
        // reprodução instantânea.
        let arquivo = self.caminho_no_cache(texto);
        let do_cache = arquivo.is_file();

        if !do_cache {
            if let Some(pai) = arquivo.parent() {
                std::fs::create_dir_all(pai)
                    .map_err(|e| format!("não criei o cache de áudio: {e}"))?;
            }
            self.sidecar
                .sintetizar(texto, &self.voz, &self.idioma, &arquivo)
                .map_err(|e| e.to_string())?;
            if !arquivo.is_file() {
                return Err("o sidecar disse ok mas não deixou arquivo".into());
            }
        }

        reprodutor.tocar(&arquivo)?;
        Ok(Saida::VozClonada { do_cache })
    }

    /// Chave de cache: (voz, idioma, texto). Hash não criptográfico de
    /// propósito — isto é um nome de arquivo, não uma garantia de integridade.
    fn caminho_no_cache(&self, texto: &str) -> PathBuf {
        let mut h = std::collections::hash_map::DefaultHasher::new();
        self.voz.hash(&mut h);
        self.idioma.hash(&mut h);
        texto.hash(&mut h);
        self.dir_cache.join(format!("{:016x}.wav", h.finish()))
    }
}

// TODO: falta a fila de `docs/pt-BR/specs/speech-output.md` — prioridade por
// urgência, colapso de momentos repetidos, expiração do que envelheceu e corte
// da fala em curso. Hoje `falar` é síncrono e serial: quem chamar duas vezes
// espera duas vezes.
//
// TODO: o corte, quando existir, mata o processo do reprodutor — é o que a
// escolha do ADR-0009 permite. Se soar ruim, é sinal de revisitar aquele ADR.

#[cfg(test)]
mod testes {
    use super::*;

    #[test]
    fn a_mesma_frase_cai_no_mesmo_arquivo_de_cache() {
        let voz = Voz::nova(&Config::default());
        assert_eq!(voz.caminho_no_cache("oi"), voz.caminho_no_cache("oi"));
        assert_ne!(voz.caminho_no_cache("oi"), voz.caminho_no_cache("tchau"));
    }

    #[test]
    fn sem_voz_configurada_nao_tenta_a_clonada() {
        let voz = Voz::nova(&Config::default());
        let erro = voz.tentar_voz_clonada("teste").unwrap_err();
        assert!(erro.contains("geral.voz"), "{erro}");
    }

    #[test]
    fn texto_que_some_na_redacao_nao_vira_fala() {
        let mut c = Config::default();
        c.geral.voz = "alguem".into();
        let voz = Voz::nova(&c);
        match voz.falar("   ") {
            Saida::Mudo { motivo } => assert!(motivo.contains("redação")),
            outro => panic!("esperava mudo, veio {outro:?}"),
        }
    }
}
