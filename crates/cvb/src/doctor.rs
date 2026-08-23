//! `cvb doctor` — o diagnóstico.
//!
//! É o primeiro passo de qualquer investigação, e por isso ele não pode
//! depender de nada estar funcionando. Cada verificação é independente: uma
//! falha não impede as outras de rodarem, porque o valor está em ver o quadro
//! inteiro de uma vez.
//!
//! Recurso indisponível na plataforma vira aviso explícito, nunca falha
//! silenciosa (`docs/specs/portability.md`).

use std::path::{Path, PathBuf};
use std::process::ExitCode;

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
    let mut checagens = vec![
        verificar_config(),
        verificar_voice_clone(),
        verificar_clis(),
        verificar_plataforma(),
    ];
    if !offline {
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

fn verificar_config() -> Verificacao {
    let arquivo = caminhos::arquivo_config();
    let veredito = if arquivo.exists() {
        Veredito::Ok(arquivo.display().to_string())
    } else {
        // Ausência não é falha: os padrões embutidos bastam para começar.
        Veredito::Aviso(format!(
            "ausente, usando os padrões ({})",
            arquivo.display()
        ))
    };
    Verificacao {
        titulo: "configuração",
        veredito,
    }
}

/// O `voice-clone` é dependência externa somente leitura (ADR-0003).
///
/// TODO: o caminho vem da configuração, que ainda não é lida. Por ora só o
/// ambiente, para nunca embutir um caminho no código.
fn verificar_voice_clone() -> Verificacao {
    let raiz = std::env::var_os("CVB_VOICE_CLONE").map(PathBuf::from);

    let veredito = match raiz {
        None => Veredito::Aviso(
            "caminho não configurado — defina CVB_VOICE_CLONE ou `voice_clone.raiz`".into(),
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
