# GUI — Tauri v2

Janela mais ícone de bandeja. É um cliente do `hookd` como qualquer outro:
nenhuma lógica de política, fila ou síntese mora aqui (ADR-0002 e
`docs/specs/interfaces.md`).

## Estado: não criada

O scaffold do Tauri é gerado por ferramenta, e gerá-lo sem `npm install` deixaria
uma casca quebrada no repositório. Melhor um README honesto.

Quando for a hora, com Rust e Node instalados:

```bash
cd gui
npm create tauri-app@latest .
```

Escolhas que já estão decididas e não precisam ser rediscutidas no assistente:

- **Tauri v2** (ADR-0002), não Electron.
- **Bandeja obrigatória** — o uso principal é ficar de fundo.
- **Front-end:** TODO, ainda não decidido. A tela é pequena; avalie sem
  framework antes de importar um.

## O que a GUI mostra

Especificado em `docs/specs/interfaces.md`:

- Painel ao vivo: momentos chegando, o que está falando, a fila, botão de cortar
- Indicador de microfone **inconfundível** quando está gravando
- Configuração — as mesmas chaves do `config.toml`, preservando comentários
- Vozes do `voice-clone`, com amostra
- O `cvb doctor` em forma de tela

## Paridade

Tudo que a GUI faz, a CLI faz — e vice-versa. As duas únicas divergências
deliberadas estão listadas em `docs/specs/interfaces.md`. Qualquer outra é
defeito.
