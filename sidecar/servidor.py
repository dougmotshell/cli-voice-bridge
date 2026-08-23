#!/usr/bin/env python3
"""
Sidecar de síntese: mantém o XTTS-v2 carregado e atende o daemon.

Roda com o interpretador do venv do `voice-clone`, porque é lá que o XTTS está
instalado. O `voice-clone` é dependência externa somente leitura (ADR-0003):
importamos o módulo dele, nunca copiamos nem editamos.

    CVB_VOICE_CLONE=/caminho/voice-clone \\
      /caminho/voice-clone/.venv/bin/python sidecar/servidor.py

Estado: esqueleto. O laço e o protocolo estão de pé; a síntese de verdade
depende do `voice-clone` estar instalado e ainda não foi exercitada.
"""

import json
import os
import socket
import sys
import threading
from pathlib import Path

# No Windows, com a saída redirecionada, o encoding padrão é o do locale e os
# acentos estouram com UnicodeEncodeError. Lição já paga no voice-clone.
for _fluxo in (sys.stdout, sys.stderr):
    try:
        _fluxo.reconfigure(encoding="utf-8")
    except (AttributeError, OSError, ValueError):
        pass

NOME_APP = "cli-voice-bridge"


def raiz_voice_clone() -> Path:
    """O caminho vem do ambiente ou da configuração — nunca embutido no código."""
    bruto = os.environ.get("CVB_VOICE_CLONE", "").strip()
    if not bruto:
        raise SystemExit(
            "CVB_VOICE_CLONE não definido. Aponte para a raiz do voice-clone.\n"
            "Ver docs/pt-BR/manual/instalacao.md."
        )
    raiz = Path(bruto).expanduser()
    if not (raiz / "falar.py").is_file():
        raise SystemExit(f"não achei falar.py em {raiz} — o caminho está certo?")
    return raiz


def endereco_socket() -> str:
    """Mesma regra do `cvb_core::caminhos::endereco_sidecar`."""
    explicito = os.environ.get("CVB_SIDECAR_SOCKET", "").strip()
    if explicito:
        return explicito
    base = os.environ.get("XDG_RUNTIME_DIR") or os.environ.get("TMPDIR") or "/tmp"
    return str(Path(base) / f"{NOME_APP}-sidecar.sock")


class Motor:
    """Carrega o XTTS uma vez e sintetiza sob demanda.

    A carga é preguiçosa de propósito: subir o sidecar tem de ser instantâneo,
    mesmo que a primeira fala custe os ~30 segundos do modelo.
    """

    def __init__(self, raiz: Path) -> None:
        self._raiz = raiz
        self._vozclone = None
        self._trava = threading.Lock()

    def _modulo(self):
        if self._vozclone is None:
            # `vozclone` faz `compat.aplicar()` no topo, antes de importar TTS.
            # A ordem é contratual lá; importar TTS direto reintroduz falhas já
            # resolvidas. Por isso importamos o módulo dele, não o TTS.
            sys.path.insert(0, str(self._raiz))
            import vozclone  # noqa: PLC0415  (import tardio é o ponto)

            self._vozclone = vozclone
        return self._vozclone

    def sintetizar(self, texto: str, voz: str, idioma: str, saida: str) -> dict:
        # O XTTS não é reentrante; uma síntese por vez. A fila de prioridade
        # mora no daemon, não aqui (docs/pt-BR/specs/speech-output.md).
        with self._trava:
            r = self._modulo().sintetizar(
                texto=texto, voz=voz, idioma=idioma, saida=saida
            )
        return {
            "tipo": "ok",
            "caminho": str(getattr(r, "caminho", saida)),
            "duracao_s": float(getattr(r, "duracao_audio", 0.0)),
        }

    def vozes(self) -> dict:
        return {"tipo": "vozes", "vozes": list(self._modulo().listar_vozes())}


def atender(conexao: socket.socket, motor: Motor) -> None:
    with conexao, conexao.makefile("rwb") as fluxo:
        for linha in fluxo:
            linha = linha.strip()
            if not linha:
                continue
            try:
                req = json.loads(linha)
            except json.JSONDecodeError as e:
                # Linha estranha não derruba a conexão: pode ser outra versão.
                resposta = {"tipo": "erro", "mensagem": f"json inválido: {e}"}
            else:
                resposta = despachar(req, motor)
            fluxo.write((json.dumps(resposta, ensure_ascii=False) + "\n").encode())
            fluxo.flush()


def despachar(req: dict, motor: Motor) -> dict:
    tipo = req.get("tipo")
    try:
        if tipo == "ping":
            return {"tipo": "ok"}
        if tipo == "vozes":
            return motor.vozes()
        if tipo == "sintetizar":
            return motor.sintetizar(
                texto=req["texto"],
                voz=req["voz"],
                idioma=req.get("idioma", "pt-BR"),
                saida=req["saida"],
            )
        return {"tipo": "erro", "mensagem": f"requisição desconhecida: {tipo}"}
    except Exception as e:  # noqa: BLE001
        # O daemon precisa saber que falhou para cair na voz do sistema, e não
        # ficar mudo. Derrubar o sidecar por uma síntese ruim seria pior.
        return {"tipo": "erro", "mensagem": f"{type(e).__name__}: {e}"}


def main() -> int:
    raiz = raiz_voice_clone()
    endereco = endereco_socket()
    motor = Motor(raiz)

    if os.path.exists(endereco):
        os.unlink(endereco)

    servidor = socket.socket(socket.AF_UNIX, socket.SOCK_STREAM)
    servidor.bind(endereco)
    # Só o dono lê e escreve: é isto que faz o controle de acesso (ADR-0008).
    os.chmod(endereco, 0o600)
    servidor.listen(8)
    print(f"sidecar: escutando em {endereco} (voice-clone em {raiz})")

    try:
        while True:
            conexao, _ = servidor.accept()
            threading.Thread(
                target=atender, args=(conexao, motor), daemon=True
            ).start()
    except KeyboardInterrupt:
        print("\nsidecar: encerrando")
    finally:
        servidor.close()
        if os.path.exists(endereco):
            os.unlink(endereco)
    return 0


if __name__ == "__main__":
    # TODO: no Windows não há AF_UNIX aqui; precisa de named pipe, como o daemon
    # (ADR-0008, docs/pt-BR/specs/portability.md).
    sys.exit(main())
