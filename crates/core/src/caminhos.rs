//! Caminhos de configuração, socket, cache e log — nos três sistemas.
//!
//! Nenhum outro módulo escreve caminho de plataforma à mão. Um literal
//! `~/.config` espalhado pelo código é defeito, e é o tipo de defeito que só
//! aparece na máquina de outra pessoa (`docs/pt-BR/specs/portability.md`).

use std::path::PathBuf;

pub const NOME_APP: &str = "cli-voice-bridge";

fn var(nome: &str) -> Option<PathBuf> {
    std::env::var_os(nome)
        .filter(|v| !v.is_empty())
        .map(PathBuf::from)
}

fn casa() -> PathBuf {
    var("HOME")
        .or_else(|| var("USERPROFILE"))
        .unwrap_or_else(|| PathBuf::from("."))
}

/// Diretório de configuração da pessoa.
///
/// - Linux: `$XDG_CONFIG_HOME/cli-voice-bridge` ou `~/.config/cli-voice-bridge`
/// - macOS: `~/Library/Application Support/cli-voice-bridge`
/// - Windows: `%APPDATA%\cli-voice-bridge`
pub fn dir_config() -> PathBuf {
    #[cfg(target_os = "macos")]
    {
        casa().join("Library/Application Support").join(NOME_APP)
    }
    #[cfg(target_os = "windows")]
    {
        var("APPDATA")
            .unwrap_or_else(|| casa().join("AppData/Roaming"))
            .join(NOME_APP)
    }
    #[cfg(all(unix, not(target_os = "macos")))]
    {
        var("XDG_CONFIG_HOME")
            .unwrap_or_else(|| casa().join(".config"))
            .join(NOME_APP)
    }
}

/// O arquivo de configuração da pessoa.
pub fn arquivo_config() -> PathBuf {
    dir_config().join("config.toml")
}

/// Diretório de dados: cache de áudio, modelos de STT, log.
pub fn dir_dados() -> PathBuf {
    #[cfg(target_os = "macos")]
    {
        casa().join("Library/Application Support").join(NOME_APP)
    }
    #[cfg(target_os = "windows")]
    {
        var("LOCALAPPDATA")
            .unwrap_or_else(|| casa().join("AppData/Local"))
            .join(NOME_APP)
    }
    #[cfg(all(unix, not(target_os = "macos")))]
    {
        var("XDG_DATA_HOME")
            .unwrap_or_else(|| casa().join(".local/share"))
            .join(NOME_APP)
    }
}

/// Endereço do socket do daemon.
///
/// No Windows é o nome de um named pipe, não um caminho de arquivo — por isso o
/// tipo é o mesmo, mas o significado não. Nunca é uma porta TCP (ADR-0008).
pub fn endereco_daemon() -> PathBuf {
    if let Some(explicito) = var("CVB_SOCKET") {
        return explicito;
    }
    #[cfg(windows)]
    {
        PathBuf::from(format!(r"\\.\pipe\{NOME_APP}"))
    }
    #[cfg(target_os = "macos")]
    {
        var("TMPDIR")
            .unwrap_or_else(|| PathBuf::from("/tmp"))
            .join(format!("{NOME_APP}.sock"))
    }
    #[cfg(all(unix, not(target_os = "macos")))]
    {
        var("XDG_RUNTIME_DIR")
            .unwrap_or_else(|| PathBuf::from("/tmp"))
            .join(format!("{NOME_APP}.sock"))
    }
}

/// Endereço do socket do sidecar de síntese.
pub fn endereco_sidecar() -> PathBuf {
    if let Some(explicito) = var("CVB_SIDECAR_SOCKET") {
        return explicito;
    }
    let base = endereco_daemon();
    let pai = base.parent().unwrap_or_else(|| std::path::Path::new("."));
    #[cfg(windows)]
    {
        let _ = pai;
        PathBuf::from(format!(r"\\.\pipe\{NOME_APP}-sidecar"))
    }
    #[cfg(unix)]
    {
        pai.join(format!("{NOME_APP}-sidecar.sock"))
    }
}

#[cfg(test)]
mod testes {
    use super::*;

    #[test]
    fn config_fica_dentro_do_diretorio_do_app() {
        let arquivo = arquivo_config();
        assert!(arquivo.ends_with("config.toml"));
        assert!(arquivo.to_string_lossy().contains(NOME_APP));
    }

    #[test]
    fn endereco_do_daemon_nunca_e_porta_tcp() {
        // Guarda de regressão do ADR-0008: nada de host:porta aqui.
        let e = endereco_daemon().to_string_lossy().to_string();
        assert!(!e.contains("127.0.0.1"));
        assert!(!e.contains("localhost"));
    }
}
