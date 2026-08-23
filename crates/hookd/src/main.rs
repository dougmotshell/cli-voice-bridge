//! `cvb-hookd` — o daemon.
//!
//! Toda a lógica mora aqui: normalização, política, fila, síntese e escuta
//! (ADR-0001). Os clientes — `cvb-hook`, `cvb`, a GUI e o wrapper de PTY — são
//! todos iguais do ponto de vista do daemon.
//!
//! **Estado: esqueleto que roda.** O caminho evento → momento está de pé e é
//! testável. Fala, escuta e fila ainda não existem; ver os `TODO:` abaixo.

mod adapters;
mod normalize;
mod state;

use std::io::BufRead;
use std::sync::{Arc, Mutex};

use cvb_core::ipc::{self, Conexao};
use cvb_core::{caminhos, Requisicao, Resposta, VERSAO_PROTOCOLO};

use state::Estado;

fn main() -> std::process::ExitCode {
    let endereco = caminhos::endereco_daemon();

    let ouvinte = match ipc::Ouvinte::abrir(&endereco) {
        Ok(o) => o,
        Err(e) => {
            eprintln!(
                "cvb-hookd: não consegui escutar em {}: {e}",
                endereco.display()
            );
            return std::process::ExitCode::FAILURE;
        }
    };

    println!("cvb-hookd: escutando em {}", endereco.display());
    // TODO: morrer por sinal (SIGTERM, SIGINT) não roda o `Drop` do `Ouvinte`,
    // então o socket fica no disco. Não é grave — `Ouvinte::abrir` detecta o
    // órfão e o remove —, mas um manipulador de sinal deixaria mais limpo.
    let estado = Arc::new(Mutex::new(Estado::novo()));

    loop {
        match ouvinte.aceitar() {
            Ok(conexao) => {
                let estado = Arc::clone(&estado);
                // Uma thread por conexão. São poucas e de vida curta; um runtime
                // assíncrono não se paga aqui, e são menos dependências no
                // caminho que precisa ser confiável.
                std::thread::spawn(move || atender(conexao, estado));
            }
            Err(e) => eprintln!("cvb-hookd: conexão recusada: {e}"),
        }
    }
}

fn atender(conexao: Conexao, estado: Arc<Mutex<Estado>>) {
    let mut leitor = std::io::BufReader::new(conexao);
    let mut linha = String::new();
    let mut apresentado = false;

    loop {
        linha.clear();
        match leitor.read_line(&mut linha) {
            Ok(0) => return,
            Ok(_) => {}
            Err(e) => {
                eprintln!("cvb-hookd: leitura interrompida: {e}");
                return;
            }
        }
        if linha.trim().is_empty() {
            continue;
        }

        let req: Requisicao = match serde_json::from_str(linha.trim()) {
            Ok(r) => r,
            Err(e) => {
                // Linha estranha não derruba a conexão: pode ser um cliente de
                // outra versão. Ver `ipc::ler_linhas`.
                eprintln!("cvb-hookd: linha ignorada: {e}");
                continue;
            }
        };

        match req {
            Requisicao::Ola { versao, cliente } => {
                if versao != VERSAO_PROTOCOLO {
                    let r = Resposta::VersaoIncompativel {
                        esperada: VERSAO_PROTOCOLO,
                        recebida: versao,
                    };
                    let _ = ipc::enviar_linha(leitor.get_mut(), &r);
                    return;
                }
                apresentado = true;
                eprintln!("cvb-hookd: cliente '{cliente}' conectado");
            }

            _ if !apresentado => {
                let r = Resposta::Erro {
                    mensagem: "handshake ausente: mande `ola` primeiro".into(),
                };
                let _ = ipc::enviar_linha(leitor.get_mut(), &r);
                return;
            }

            Requisicao::EventoBruto {
                origem,
                transporte,
                evento,
                payload,
            } => {
                let normalizado = normalize::normalizar(origem, transporte, &evento, &payload);
                {
                    let mut e = estado.lock().expect("estado envenenado");
                    e.registrar(&normalizado);
                }
                // TODO: daqui em diante falta tudo — deduplicação entre
                // transportes, política (`docs/specs/speech-output.md`),
                // redação, fila e síntese. Por ora só se vê o momento.
                println!(
                    "{} [{}/{}] {}",
                    normalizado.momento,
                    normalizado.origem.falado(),
                    serde_json::to_string(&normalizado.transporte)
                        .unwrap_or_default()
                        .trim_matches('"'),
                    normalizado.texto
                );
            }

            Requisicao::Status => {
                let e = estado.lock().expect("estado envenenado");
                let r = Resposta::Status {
                    versao: env!("CARGO_PKG_VERSION").to_string(),
                    sessoes: e.sessoes(),
                    fila: 0,
                    silenciado: e.silenciado(),
                };
                let _ = ipc::enviar_linha(leitor.get_mut(), &r);
            }

            Requisicao::Silenciar { segundos } => {
                estado
                    .lock()
                    .expect("estado envenenado")
                    .silenciar(segundos);
                let _ = ipc::enviar_linha(leitor.get_mut(), &Resposta::Ok);
            }

            Requisicao::Retomar => {
                estado.lock().expect("estado envenenado").retomar();
                let _ = ipc::enviar_linha(leitor.get_mut(), &Resposta::Ok);
            }

            Requisicao::Falar { texto } => {
                // TODO: ponte com o sidecar do voice-clone (ADR-0003).
                eprintln!("cvb-hookd: TODO falar: {texto}");
                let r = Resposta::Erro {
                    mensagem: "síntese ainda não implementada — ver ADR-0003".into(),
                };
                let _ = ipc::enviar_linha(leitor.get_mut(), &r);
            }

            Requisicao::Assinar => {
                // TODO: empurrar os momentos para este cliente enquanto ele
                // estiver conectado. Precisa de um barramento no `state`.
                let r = Resposta::Erro {
                    mensagem: "assinatura de momentos ainda não implementada".into(),
                };
                let _ = ipc::enviar_linha(leitor.get_mut(), &r);
            }
        }
    }
}
