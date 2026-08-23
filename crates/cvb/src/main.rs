//! `cvb` — a interface de linha de comando.
//!
//! É um cliente do daemon como qualquer outro: nenhuma lógica de política, fila
//! ou síntese mora aqui. Recurso novo entra no daemon e é exposto nas duas
//! interfaces — é o que torna a paridade com a GUI uma consequência em vez de
//! uma promessa (`docs/pt-BR/specs/interfaces.md`).
//!
//! Códigos de saída: `0` sucesso, `1` falha de execução, `2` configuração
//! inválida, `3` daemon fora do ar.

mod doctor;

use std::process::ExitCode;

use clap::{Parser, Subcommand};
use cvb_core::ipc;
use cvb_core::{caminhos, Requisicao, Resposta, VERSAO_PROTOCOLO};

const SAIDA_FALHA: u8 = 1;
const SAIDA_CONFIG_INVALIDA: u8 = 2;
const SAIDA_SEM_DAEMON: u8 = 3;

#[derive(Parser)]
#[command(
    name = "cvb",
    about = "Dá voz aos CLIs de IA e aceita a sua resposta falada",
    version
)]
struct Cli {
    #[command(subcommand)]
    comando: Comando,
}

#[derive(Subcommand)]
enum Comando {
    /// Diagnóstico completo. Primeiro passo de qualquer investigação
    Doctor {
        /// Não tenta falar com o daemon nem com o áudio
        #[arg(long)]
        offline: bool,
    },
    /// Instala ou atualiza os hooks nos CLIs, compondo com os que já existem
    Install {
        /// Quais CLIs, separados por vírgula (claude, codex, copilot)
        #[arg(long, value_delimiter = ',')]
        cli: Vec<String>,
        /// Mostra o diff sem escrever
        #[arg(long)]
        dry_run: bool,
    },
    /// Remove só os hooks que o cvb instalou
    Uninstall {
        #[arg(long, value_delimiter = ',')]
        cli: Vec<String>,
    },
    /// Ciclo de vida do daemon
    Daemon {
        #[command(subcommand)]
        acao: AcaoDaemon,
    },
    /// Testa a saída de voz fim a fim
    Say { texto: String },
    /// Testa a entrada de voz e mostra a transcrição
    Listen,
    /// Abre um CLI dentro do wrapper de pseudo-terminal
    Wrap {
        #[arg(trailing_var_arg = true, allow_hyphen_values = true)]
        comando: Vec<String>,
    },
    /// Modo cliente de protocolo (ACP / app-server)
    Console {
        #[arg(long)]
        cli: String,
    },
    /// Configuração
    Config {
        #[command(subcommand)]
        acao: AcaoConfig,
    },
    /// Perfis
    Profile {
        #[command(subcommand)]
        acao: AcaoProfile,
    },
    /// Fluxo de momentos
    Events {
        #[arg(long)]
        follow: bool,
        #[arg(long)]
        json: bool,
    },
    /// Silêncio temporário. Sem duração, cala até `unmute`
    Mute {
        /// Em segundos
        segundos: Option<u64>,
    },
    /// Volta a falar
    Unmute,
}

#[derive(Subcommand)]
enum AcaoDaemon {
    Start,
    Stop,
    Status,
    Logs,
}

#[derive(Subcommand)]
enum AcaoConfig {
    Show,
    Edit,
    Check,
}

#[derive(Subcommand)]
enum AcaoProfile {
    List,
    Use { nome: String },
}

fn main() -> ExitCode {
    let cli = Cli::parse();

    match cli.comando {
        Comando::Doctor { offline } => doctor::executar(offline),

        Comando::Daemon { acao } => match acao {
            AcaoDaemon::Status => status(),
            AcaoDaemon::Start => nao_implementado("cvb daemon start"),
            AcaoDaemon::Stop => nao_implementado("cvb daemon stop"),
            AcaoDaemon::Logs => nao_implementado("cvb daemon logs"),
        },

        Comando::Mute { segundos } => pedir(Requisicao::Silenciar { segundos }),
        Comando::Unmute => pedir(Requisicao::Retomar),
        Comando::Say { texto } => pedir(Requisicao::Falar { texto }),

        // Estes ainda não existem. Repetir o que foi pedido não é enfeite: é o
        // que deixa claro que o argumento foi entendido e não engolido.
        Comando::Install { cli, dry_run } => nao_implementado(&format!(
            "cvb install (cli={}, dry-run={dry_run})",
            lista(&cli)
        )),
        Comando::Uninstall { cli } => {
            nao_implementado(&format!("cvb uninstall (cli={})", lista(&cli)))
        }
        Comando::Listen => nao_implementado("cvb listen"),
        Comando::Wrap { comando } => {
            nao_implementado(&format!("cvb wrap -- {}", comando.join(" ")))
        }
        Comando::Console { cli } => nao_implementado(&format!("cvb console --cli {cli}")),
        Comando::Events { follow, json } => {
            nao_implementado(&format!("cvb events (follow={follow}, json={json})"))
        }
        Comando::Config { acao } => nao_implementado(&format!(
            "cvb config {}",
            match acao {
                AcaoConfig::Show => "show",
                AcaoConfig::Edit => "edit",
                AcaoConfig::Check => "check",
            }
        )),
        Comando::Profile { acao } => nao_implementado(&match acao {
            AcaoProfile::List => "cvb profile list".to_string(),
            AcaoProfile::Use { nome } => format!("cvb profile use {nome}"),
        }),
    }
}

fn lista(itens: &[String]) -> String {
    if itens.is_empty() {
        "todos".to_string()
    } else {
        itens.join(",")
    }
}

/// Abre conexão e faz o handshake.
fn conectar() -> Result<ipc::Conexao, ExitCode> {
    let endereco = caminhos::endereco_daemon();
    let mut conexao = match ipc::conectar(&endereco) {
        Ok(c) => c,
        Err(e) => {
            eprintln!("cvb: daemon fora do ar em {} ({e})", endereco.display());
            eprintln!("     suba com: cvb daemon start");
            return Err(ExitCode::from(SAIDA_SEM_DAEMON));
        }
    };
    let ola = Requisicao::Ola {
        versao: VERSAO_PROTOCOLO,
        cliente: "cvb".into(),
    };
    if let Err(e) = ipc::enviar_linha(&mut conexao, &ola) {
        eprintln!("cvb: handshake falhou: {e}");
        return Err(ExitCode::from(SAIDA_FALHA));
    }
    Ok(conexao)
}

fn pedir(req: Requisicao) -> ExitCode {
    let mut conexao = match conectar() {
        Ok(c) => c,
        Err(codigo) => return codigo,
    };
    if let Err(e) = ipc::enviar_linha(&mut conexao, &req) {
        eprintln!("cvb: envio falhou: {e}");
        return ExitCode::from(SAIDA_FALHA);
    }
    match ler_resposta(conexao) {
        Some(Resposta::Ok) => ExitCode::SUCCESS,
        Some(Resposta::Erro { mensagem }) => {
            eprintln!("cvb: {mensagem}");
            ExitCode::from(SAIDA_FALHA)
        }
        Some(Resposta::VersaoIncompativel { esperada, recebida }) => {
            eprintln!("cvb: protocolo incompatível — daemon fala {esperada}, eu falo {recebida}");
            eprintln!("     atualize os dois lados para a mesma versão");
            ExitCode::from(SAIDA_CONFIG_INVALIDA)
        }
        Some(outra) => {
            println!("{}", serde_json::to_string(&outra).unwrap_or_default());
            ExitCode::SUCCESS
        }
        None => ExitCode::SUCCESS,
    }
}

fn status() -> ExitCode {
    let mut conexao = match conectar() {
        Ok(c) => c,
        Err(codigo) => return codigo,
    };
    if ipc::enviar_linha(&mut conexao, &Requisicao::Status).is_err() {
        return ExitCode::from(SAIDA_FALHA);
    }
    match ler_resposta(conexao) {
        Some(Resposta::Status {
            versao,
            sessoes,
            fila,
            silenciado,
        }) => {
            println!("daemon      de pé, versão {versao}");
            println!("sessões     {sessoes}");
            println!("fila        {fila}");
            println!("silenciado  {}", if silenciado { "sim" } else { "não" });
            ExitCode::SUCCESS
        }
        _ => {
            eprintln!("cvb: resposta inesperada do daemon");
            ExitCode::from(SAIDA_FALHA)
        }
    }
}

fn ler_resposta(conexao: ipc::Conexao) -> Option<Resposta> {
    use std::io::BufRead;
    let mut leitor = std::io::BufReader::new(conexao);
    let mut linha = String::new();
    match leitor.read_line(&mut linha) {
        Ok(0) | Err(_) => None,
        Ok(_) => serde_json::from_str(linha.trim()).ok(),
    }
}

fn nao_implementado(comando: &str) -> ExitCode {
    eprintln!("cvb: `{comando}` ainda não foi implementado.");
    eprintln!("     O contrato pretendido está em docs/pt-BR/specs/interfaces.md.");
    ExitCode::from(SAIDA_FALHA)
}
