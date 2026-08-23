# Spec — Interfaces: CLI e GUI

**Capacidade:** operar tudo por linha de comando e tudo por interface gráfica,
com paridade real.

**ADRs que restringem este spec:** [ADR-0002](../decisions/0002-gui-em-tauri-v2.md),
[ADR-0008](../decisions/0008-ipc-por-socket-local.md).
**Nível C4:** [contêiner](../architecture/02-container.md).

## Problema

O projeto precisa servir dois momentos diferentes: configurar e observar (onde
uma tela ganha) e automatizar e diagnosticar (onde a linha de comando ganha).
Fazer só um dos dois deixa metade do uso desconfortável; fazer os dois sem
disciplina produz duas ferramentas que discordam.

## A regra

Ambas são clientes do mesmo daemon, pelo mesmo IPC. Nenhuma delas tem lógica
própria de política, fila ou síntese. Recurso novo entra no daemon primeiro; CLI
e GUI só expõem. É isso que torna a paridade uma consequência em vez de uma
promessa.

## CLI — `cvb`

| Comando | Faz |
|---|---|
| `cvb doctor` | Diagnóstico completo. Primeiro passo de qualquer investigação |
| `cvb install [--cli …] [--dry-run]` | Instala/atualiza hooks, compondo com os existentes |
| `cvb uninstall [--cli …]` | Remove só o que instalou |
| `cvb daemon [start\|stop\|status\|logs]` | Ciclo de vida do `hookd` |
| `cvb say <texto>` | Testa a saída de voz fim a fim; diz por qual caminho falou |
| `cvb voices` | Lista as vozes cadastradas no `voice-clone` |
| `cvb listen` | Testa a entrada de voz e mostra a transcrição |
| `cvb wrap -- <cli> [args]` | Abre um CLI dentro do wrapper PTY |
| `cvb console --cli <nome>` | Modo cliente de protocolo (ACP/app-server) |
| `cvb config [show\|edit\|check]` | Configuração |
| `cvb profile [list\|use] <nome>` | Perfis |
| `cvb events [--follow] [--json]` | Fluxo de momentos, para depurar e para compor com outras ferramentas |
| `cvb mute [duração]` / `cvb unmute` | Silêncio temporário |

`--json` em tudo que produz dados. Código de saída significa alguma coisa:
0 sucesso, 1 falha de execução, 2 configuração inválida, 3 daemon fora do ar.

## GUI — Tauri v2

Janela normal mais ícone de bandeja, porque o uso principal é ficar de fundo.

- **Painel ao vivo:** momentos chegando, o que está falando agora, o que está na
  fila, e um botão de cortar a fala.
- **Indicador de microfone** inconfundível quando está gravando.
- **Configuração:** as mesmas chaves do TOML, com validação na hora.
- **Vozes:** lista as vozes do `voice-clone` e permite ouvir uma amostra.
- **Diagnóstico:** o `cvb doctor` em forma de tela, com o que está quebrado em
  vermelho e o que fazer a respeito.
- **Bandeja:** silenciar, trocar perfil, abrir a janela, sair.

TODO: decidir o framework de front-end. Nenhuma razão forte para algo pesado; a
tela é pequena. Avaliar sem framework antes de importar um.

## Onde as duas divergem, de propósito

- `cvb wrap` não existe na GUI: é o wrapper de um terminal, e um terminal dentro
  de uma janela gráfica seria outro produto.
- O indicador de bandeja não existe na CLI: `cvb daemon status` é o equivalente.

Qualquer outra divergência é defeito.

## Alternativas consideradas

**Só CLI.** Descartado: configurar dezenas de chaves e observar uma fila em tempo
real é desconfortável em texto puro. **Só GUI.** Descartado: mata o diagnóstico
por `ssh` e a automação. **TUI em vez de GUI.** Considerado; perde o ícone de
bandeja, que é justamente o que um processo de fundo precisa.

## Plano de teste

TODO: escrever. Mínimo: um teste que enumera os comandos da CLI e as ações da GUI
e falha quando algo existe só de um lado sem estar na lista de divergências
deliberadas acima.

## Questões em aberto

- Front-end da GUI (acima).
- Se a GUI deve poder subir o daemon sozinha ou exigir `cvb daemon start`.
