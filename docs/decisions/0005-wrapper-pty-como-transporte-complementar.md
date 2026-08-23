# ADR-0005 — Wrapper PTY como transporte complementar e opcional

**Status:** aceito — 2026-08-23
**Nível C4:** [contêiner](../architecture/02-container.md)
**Specs que esta decisão move:** [capture-transports](../specs/capture-transports.md), [speech-input](../specs/speech-input.md)

## Contexto

Duas coisas que a pessoa pediu não cabem em hook nenhum:

1. Narrar o texto do assistente enquanto ele é escrito. Só o Claude Code tem um
   evento para isso (`MessageDisplay`); Codex e Copilot, não.
2. Entregar a resposta ditada ao CLI. A TUI não tem entrada programática, e
   simular teclado no sistema é frágil no Wayland e exige permissão de
   Acessibilidade no macOS.

Um pseudo-terminal resolve as duas: o wrapper *é* o terminal do ponto de vista do
CLI, então vê tudo que sai e escreve no que entra — igual no terminal do sistema,
no integrado do VS Code e no do IntelliJ.

## Decisão

Existe um wrapper PTY (`cvb wrap -- <cli>`), **opcional e ligado por CLI** na
configuração. Ele nunca é a única fonte de um momento que o hook já cobre; na
deduplicação, o hook vence. O que só ele cobre — narração e injeção de texto —
degrada com aviso visível quando o parsing falha, nunca em silêncio.

As regras de parsing são versionadas junto da versão detectada do CLI, e
`cvb doctor --pty` testa as regras contra a versão instalada.

## Consequências

**Boas.** Narração contínua nos três CLIs. Injeção de texto sem simular teclado e
sem permissão especial do sistema. Funciona igual em qualquer terminal, inclusive
os integrados de IDE.

**Ruins.** Parsing de TUI quebra a cada redesenho — é manutenção recorrente e
sem aviso do fornecedor. Muda o jeito de abrir os CLIs. E o wrapper fica no meio
do caminho de tudo: um defeito nele atrapalha a sessão inteira, não só a voz.

**Restringe.** Transparência é obrigatória: tudo que entra sai, byte a byte,
inclusive sequências de controle, redimensionamento e sinais. Wrapper que
"melhora" a saída é wrapper quebrado.

## Alternativas

**Simular teclado pelo sistema.** Descartado como caminho principal: Wayland
restringe, macOS pede Acessibilidade, e depende de foco de janela. Continua
disponível como o caminho de menor confiança.

**Área de transferência e a pessoa cola.** É o padrão do ditado, justamente por
nunca quebrar — mas não resolve narração e não é fluido.

**Só ACP.** Resolve as duas coisas de forma limpa, ao custo da TUI. É um modo
paralelo, não substituto.
