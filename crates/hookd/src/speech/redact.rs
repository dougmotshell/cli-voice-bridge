//! Redação: tira segredo e encurta caminho **antes** de qualquer outra coisa.
//!
//! Falar em voz alta é publicar num ambiente compartilhado — quem está na sala
//! ouve. E o texto de um evento carrega caminho de arquivo, nome de cliente,
//! trecho de código, às vezes um token colado num comando.
//!
//! Roda antes do molde e antes de qualquer escrita em disco
//! (`docs/pt-BR/specs/speech-output.md`). Não há opção de desligar para segredos.
//!
//! Sem `regex` de propósito: as regras aqui são de palavra inteira, e um
//! varredor de tokens é mais fácil de ler, testar e prever do que uma bateria de
//! expressões.

/// O que substitui um segredo na fala.
pub const OMITIDO: &str = "algo omitido";

/// Palavras que anunciam um segredo logo adiante: `token abc`, `senha=abc`.
const MARCADORES: &[&str] = &[
    "token",
    "senha",
    "password",
    "passwd",
    "secret",
    "segredo",
    "chave",
    "key",
    "apikey",
    "api_key",
    "bearer",
    "authorization",
    "credential",
    "credencial",
];

/// Prefixos que já identificam a credencial sozinhos, sem palavra anunciando.
const PREFIXOS_DE_SEGREDO: &[&str] = &[
    "sk-",
    "ghp_",
    "gho_",
    "ghs_",
    "ghu_",
    "github_pat_",
    "xoxb-",
    "xoxp-",
    "AKIA",
    "ASIA",
    "AIza",
    "eyJ",
];

pub struct Redator {
    marcadores: Vec<String>,
}

impl Redator {
    /// `extras` vem de `privacidade.redigir` na configuração.
    pub fn novo(extras: &[String]) -> Redator {
        let mut marcadores: Vec<String> = MARCADORES.iter().map(|m| m.to_string()).collect();
        marcadores.extend(extras.iter().map(|e| e.trim().to_lowercase()));
        Redator { marcadores }
    }

    pub fn redigir(&self, texto: &str) -> String {
        let mut saida: Vec<String> = Vec::new();
        // Ligado por um marcador, apaga o próximo token com cara de valor.
        let mut proximo_e_segredo = false;

        for bruto in texto.split_whitespace() {
            if proximo_e_segredo && parece_valor(bruto) {
                saida.push(OMITIDO.into());
                proximo_e_segredo = false;
                continue;
            }
            proximo_e_segredo = false;

            // `chave=valor` e `chave: valor` no mesmo token.
            if let Some(reescrito) = self.redigir_par(bruto) {
                saida.push(reescrito);
                continue;
            }

            if tem_prefixo_de_segredo(bruto) || parece_credencial(bruto) {
                saida.push(OMITIDO.into());
                continue;
            }

            if let Some(nome) = encurtar_caminho(bruto) {
                saida.push(nome);
                continue;
            }

            if self.e_marcador(bruto) {
                proximo_e_segredo = true;
            }
            saida.push(bruto.to_string());
        }

        saida.join(" ")
    }

    fn e_marcador(&self, token: &str) -> bool {
        let limpo: String = token
            .chars()
            .filter(|c| c.is_alphanumeric() || *c == '_')
            .collect::<String>()
            .to_lowercase();
        !limpo.is_empty() && self.marcadores.contains(&limpo)
    }

    /// `--token=abc123` vira `--token=algo omitido`.
    fn redigir_par(&self, token: &str) -> Option<String> {
        let corte = token.find('=').or_else(|| token.find(':'))?;
        let (chave, resto) = token.split_at(corte);
        let valor = &resto[1..];
        if valor.is_empty() || !self.e_marcador(chave) {
            return None;
        }
        Some(format!("{chave}={OMITIDO}"))
    }
}

/// Um valor plausível para um segredo — não uma palavra solta de prosa.
fn parece_valor(token: &str) -> bool {
    token.len() >= 6 && !token.chars().all(|c| c.is_alphabetic()) || token.len() >= 12
}

fn tem_prefixo_de_segredo(token: &str) -> bool {
    let limpo = token.trim_matches(|c: char| !c.is_alphanumeric() && c != '_' && c != '-');
    PREFIXOS_DE_SEGREDO.iter().any(|p| limpo.starts_with(p))
}

/// Cadeia longa, sem espaço, misturando maiúscula, minúscula e dígito.
///
/// Deliberadamente conservador: prefere deixar passar uma string estranha a
/// engolir uma palavra normal, porque falar `algo omitido` no meio de uma frase
/// comum é pior que falar a frase.
fn parece_credencial(token: &str) -> bool {
    let limpo = token.trim_matches(|c: char| !c.is_alphanumeric());
    if limpo.len() < 24 {
        return false;
    }
    let tem_minuscula = limpo.chars().any(|c| c.is_lowercase());
    let tem_maiuscula = limpo.chars().any(|c| c.is_uppercase());
    let tem_digito = limpo.chars().any(|c| c.is_ascii_digit());
    let so_permitido = limpo
        .chars()
        .all(|c| c.is_ascii_alphanumeric() || c == '_' || c == '-');
    tem_minuscula && tem_maiuscula && tem_digito && so_permitido
}

/// Caminho absoluto vira só o nome do arquivo.
///
/// Ninguém precisa ouvir `/home/fulano/clientes/acme/src/main.rs`; `main.rs`
/// diz tudo que importa e não conta onde a pessoa mora nem para quem trabalha.
fn encurtar_caminho(token: &str) -> Option<String> {
    let absoluto = token.starts_with('/')
        || token.starts_with("~/")
        || token.starts_with("\\\\")
        || (token.len() > 3 && token.as_bytes()[1] == b':' && token.as_bytes()[2] == b'\\');
    if !absoluto {
        return None;
    }
    let ultimo = token.rsplit(['/', '\\']).find(|p| !p.is_empty())?;
    Some(ultimo.to_string())
}

#[cfg(test)]
mod testes {
    use super::*;

    fn redator() -> Redator {
        Redator::novo(&[])
    }

    #[test]
    fn marcador_apaga_o_valor_seguinte() {
        let r = redator();
        assert!(r.redigir("use o token abc123def").contains(OMITIDO));
        assert!(!r.redigir("use o token abc123def").contains("abc123def"));
    }

    #[test]
    fn par_chave_valor_e_apagado() {
        let r = redator();
        let saida = r.redigir("rodar --api_key=xyz987654");
        assert!(saida.contains(OMITIDO), "{saida}");
        assert!(!saida.contains("xyz987654"), "{saida}");
    }

    #[test]
    fn prefixos_conhecidos_somem_sem_palavra_anunciando() {
        let r = redator();
        for isca in [
            "ghp_16C7e42F292c6912E7710c838347Ae178B4a",
            "sk-abc123",
            "AKIAIOSFODNN7EXAMPLE",
        ] {
            let saida = r.redigir(&format!("valor {isca} aqui"));
            assert!(!saida.contains(isca), "vazou: {saida}");
        }
    }

    #[test]
    fn caminho_absoluto_vira_so_o_arquivo() {
        let r = redator();
        assert_eq!(
            r.redigir("editei /home/fulano/acme/src/main.rs"),
            "editei main.rs"
        );
        assert_eq!(
            r.redigir("veja ~/projetos/segredo/notas.md"),
            "veja notas.md"
        );
    }

    #[test]
    fn prosa_comum_atravessa_intacta() {
        // O risco oposto: redigir demais deixa a fala sem sentido.
        let r = redator();
        let frase = "o Claude quer rodar cargo test e depois abrir um subagente";
        assert_eq!(r.redigir(frase), frase);
    }

    #[test]
    fn marcador_extra_da_configuracao_funciona() {
        let r = Redator::novo(&["cliente".to_string()]);
        let saida = r.redigir("cliente ACME-12345");
        assert!(saida.contains(OMITIDO), "{saida}");
    }
}
