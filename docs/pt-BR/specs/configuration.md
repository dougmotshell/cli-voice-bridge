# Spec — Configuração

**Capacidade:** deixar tudo configurável — por CLI, por projeto, por momento —
sem transformar o arquivo de configuração num quebra-cabeça.

**ADRs que restringem este spec:** [ADR-0007](../decisions/0007-esquema-canonico-de-momentos.md).
**Nível C4:** [componente](../architecture/03-component.md) — `core::config`.

## Problema

O pedido é explícito: escolher em quais CLIs aplicar, se responde por voz ou por
texto, o que fala e o que não fala. Isso é um espaço grande de combinações. Sem
uma estrutura de precedência clara, vira um arquivo que ninguém entende e uma GUI
que não bate com a CLI.

## Escopo

Dentro: formato, localização, precedência, o conjunto de chaves, validação e a
edição pelas duas interfaces.
Fora: o significado de cada política — está nos specs de
[saída](speech-output.md) e [entrada](speech-input.md).

## Camadas, da mais fraca para a mais forte

1. Padrões embutidos no binário.
2. Configuração da pessoa: `~/.config/cli-voice-bridge/config.toml` (Linux),
   `~/Library/Application Support/cli-voice-bridge/` (macOS),
   `%APPDATA%\cli-voice-bridge\` (Windows).
3. Configuração do projeto: `.cli-voice-bridge.toml` na raiz do repositório.
4. Perfil ativo (`cvb profile use <nome>`) — ex.: `reuniao` cala tudo menos
   crítico; `foco` liga narração.
5. Variáveis de ambiente `CVB_*`.
6. Argumentos de linha de comando.

Configuração de projeto **não** pode ligar coisa que a pessoa desligou por
segurança (microfone, autorização por voz de comando destrutivo). Repositório
clonado não manda no seu microfone.

## Formato

TOML. Mesmo formato do `config.toml` do Codex, legível e diffável.

```toml
[geral]
voz = "douglas"           # nome cadastrado no voice-clone
idioma = "pt-BR"
perfil = "padrao"

[voice_clone]
raiz = "~/www/voice-clone"      # nunca embutido no código
python = "{raiz}/.venv/bin/python"

[cli.claude]
ativo = true
transportes = ["hook", "pty"]
resposta = "voz"          # "voz" | "texto" | "ambos"

[cli.codex]
ativo = true
transportes = ["hook", "notify"]
resposta = "texto"

[cli.copilot]
ativo = false

[momentos."decision.needed"]
falar = "sempre"          # "sempre" | "ausente" | "nunca"
interrompe = true
molde = "O {cli} quer rodar {ferramenta}. Autorizo?"

[momentos."tool.started"]
falar = "nunca"

[escuta]
acionamento = "push-to-talk"
atalho = "Ctrl+Alt+Space"
motor = "TODO"
confirmar_destrutivo = true

[privacidade]
modo_discreto = false
redigir = ["token", "senha", "chave", "Bearer "]
retencao_log_dias = 7
```

TODO: fixar o esquema quando o núcleo existir. A tabela acima é a intenção, não
um contrato fechado — a única parte que já é contrato é a precedência.

## Validação

`cvb config check` valida e explica o que está errado em pt-BR, apontando a
linha. Chave desconhecida é **aviso**, não erro fatal: configuração de uma versão
mais nova não pode quebrar uma mais velha. Valor inválido é erro.

`cvb doctor` vai além: confere que o `voice-clone` está no caminho declarado, que
a voz existe, que o dispositivo de áudio responde, que os hooks estão instalados
nos CLIs marcados como ativos e que o daemon está de pé.

## Paridade CLI ↔ GUI

Toda chave é editável nas duas interfaces. A GUI escreve o mesmo TOML, com
comentários preservados — a pessoa que editou à mão não perde o que escreveu.
Recarga a quente: mudança de arquivo recarrega o daemon sem derrubar a fila.

## Alternativas consideradas

**JSON.** Descartado: sem comentários, e o arquivo é para ser lido e editado à
mão. **YAML.** Descartado: ambiguidades de tipo que geram bug de configuração.
**Banco de dados com edição só pela GUI.** Descartado: mata o versionamento e o
diff, e obriga a abrir GUI para uma mudança de uma linha.

## Plano de teste

TODO: escrever. Mínimo: teste de precedência camada a camada; teste de que
configuração de projeto não consegue ligar o microfone nem a autorização de
destrutivo; e teste de ida e volta GUI → arquivo preservando comentários.

## Questões em aberto

- Esquema definitivo (acima).
- Onde ficam os perfis: seções no mesmo arquivo ou arquivos separados.
