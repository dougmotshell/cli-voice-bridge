//! Reprodução de áudio e a voz de emergência do sistema.
//!
//! Tocar é invocar um programa do sistema, não usar biblioteca nativa
//! (ADR-0009): `cargo build` funciona numa máquina recém-clonada, sem
//! `libasound2-dev` nem pacote nenhum.

use std::path::Path;
use std::process::{Command, Stdio};

/// Candidatos por plataforma, em ordem de preferência.
#[cfg(target_os = "linux")]
const CANDIDATOS: &[(&str, &[&str])] = &[
    ("paplay", &[]),
    ("pw-play", &[]),
    ("aplay", &["-q"]),
    ("ffplay", &["-nodisp", "-autoexit", "-loglevel", "quiet"]),
];

#[cfg(target_os = "macos")]
const CANDIDATOS: &[(&str, &[&str])] = &[("afplay", &[])];

#[cfg(windows)]
const CANDIDATOS: &[(&str, &[&str])] = &[("powershell", &["-NoProfile", "-Command"])];

#[cfg(not(any(target_os = "linux", target_os = "macos", windows)))]
const CANDIDATOS: &[(&str, &[&str])] = &[];

/// O comando que vai tocar, já resolvido.
#[derive(Debug, Clone)]
pub struct Reprodutor {
    programa: String,
    argumentos: Vec<String>,
}

impl Reprodutor {
    /// `configurado` vem de `geral.reprodutor`; vazio consulta a lista.
    pub fn descobrir(configurado: &[String]) -> Option<Reprodutor> {
        if let Some((programa, resto)) = configurado.split_first() {
            return Some(Reprodutor {
                programa: programa.clone(),
                argumentos: resto.to_vec(),
            });
        }
        CANDIDATOS
            .iter()
            .find(|(p, _)| existe_no_path(p))
            .map(|(p, args)| Reprodutor {
                programa: p.to_string(),
                argumentos: args.iter().map(|a| a.to_string()).collect(),
            })
    }

    pub fn nome(&self) -> &str {
        &self.programa
    }

    /// O que foi procurado, para o `cvb doctor` dizer em vez de errar genérico.
    pub fn candidatos() -> Vec<&'static str> {
        CANDIDATOS.iter().map(|(p, _)| *p).collect()
    }

    /// Toca e espera terminar. Bloqueia: a fila de voz é serial por construção.
    pub fn tocar(&self, arquivo: &Path) -> Result<(), String> {
        let mut cmd = Command::new(&self.programa);
        cmd.args(&self.argumentos);

        #[cfg(windows)]
        if self.programa.eq_ignore_ascii_case("powershell") {
            cmd.arg(format!(
                "(New-Object Media.SoundPlayer '{}').PlaySync()",
                arquivo.display()
            ));
        } else {
            cmd.arg(arquivo);
        }
        #[cfg(not(windows))]
        cmd.arg(arquivo);

        let estado = cmd
            .stdin(Stdio::null())
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .status()
            .map_err(|e| format!("{} não executou: {e}", self.programa))?;

        if estado.success() {
            Ok(())
        } else {
            Err(format!("{} saiu com {estado}", self.programa))
        }
    }
}

/// A voz de emergência do sistema.
///
/// Feia, e é o ponto: avisar com voz feia é melhor que não avisar que o agente
/// está travado esperando permissão (ADR-0003).
pub fn falar_com_voz_do_sistema(texto: &str) -> Result<(), String> {
    let tentativas: Vec<(&str, Vec<String>)> = if cfg!(target_os = "macos") {
        vec![("say", vec![texto.to_string()])]
    } else if cfg!(windows) {
        vec![(
            "powershell",
            vec![
                "-NoProfile".into(),
                "-Command".into(),
                format!(
                    "Add-Type -AssemblyName System.Speech; \
                     (New-Object System.Speech.Synthesis.SpeechSynthesizer).Speak('{}')",
                    texto.replace('\'', "''")
                ),
            ],
        )]
    } else {
        vec![
            (
                "espeak-ng",
                vec!["-v".into(), "pt-br".into(), texto.to_string()],
            ),
            ("espeak", vec!["-v".into(), "pt".into(), texto.to_string()]),
            ("spd-say", vec![texto.to_string()]),
        ]
    };

    let mut ultimo = String::from("nenhuma voz de sistema encontrada");
    for (programa, args) in tentativas {
        if !existe_no_path(programa) {
            ultimo = format!("{programa} não está no PATH");
            continue;
        }
        match Command::new(programa)
            .args(&args)
            .stdin(Stdio::null())
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .status()
        {
            Ok(e) if e.success() => return Ok(()),
            Ok(e) => ultimo = format!("{programa} saiu com {e}"),
            Err(e) => ultimo = format!("{programa} não executou: {e}"),
        }
    }
    Err(ultimo)
}

fn existe_no_path(programa: &str) -> bool {
    let Some(path) = std::env::var_os("PATH") else {
        return false;
    };
    std::env::split_paths(&path).any(|dir| {
        let alvo = dir.join(programa);
        alvo.is_file()
            || ["exe", "cmd", "bat"]
                .iter()
                .any(|ext| alvo.with_extension(ext).is_file())
    })
}

#[cfg(test)]
mod testes {
    use super::*;

    #[test]
    fn reprodutor_configurado_vence_a_lista() {
        let r = Reprodutor::descobrir(&["meu-player".into(), "--quiet".into()]).unwrap();
        assert_eq!(r.nome(), "meu-player");
    }

    #[test]
    fn a_lista_de_candidatos_nao_e_vazia_nas_plataformas_suportadas() {
        // Se um dia alguém compilar num alvo sem candidatos, o doctor precisa
        // ter o que dizer; aqui só garantimos que as três têm lista.
        if cfg!(any(target_os = "linux", target_os = "macos", windows)) {
            assert!(!Reprodutor::candidatos().is_empty());
        }
    }
}
