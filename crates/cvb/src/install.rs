//! Instalação e remoção dos hooks nos três CLIs.
//!
//! **A regra que manda em tudo aqui: compor, nunca substituir.** Esta máquina já
//! tem hooks de terceiros — `rtk hook claude` em `PreToolUse`, no Claude e no
//! Codex. Um instalador que regrava `settings.json` inteiro apaga trabalho
//! alheio, e é o tipo de estrago que só aparece quando a outra ferramenta para
//! de funcionar sem motivo aparente (ADR-0004).
//!
//! Por isso: ler o arquivo, acrescentar as entradas do `cvb`, preservar o resto,
//! e mostrar o diff antes de escrever. `serde_json` roda com `preserve_order`
//! justamente para não embaralhar a ordem das chaves de quem já estava lá.
//!
//! Reconhecer o que é nosso: a entrada é do `cvb` quando o comando dela menciona
//! o binário `cvb-hook`. JSON não tem comentário, então não há como carimbar de
//! outro jeito — e é isto que `desinstalar` usa para tirar só o que instalou.

use std::path::{Path, PathBuf};

use serde_json::{json, Map, Value};

use crate::diff;

/// O que identifica uma entrada como nossa.
const MARCA: &str = "cvb-hook";

/// Eventos que valem a pena assinar, por CLI.
///
/// **`PreToolUse` e `PostToolUse` ficam de fora no Claude e no Codex, de
/// propósito.** São momentos silenciosos por padrão, e `PreToolUse` é o caminho
/// quente onde o `rtk` já mora — assinar os dois custaria em toda chamada de
/// ferramenta para não falar nada. Quem quiser narração liga na configuração e
/// acrescenta o hook à mão.
///
/// No Copilot, `preToolUse` entra **com matcher `ask_user`**, porque é assim que
/// aquele agente faz pergunta: sem isso, o momento em que a pessoa é necessária
/// passaria despercebido.
const EVENTOS_CLAUDE: &[(&str, Option<&str>)] = &[
    ("PermissionRequest", None),
    ("Notification", Some("permission_prompt|idle_prompt")),
    ("Elicitation", None),
    ("Stop", None),
    ("StopFailure", None),
    ("SubagentStart", None),
    ("SubagentStop", None),
    ("TaskCompleted", None),
    ("PostToolUseFailure", None),
    ("SessionStart", None),
    ("SessionEnd", None),
];

const EVENTOS_CODEX: &[(&str, Option<&str>)] = &[
    ("PermissionRequest", None),
    ("Stop", None),
    ("SubagentStart", None),
    ("SubagentStop", None),
    ("UserPromptSubmit", None),
    ("SessionStart", None),
    ("SessionEnd", None),
];

const EVENTOS_COPILOT: &[(&str, Option<&str>)] = &[
    ("permissionRequest", None),
    ("notification", None),
    ("agentStop", None),
    ("subagentStart", None),
    ("subagentStop", None),
    ("postToolUseFailure", None),
    ("errorOccurred", None),
    ("preToolUse", Some("ask_user")),
    ("sessionStart", None),
    ("sessionEnd", None),
];

/// O que fazer com um arquivo, já decidido mas ainda não escrito.
pub struct Plano {
    pub cli: &'static str,
    pub caminho: PathBuf,
    pub antes: String,
    pub depois: String,
    /// Eventos que esta operação acrescenta ou remove. Existe porque o diff
    /// completo de um `settings.json` passa de duzentas linhas e esconde a
    /// mudança em vez de mostrá-la.
    pub eventos: Vec<String>,
    pub avisos: Vec<String>,
}

impl Plano {
    pub fn muda(&self) -> bool {
        self.antes != self.depois
    }
}

#[derive(Debug)]
pub struct Erro(pub String);

impl std::fmt::Display for Erro {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.0)
    }
}

fn casa() -> PathBuf {
    std::env::var_os("HOME")
        .or_else(|| std::env::var_os("USERPROFILE"))
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from("."))
}

/// Onde está o `cvb-hook`.
///
/// Irmão do binário em execução, quando existir — assim o hook continua
/// apontando para a mesma instalação que o instalou. Senão, o nome pelado, e que
/// o PATH resolva.
pub fn caminho_do_hookc() -> String {
    if let Ok(eu) = std::env::current_exe() {
        if let Some(dir) = eu.parent() {
            let irmao = dir.join(if cfg!(windows) {
                "cvb-hook.exe"
            } else {
                "cvb-hook"
            });
            if irmao.is_file() {
                return irmao.to_string_lossy().into_owned();
            }
        }
    }
    MARCA.to_string()
}

fn comando(origem: &str) -> String {
    let bin = caminho_do_hookc();
    let bin = if bin.contains(char::is_whitespace) {
        format!("\"{bin}\"")
    } else {
        bin
    };
    format!("{bin} --origem {origem} --transporte hook")
}

fn ler(caminho: &Path) -> Result<String, Erro> {
    if !caminho.exists() {
        return Ok(String::new());
    }
    std::fs::read_to_string(caminho)
        .map_err(|e| Erro(format!("não consegui ler {}: {e}", caminho.display())))
}

fn json_de(texto: &str, caminho: &Path) -> Result<Value, Erro> {
    if texto.trim().is_empty() {
        return Ok(Value::Object(Map::new()));
    }
    serde_json::from_str(texto)
        .map_err(|e| Erro(format!("{} não é JSON válido: {e}", caminho.display())))
}

fn objeto<'a>(raiz: &'a mut Value, chave: &str) -> &'a mut Map<String, Value> {
    if !raiz.get(chave).map(Value::is_object).unwrap_or(false) {
        raiz[chave] = json!({});
    }
    raiz[chave].as_object_mut().expect("acabou de virar objeto")
}

fn e_nosso(v: &Value) -> bool {
    serde_json::to_string(v)
        .map(|s| s.contains(MARCA))
        .unwrap_or(false)
}

// --- Claude e Codex: mesma forma de arquivo -------------------------------
//
// `hooks.<Evento>[] -> { matcher?, hooks: [ { type: "command", command } ] }`

fn plano_aninhado(
    cli: &'static str,
    caminho: PathBuf,
    eventos: &[(&str, Option<&str>)],
    origem: &str,
    remover: bool,
) -> Result<Plano, Erro> {
    let antes = ler(&caminho)?;
    let mut raiz = json_de(&antes, &caminho)?;
    if !raiz.is_object() {
        return Err(Erro(format!(
            "{} não tem um objeto na raiz",
            caminho.display()
        )));
    }

    let cmd = comando(origem);
    {
        let hooks = objeto(&mut raiz, "hooks");

        for (evento, matcher) in eventos {
            let lista = hooks
                .entry(evento.to_string())
                .or_insert_with(|| Value::Array(vec![]));
            let Some(entradas) = lista.as_array_mut() else {
                return Err(Erro(format!(
                    "{}: `hooks.{evento}` não é uma lista",
                    caminho.display()
                )));
            };

            // Tira o que já era nosso antes de decidir: assim reinstalar
            // atualiza em vez de duplicar.
            entradas.retain(|e| !e_nosso(e));

            if !remover {
                let mut entrada = json!({
                    "hooks": [ { "type": "command", "command": cmd, "timeout": 5 } ]
                });
                if let Some(m) = matcher {
                    entrada["matcher"] = json!(m);
                }
                entradas.push(entrada);
            }
        }

        // Evento que ficou sem nenhuma entrada some — não deixamos chave vazia
        // de lembrança num arquivo que não é nosso.
        hooks.retain(|_, v| !v.as_array().map(|a| a.is_empty()).unwrap_or(false));
    }

    if raiz["hooks"]
        .as_object()
        .map(Map::is_empty)
        .unwrap_or(false)
    {
        raiz.as_object_mut().expect("objeto").remove("hooks");
    }

    let mut avisos = Vec::new();
    if cli == "codex" {
        avisos.push(
            "o Codex guarda um `trusted_hash` do comando de hook em config.toml — \
             abra uma sessão do Codex e confirme, senão o hook fica inerte"
                .into(),
        );
        avisos.push(
            "o `notify` do config.toml não é tocado por aqui — é redundante com o hook `Stop`. \
             Ver docs/pt-BR/specs/capture-transports.md"
                .into(),
        );
    }

    Ok(Plano {
        cli,
        caminho,
        antes,
        depois: formatar(&raiz),
        eventos: eventos.iter().map(|(e, _)| e.to_string()).collect(),
        avisos,
    })
}

// --- Copilot: arquivo só nosso -------------------------------------------
//
// `~/.copilot/hooks/` é um diretório de arquivos JSON. Ou seja: dá para ter um
// arquivo só nosso, e aí instalar é escrever e desinstalar é apagar — sem
// encostar em nada de terceiros.

fn plano_copilot(remover: bool) -> Result<Plano, Erro> {
    let caminho = casa().join(".copilot/hooks/cli-voice-bridge.json");
    let antes = ler(&caminho)?;

    if remover {
        return Ok(Plano {
            cli: "copilot",
            caminho,
            antes,
            depois: String::new(),
            eventos: EVENTOS_COPILOT.iter().map(|(e, _)| e.to_string()).collect(),
            avisos: Vec::new(),
        });
    }

    let bin = caminho_do_hookc();
    let mut hooks = Map::new();
    for (evento, matcher) in EVENTOS_COPILOT {
        let mut entrada = json!({
            "type": "command",
            "bash": format!("{bin} --origem copilot --transporte hook"),
            "powershell": format!("& '{bin}' --origem copilot --transporte hook"),
            "timeoutSec": 5
        });
        if let Some(m) = matcher {
            entrada["matcher"] = json!(m);
        }
        hooks.insert(evento.to_string(), Value::Array(vec![entrada]));
    }

    let raiz = json!({ "version": 1, "hooks": Value::Object(hooks) });
    Ok(Plano {
        cli: "copilot",
        caminho,
        antes,
        depois: formatar(&raiz),
        eventos: EVENTOS_COPILOT.iter().map(|(e, _)| e.to_string()).collect(),
        avisos: Vec::new(),
    })
}

fn formatar(v: &Value) -> String {
    let mut s = serde_json::to_string_pretty(v).unwrap_or_default();
    s.push('\n');
    s
}

// --- API ------------------------------------------------------------------

pub fn planejar(clis: &[String], remover: bool) -> Result<Vec<Plano>, Erro> {
    let quer = |nome: &str| clis.is_empty() || clis.iter().any(|c| c == nome);
    let mut planos = Vec::new();

    if quer("claude") {
        planos.push(plano_aninhado(
            "claude",
            casa().join(".claude/settings.json"),
            EVENTOS_CLAUDE,
            "claude",
            remover,
        )?);
    }
    if quer("codex") {
        planos.push(plano_aninhado(
            "codex",
            casa().join(".codex/hooks.json"),
            EVENTOS_CODEX,
            "codex",
            remover,
        )?);
    }
    if quer("copilot") {
        planos.push(plano_copilot(remover)?);
    }

    if planos.is_empty() {
        return Err(Erro(format!(
            "nenhum CLI reconhecido em {:?} — use claude, codex ou copilot",
            clis
        )));
    }
    Ok(planos)
}

/// Escreve, guardando uma cópia do que estava lá.
///
/// O backup não é zelo excessivo: reescrever o JSON normaliza a formatação da
/// pessoa, e ainda que o conteúdo seja preservado, ter o original de volta é o
/// que torna a operação reversível de fato.
pub fn aplicar(plano: &Plano) -> Result<(), Erro> {
    if !plano.muda() {
        return Ok(());
    }
    if let Some(pai) = plano.caminho.parent() {
        std::fs::create_dir_all(pai)
            .map_err(|e| Erro(format!("não criei {}: {e}", pai.display())))?;
    }
    if !plano.antes.is_empty() {
        let backup = plano.caminho.with_extension("cvb-backup");
        std::fs::write(&backup, &plano.antes)
            .map_err(|e| Erro(format!("não gravei o backup {}: {e}", backup.display())))?;
    }
    if plano.depois.is_empty() {
        std::fs::remove_file(&plano.caminho)
            .map_err(|e| Erro(format!("não removi {}: {e}", plano.caminho.display())))?;
    } else {
        std::fs::write(&plano.caminho, &plano.depois)
            .map_err(|e| Erro(format!("não gravei {}: {e}", plano.caminho.display())))?;
    }
    Ok(())
}

pub fn relatar(plano: &Plano, remover: bool, diff_completo: bool) {
    println!("\n{} — {}", plano.cli, plano.caminho.display());
    if !plano.muda() {
        println!("  já está como devia; nada a fazer");
    } else if diff_completo {
        for linha in diff::resumido(&plano.antes, &plano.depois).lines() {
            println!("  {linha}");
        }
    } else {
        // O resumo por evento é o que a pessoa precisa ver; o diff é para
        // quando ela quiser conferir byte a byte.
        let sinal = if remover { '-' } else { '+' };
        println!("  {sinal} {}", plano.eventos.join(", "));
        if plano.antes.is_empty() {
            println!("  (arquivo novo, só do cvb)");
        } else {
            println!("  (o resto do arquivo fica como está)");
        }
    }
    for aviso in &plano.avisos {
        println!("  aviso: {aviso}");
    }
}

#[cfg(test)]
mod testes {
    use super::*;

    fn com_hook_de_terceiro() -> String {
        serde_json::to_string_pretty(&json!({
            "permissions": { "allow": ["Bash(rtk read *)"] },
            "hooks": {
                "PreToolUse": [ {
                    "matcher": "Bash",
                    "hooks": [ { "type": "command", "command": "rtk hook claude" } ]
                } ]
            },
            "theme": "dark"
        }))
        .unwrap()
    }

    /// Diretório único por chamada: os testes rodam em paralelo, e derivar o
    /// nome do conteúdo fazia dois deles brigarem pelo mesmo arquivo.
    fn planejar_sobre(texto: &str, remover: bool) -> Value {
        use std::sync::atomic::{AtomicUsize, Ordering};
        static SEQ: AtomicUsize = AtomicUsize::new(0);
        let dir = std::env::temp_dir().join(format!(
            "cvb-teste-install-{}-{}",
            std::process::id(),
            SEQ.fetch_add(1, Ordering::Relaxed)
        ));
        std::fs::create_dir_all(&dir).unwrap();
        let arquivo = dir.join("settings.json");
        std::fs::write(&arquivo, texto).unwrap();
        let plano =
            plano_aninhado("claude", arquivo.clone(), EVENTOS_CLAUDE, "claude", remover).unwrap();
        let _ = std::fs::remove_dir_all(&dir);
        serde_json::from_str(&plano.depois).unwrap()
    }

    #[test]
    fn o_hook_de_terceiro_sobrevive() {
        // O teste que justifica todo este módulo.
        let depois = planejar_sobre(&com_hook_de_terceiro(), false);
        let pre = &depois["hooks"]["PreToolUse"];
        assert_eq!(pre[0]["hooks"][0]["command"], "rtk hook claude");
        assert_eq!(depois["theme"], "dark");
        assert_eq!(depois["permissions"]["allow"][0], "Bash(rtk read *)");
    }

    #[test]
    fn instalar_acrescenta_os_nossos_eventos() {
        let depois = planejar_sobre(&com_hook_de_terceiro(), false);
        for (evento, _) in EVENTOS_CLAUDE {
            assert!(
                depois["hooks"][evento].is_array(),
                "faltou o evento {evento}"
            );
        }
        assert!(
            depois["hooks"]["PermissionRequest"][0]["hooks"][0]["command"]
                .as_str()
                .unwrap()
                .contains(MARCA)
        );
    }

    #[test]
    fn reinstalar_nao_duplica() {
        let uma = planejar_sobre(&com_hook_de_terceiro(), false);
        let duas = planejar_sobre(&serde_json::to_string_pretty(&uma).unwrap(), false);
        assert_eq!(duas["hooks"]["Stop"].as_array().unwrap().len(), 1);
    }

    #[test]
    fn desinstalar_tira_so_o_que_e_nosso() {
        let instalado = planejar_sobre(&com_hook_de_terceiro(), false);
        let limpo = planejar_sobre(&serde_json::to_string_pretty(&instalado).unwrap(), true);

        // O nosso saiu...
        assert!(limpo["hooks"].get("PermissionRequest").is_none());
        assert!(limpo["hooks"].get("Stop").is_none());
        // ...e o do vizinho ficou, intacto.
        assert_eq!(
            limpo["hooks"]["PreToolUse"][0]["hooks"][0]["command"],
            "rtk hook claude"
        );
        assert_eq!(limpo["theme"], "dark");
    }

    #[test]
    fn arquivo_inexistente_vira_configuracao_nova() {
        let depois = planejar_sobre("", false);
        assert!(depois["hooks"]["Stop"].is_array());
    }

    #[test]
    fn notification_entra_com_matcher() {
        // Sem matcher, assinaríamos auth_success e todo o resto para nada.
        let depois = planejar_sobre("", false);
        assert_eq!(
            depois["hooks"]["Notification"][0]["matcher"],
            "permission_prompt|idle_prompt"
        );
    }

    #[test]
    fn o_copilot_ganha_arquivo_proprio_e_some_ao_desinstalar() {
        let instalar = plano_copilot(false).unwrap();
        assert!(instalar.caminho.ends_with("cli-voice-bridge.json"));
        let v: Value = serde_json::from_str(&instalar.depois).unwrap();
        assert_eq!(v["version"], 1);
        assert_eq!(v["hooks"]["preToolUse"][0]["matcher"], "ask_user");
        assert!(v["hooks"]["agentStop"][0]["bash"].is_string());

        let remover = plano_copilot(true).unwrap();
        assert!(remover.depois.is_empty());
    }
}
