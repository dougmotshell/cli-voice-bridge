//! Transporte local entre o daemon e os clientes.
//!
//! Socket UNIX no Linux e no macOS. No Windows seria named pipe — **ainda não
//! implementado**, e a função diz isso em vez de fingir que funciona
//! (ADR-0008).
//!
//! Nunca porta TCP. Se você se pegou querendo abrir uma, leia o ADR-0008 antes.

use std::io::{self, BufRead, BufReader};
use std::path::Path;

/// Qualquer coisa por onde dá para conversar: leitura, escrita, entre threads.
pub trait Canal: io::Read + io::Write + Send {}
impl<T: io::Read + io::Write + Send> Canal for T {}

/// Uma conexão aberta com o outro lado.
pub type Conexao = Box<dyn Canal>;

#[cfg(windows)]
fn nao_implementado_no_windows() -> io::Error {
    io::Error::new(
        io::ErrorKind::Unsupported,
        "TODO: named pipes no Windows ainda não implementados — ver ADR-0008 e docs/pt-BR/specs/portability.md",
    )
}

/// Conecta a um daemon que já está de pé.
///
/// Erro aqui é situação normal, não excepcional: o daemon pode simplesmente não
/// estar rodando. Quem chama decide o que fazer — e no caso do `hookc` a
/// resposta é sair em silêncio, nunca travar o agente de IA.
pub fn conectar(endereco: &Path) -> io::Result<Conexao> {
    #[cfg(unix)]
    {
        let fluxo = std::os::unix::net::UnixStream::connect(endereco)?;
        Ok(Box::new(fluxo))
    }
    #[cfg(windows)]
    {
        let _ = endereco;
        Err(nao_implementado_no_windows())
    }
}

/// O lado que aceita conexões.
pub struct Ouvinte {
    #[cfg(unix)]
    escutador: std::os::unix::net::UnixListener,
    #[cfg(unix)]
    endereco: std::path::PathBuf,
    #[cfg(windows)]
    #[allow(dead_code)]
    _privado: (),
}

impl Ouvinte {
    /// Abre o ponto de escuta, removendo um socket órfão de uma execução
    /// anterior que morreu sem limpar.
    pub fn abrir(endereco: &Path) -> io::Result<Ouvinte> {
        #[cfg(unix)]
        {
            if endereco.exists() {
                // Socket de um daemon que morreu: se ninguém atende, é lixo.
                if std::os::unix::net::UnixStream::connect(endereco).is_ok() {
                    return Err(io::Error::new(
                        io::ErrorKind::AddrInUse,
                        "já existe um daemon atendendo neste endereço",
                    ));
                }
                std::fs::remove_file(endereco)?;
            }
            if let Some(pai) = endereco.parent() {
                std::fs::create_dir_all(pai)?;
            }
            let escutador = std::os::unix::net::UnixListener::bind(endereco)?;
            restringir_permissoes(endereco)?;
            Ok(Ouvinte {
                escutador,
                endereco: endereco.to_path_buf(),
            })
        }
        #[cfg(windows)]
        {
            let _ = endereco;
            Err(nao_implementado_no_windows())
        }
    }

    /// Bloqueia até chegar uma conexão.
    pub fn aceitar(&self) -> io::Result<Conexao> {
        #[cfg(unix)]
        {
            let (fluxo, _) = self.escutador.accept()?;
            Ok(Box::new(fluxo))
        }
        #[cfg(windows)]
        {
            Err(nao_implementado_no_windows())
        }
    }
}

impl Drop for Ouvinte {
    fn drop(&mut self) {
        #[cfg(unix)]
        {
            let _ = std::fs::remove_file(&self.endereco);
        }
    }
}

/// Só o dono lê e escreve.
///
/// É isto que faz o controle de acesso do daemon — não há autenticação
/// inventada por cima (ADR-0008). Por aqui trafega o conteúdo do trabalho.
#[cfg(unix)]
fn restringir_permissoes(endereco: &Path) -> io::Result<()> {
    use std::os::unix::fs::PermissionsExt;
    std::fs::set_permissions(endereco, std::fs::Permissions::from_mode(0o600))
}

/// Escreve uma mensagem JSON seguida de `\n`.
pub fn enviar_linha<T: serde::Serialize>(destino: &mut dyn io::Write, msg: &T) -> io::Result<()> {
    let linha = serde_json::to_string(msg).map_err(io::Error::other)?;
    destino.write_all(linha.as_bytes())?;
    destino.write_all(b"\n")?;
    destino.flush()
}

/// Lê mensagens JSON, uma por linha.
///
/// Linha malformada é **pulada com aviso**, não fatal: o outro lado pode ser uma
/// versão diferente, e derrubar a conexão por causa de uma linha estranha é pior
/// que ignorá-la.
pub fn ler_linhas<T, F>(origem: Conexao, mut ao_receber: F) -> io::Result<()>
where
    T: serde::de::DeserializeOwned,
    F: FnMut(T),
{
    let leitor = BufReader::new(origem);
    for linha in leitor.lines() {
        let linha = linha?;
        if linha.trim().is_empty() {
            continue;
        }
        match serde_json::from_str::<T>(&linha) {
            Ok(msg) => ao_receber(msg),
            Err(e) => eprintln!("cvb: linha ignorada, não decodifica: {e}"),
        }
    }
    Ok(())
}
