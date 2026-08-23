//! `cvb doctor` — o diagnóstico.
//!
//! É o primeiro passo de qualquer investigação, e por isso ele não pode
//! depender de nada estar funcionando. Cada verificação é independente: uma
//! falha não impede as outras de rodarem, porque o valor está em ver o quadro
//! inteiro de uma vez.
//!
//! Recurso indisponível na plataforma vira aviso explícito, nunca falha
//! silenciosa (`docs/pt-BR/specs/portability.md`).

use std::path::{Path, PathBuf};
use std::process::ExitCode;

use cvb_core::audio::Reprodutor;
use cvb_core::config::Config;
use cvb_core::sidecar::Sidecar;
use cvb_core::{caminhos, ipc};

enum Veredito {
    Ok(String),
    Aviso(String),
    Falha(String),
}

struct Verificacao {
    titulo: &'static str,
    veredito: Veredito,
}

pub fn executar(offline: bool) -> ExitCode {
    // Cada verificação é independente: uma falha não impede as outras de
    // rodarem, porque o valor está em ver o quadro inteiro de uma vez.
    let config = Config::carregar();

    let mut checagens = vec![
        verificar_config(&config),
        verificar_voice_clone(config.as_ref().ok()),
        verificar_voz(config.as_ref().ok()),
        verificar_clis(),
        verificar_reprodutor(config.as_ref().ok()),
        verificar_plataforma(),
    ];
    if !offline {
        checagens.push(verificar_sidecar());
        checagens.push(verificar_daemon());
    }

    let mut falhas = 0;
    let mut avisos = 0;

    for c in &checagens {
        let (marca, texto) = match &c.veredito {
            Veredito::Ok(t) => ("ok   ", t),
            Veredito::Aviso(t) => {
                avisos += 1;
                ("aviso", t)
            }
            Veredito::Falha(t) => {
                falhas += 1;
                ("FALHA", t)
            }
        };
        println!("{marca}  {:<22} {texto}", c.titulo);
    }

    println!();
    if falhas > 0 {
        println!("{falhas} falha(s), {avisos} aviso(s).");
        ExitCode::FAILURE
    } else if avisos > 0 {
        println!("Sem falhas, {avisos} aviso(s).");
        ExitCode::SUCCESS
    } else {
        println!("Tudo em ordem.");
        ExitCode::SUCCESS
    }
}

fn verificar_config(config: &Result<Config, cvb_core::config::ErroConfig>) -> Verificacao {
    let arquivo = caminhos::arquivo_config();
    let veredito = match config {
        // Arquivo inválido é falha: seguir com os padrões faria a pessoa achar
        // que o que ela escreveu está valendo.
        Err(e) => Veredito::Falha(e.to_string()),
        Ok(_) if arquivo.exists() => Veredito::Ok(arquivo.display().to_string()),
        // Ausência não é falha: os padrões embutidos bastam para começar.
        Ok(_) => Veredito::Aviso(format!(
            "ausente, usando os padrões ({})",
            arquivo.display()
        )),
    };
    Verificacao {
        titulo: "configuração",
        veredito,
    }
}

fn verificar_voz(config: Option<&Config>) -> Verificacao {
    let veredito = match config.map(|c| c.geral.voz.clone()) {
        Some(v) if !v.is_empty() => Veredito::Ok(v),
        _ => Veredito::Aviso(
            "`geral.voz` vazio — vou falar com a voz do sistema; veja `cvb voices`".into(),
        ),
    };
    Verificacao {
        titulo: "voz",
        veredito,
    }
}

fn verificar_reprodutor(config: Option<&Config>) -> Verificacao {
    let configurado = config
        .map(|c| c.geral.reprodutor.clone())
        .unwrap_or_default();
    let veredito = match Reprodutor::descobrir(&configurado) {
        Some(r) => Veredito::Ok(r.nome().to_string()),
        None => Veredito::Falha(format!(
            "nenhum encontrado — procurei por {}; fixe um em `geral.reprodutor`",
            Reprodutor::candidatos().join(", ")
        )),
    };
    Verificacao {
        titulo: "reprodutor de áudio",
        veredito,
    }
}

fn verificar_sidecar() -> Verificacao {
    let sidecar = Sidecar::novo();
    let veredito = if sidecar.vivo() {
        Veredito::Ok(sidecar.endereco().display().to_string())
    } else {
        // Não é falha: existe o caminho da voz do sistema, e ele é o combinado
        // (ADR-0003). Mas a pessoa precisa saber por que a voz está feia.
        Veredito::Aviso(format!(
            "fora do ar em {} — a fala sai pela voz do sistema",
            sidecar.endereco().display()
        ))
    };
    Verificacao {
        titulo: "sidecar de síntese",
        veredito,
    }
}

fn verificar_voice_clone(config: Option<&Config>) -> Verificacao {
    let raiz = config
        .and_then(|c| c.voice_clone.raiz_resolvida())
        .or_else(|| std::env::var_os("CVB_VOICE_CLONE").map(PathBuf::from));

    let veredito = match raiz {
        None => Veredito::Aviso(
            "caminho não configurado — defina `voice_clone.raiz` ou CVB_VOICE_CLONE".into(),
        ),
        Some(r) if !r.join("falar.py").is_file() => {
            Veredito::Falha(format!("não achei falar.py em {}", r.display()))
        }
        Some(r) => Veredito::Ok(r.display().to_string()),
    };

    Verificacao {
        titulo: "voice-clone",
        veredito,
    }
}

fn verificar_clis() -> Verificacao {
    let achados: Vec<&str> = ["claude", "codex", "copilot"]
        .into_iter()
        .filter(|n| esta_no_path(n))
        .collect();

    let veredito = if achados.is_empty() {
        Veredito::Falha("nenhum CLI de IA encontrado no PATH".into())
    } else {
        Veredito::Ok(achados.join(", "))
    };

    Verificacao {
        titulo: "CLIs de IA",
        veredito,
    }
}

fn verificar_daemon() -> Verificacao {
    let endereco = caminhos::endereco_daemon();
    let veredito = match ipc::conectar(&endereco) {
        Ok(_) => Veredito::Ok(endereco.display().to_string()),
        Err(e) => Veredito::Falha(format!("fora do ar ({e})")),
    };
    Verificacao {
        titulo: "daemon",
        veredito,
    }
}

/// O que a plataforma desta máquina não suporta.
///
/// Dizer isto em voz alta é o ponto: no Wayland sem portal o atalho global
/// simplesmente não funciona, e descobrir isso por tentativa e erro custa uma
/// tarde.
fn verificar_plataforma() -> Verificacao {
    #[cfg(windows)]
    let veredito =
        Veredito::Falha("IPC por named pipe ainda não implementado — ver ADR-0008".into());

    #[cfg(target_os = "macos")]
    let veredito = Veredito::Aviso(
        "macOS: atalho global e injeção de teclado exigem permissão de Acessibilidade".into(),
    );

    #[cfg(all(unix, not(target_os = "macos")))]
    let veredito = if std::env::var_os("WAYLAND_DISPLAY").is_some() {
        Veredito::Aviso(
            "Wayland: atalho global depende do portal do sistema; sem ele, use o ditado por área de transferência"
                .into(),
        )
    } else {
        Veredito::Ok("X11".into())
    };

    Verificacao {
        titulo: "plataforma",
        veredito,
    }
}

fn esta_no_path(programa: &str) -> bool {
    let Some(path) = std::env::var_os("PATH") else {
        return false;
    };
    std::env::split_paths(&path).any(|dir| executavel(&dir.join(programa)))
}

fn executavel(caminho: &Path) -> bool {
    if caminho.is_file() {
        return true;
    }
    // No Windows o binário tem extensão; no PATH ela não vem no nome.
    ["exe", "cmd", "bat"]
        .iter()
        .any(|ext| caminho.with_extension(ext).is_file())
}
