# ADR-0002 — GUI in Tauri v2

> Translation of [`../../pt-BR/decisions/0002-gui-em-tauri-v2.md`](../../pt-BR/decisions/0002-gui-em-tauri-v2.md),
> which is the source of truth.

**Status:** accepted — 2026-08-23
**C4 level:** [container](../architecture/02-container.md)
**Specs this decision moves:** [interfaces](../specs/interfaces.md), [portability](../specs/portability.md)

## Context

The project needs a graphical interface on all three systems, with a tray icon —
the main use is running in the background. The surface is small: live panel,
configuration, diagnostics, list of voices. The core is already Rust
([ADR-0001](0001-nucleo-em-rust-com-cliente-de-hook-separado.md)).

## Decision

Tauri v2. The GUI is a client of `hookd` like any other, with no logic of its own.

## Consequences

**Good.** It uses the system webview: the bundle is measured in megabytes, not
hundreds of megabytes. Tray, global shortcut, and native notifications come
solved on all three systems. It shares code and types with the Rust core.

**Bad.** The webview varies per system (WebKitGTK, WKWebView, WebView2) and
brings rendering differences. On Linux, WebKitGTK is a dependency the person
must have.

**Constrains.** The GUI must not grow policy or queue logic. If it needs
something the CLI does not have, the right move is to add it to the daemon and
expose it in both — the parity rule of [interfaces](../specs/interfaces.md).

## Alternatives

**Electron.** Rejected: hundreds of megabytes and a second runtime, for a small
surface. **Egui/Iced (native Rust).** Considered; fewer dependencies, but tray
and system integration take more work, and the configuration screen gets more
expensive to build. **TUI only.** Rejected: no tray icon, which is exactly what a
background process needs.
