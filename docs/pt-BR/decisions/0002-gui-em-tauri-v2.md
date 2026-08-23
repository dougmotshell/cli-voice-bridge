# ADR-0002 — GUI em Tauri v2

**Status:** aceito — 2026-08-23
**Nível C4:** [contêiner](../architecture/02-container.md)
**Specs que esta decisão move:** [interfaces](../specs/interfaces.md), [portability](../specs/portability.md)

## Contexto

O projeto precisa de interface gráfica nos três sistemas, com ícone de bandeja —
o uso principal é ficar de fundo. A tela é pequena: painel ao vivo, configuração,
diagnóstico, lista de vozes. O núcleo já é Rust ([ADR-0001](0001-nucleo-em-rust-com-cliente-de-hook-separado.md)).

## Decisão

Tauri v2. A GUI é um cliente do `hookd` como qualquer outro, sem lógica própria.

## Consequências

**Boas.** Usa a webview do sistema: o pacote fica em megabytes, não em centenas
de megabytes. Bandeja, atalho global e notificação nativa vêm resolvidos nos três
sistemas. Compartilha código e tipos com o núcleo em Rust.

**Ruins.** A webview varia por sistema (WebKitGTK, WKWebView, WebView2) e traz
diferenças de renderização. No Linux, WebKitGTK é dependência que a pessoa
precisa ter.

**Restringe.** A GUI não pode ganhar lógica de política ou de fila. Se ela
precisar de algo que a CLI não tem, o certo é adicionar ao daemon e expor nas
duas — a regra de paridade de [interfaces](../specs/interfaces.md).

## Alternativas

**Electron.** Descartado: centenas de megabytes e um segundo runtime, para uma
tela pequena. **Egui/Iced (Rust nativo).** Considerado; menos dependências, mas
bandeja e integração com o sistema dão mais trabalho, e a tela de configuração
fica mais cara de construir. **Só TUI.** Descartado: sem ícone de bandeja, que é
o que um processo de fundo precisa.
