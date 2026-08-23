//! Diff de linhas, só para o `--dry-run` mostrar o que mudaria.
//!
//! Escrever num arquivo de configuração alheio sem mostrar antes é o tipo de
//! coisa que quebra confiança uma vez e nunca mais recupera. Daí um diff de
//! verdade, e não um "vou acrescentar algumas entradas".

/// LCS clássico. Os arquivos aqui têm dezenas de linhas; O(n·m) está de bom
/// tamanho e evita uma dependência.
pub fn unificado(antes: &str, depois: &str) -> String {
    let a: Vec<&str> = antes.lines().collect();
    let b: Vec<&str> = depois.lines().collect();

    let mut tabela = vec![vec![0usize; b.len() + 1]; a.len() + 1];
    for i in (0..a.len()).rev() {
        for j in (0..b.len()).rev() {
            tabela[i][j] = if a[i] == b[j] {
                tabela[i + 1][j + 1] + 1
            } else {
                tabela[i + 1][j].max(tabela[i][j + 1])
            };
        }
    }

    let mut saida = String::new();
    let (mut i, mut j) = (0, 0);
    while i < a.len() && j < b.len() {
        if a[i] == b[j] {
            saida.push_str(&format!("  {}\n", a[i]));
            i += 1;
            j += 1;
        } else if tabela[i + 1][j] >= tabela[i][j + 1] {
            saida.push_str(&format!("- {}\n", a[i]));
            i += 1;
        } else {
            saida.push_str(&format!("+ {}\n", b[j]));
            j += 1;
        }
    }
    while i < a.len() {
        saida.push_str(&format!("- {}\n", a[i]));
        i += 1;
    }
    while j < b.len() {
        saida.push_str(&format!("+ {}\n", b[j]));
        j += 1;
    }
    saida
}

/// Só as linhas que mudaram, com três de contexto. Um `settings.json` inteiro
/// no terminal esconde a mudança em vez de mostrá-la.
pub fn resumido(antes: &str, depois: &str) -> String {
    let completo = unificado(antes, depois);
    let linhas: Vec<&str> = completo.lines().collect();
    let interessantes: Vec<usize> = linhas
        .iter()
        .enumerate()
        .filter(|(_, l)| l.starts_with('+') || l.starts_with('-'))
        .map(|(i, _)| i)
        .collect();

    if interessantes.is_empty() {
        return String::new();
    }

    let mut mostrar = vec![false; linhas.len()];
    for i in interessantes {
        let inicio = i.saturating_sub(3);
        let fim = (i + 4).min(linhas.len());
        for m in mostrar.iter_mut().take(fim).skip(inicio) {
            *m = true;
        }
    }

    let mut saida = String::new();
    let mut pulou = false;
    for (i, linha) in linhas.iter().enumerate() {
        if mostrar[i] {
            if pulou {
                saida.push_str("  ...\n");
                pulou = false;
            }
            saida.push_str(linha);
            saida.push('\n');
        } else {
            pulou = true;
        }
    }
    saida
}

#[cfg(test)]
mod testes {
    use super::*;

    #[test]
    fn linha_acrescentada_aparece_com_mais() {
        let d = unificado("a\nb\n", "a\nx\nb\n");
        assert!(d.contains("+ x"), "{d}");
        assert!(d.contains("  a"), "{d}");
        assert!(!d.contains("- "), "{d}");
    }

    #[test]
    fn linha_removida_aparece_com_menos() {
        let d = unificado("a\nx\nb\n", "a\nb\n");
        assert!(d.contains("- x"), "{d}");
    }

    #[test]
    fn sem_mudanca_o_resumo_e_vazio() {
        assert!(resumido("a\nb\n", "a\nb\n").is_empty());
    }

    #[test]
    fn o_resumo_corta_o_que_esta_longe_da_mudanca() {
        let antes: String = (0..40).map(|i| format!("linha {i}\n")).collect();
        let depois = antes.replace("linha 20\n", "linha 20\nNOVA\n");
        let r = resumido(&antes, &depois);
        assert!(r.contains("+ NOVA"), "{r}");
        assert!(!r.contains("linha 0"), "contexto demais:\n{r}");
        assert!(r.contains("..."), "faltou marcar o corte:\n{r}");
    }
}
