//! `cvb-ptywrap` — o wrapper de pseudo-terminal.
//!
//! Transporte **complementar e opcional** (ADR-0005). Existe por duas coisas
//! que nenhum hook entrega: narrar o texto do assistente enquanto ele é escrito,
//! e injetar a resposta ditada no `stdin` do CLI sem simular teclado.
//!
//! **Estado: não implementado.** Este binário existe para o workspace ficar
//! honesto sobre o que falta, e sai com erro em vez de fingir que funciona.
//!
//! ## O contrato quando for implementado
//!
//! **Transparência é obrigatória.** Tudo que entra sai, byte a byte: sequências
//! de controle, redimensionamento de janela (`SIGWINCH`), sinais, código de
//! saída do filho. Wrapper que "melhora" a saída é wrapper quebrado — ele fica
//! no caminho de tudo, e um defeito aqui atrapalha a sessão inteira, não só a
//! voz.
//!
//! **Nunca é a única fonte de um momento que o hook já cobre.** Na deduplicação
//! o hook vence (`Transporte::confianca`). O que só ele cobre — narração e
//! injeção — degrada com aviso visível quando o parsing falha, jamais em
//! silêncio.
//!
//! **As regras de parsing são versionadas junto da versão detectada do CLI**, e
//! `cvb doctor --pty` as testa contra a versão instalada.
//!
//! ## O que falta decidir
//!
//! TODO: a estratégia de reconstrução da tela lógica. Hipótese de trabalho:
//! um interpretador de sequências ANSI mantendo uma tela virtual, com regras
//! por CLI. Ver `docs/specs/capture-transports.md`, seção *pty*.
//!
//! TODO: `openpty` no Linux e no macOS, ConPTY no Windows
//! (`docs/specs/portability.md`).

fn main() -> std::process::ExitCode {
    eprintln!("cvb-ptywrap: ainda não implementado.");
    eprintln!("  O contrato está em docs/specs/capture-transports.md (seção pty)");
    eprintln!(
        "  e as restrições em docs/decisions/0005-wrapper-pty-como-transporte-complementar.md."
    );
    eprintln!();
    eprintln!("  Enquanto isso, os hooks funcionam sozinhos: eles são o transporte");
    eprintln!("  primário e não dependem deste wrapper (ADR-0004).");
    std::process::ExitCode::FAILURE
}
