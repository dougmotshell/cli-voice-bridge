> Translation of [`rust-sources.md`](rust-sources.md), the source of truth. No
> frontmatter on purpose: only the pt-BR file carries `paths:` and is loaded as a
> rule. This one is for humans to read.

# Rust code in this project

**`hookc` is a hot path.** It runs in series with the AI agent, hundreds of times
per session. No model loading, no network, no heavy configuration reads, no
parsing beyond what routing requires. A new dependency in `hookc` needs
justification; in `hookd` it does not.

**A hook failure never blocks the agent.** On the `hookc` path, every error
becomes an exit with code 0 and silence. `unwrap()` and `expect()` there are
defects.

**`core` does not depend on `adapters`.** The arrow always points
`adapters → core`. That is what lets a fourth CLI be added without touching the
core ([ADR-0007](../../docs/en-US/decisions/0007-esquema-canonico-de-momentos.md)).

**A third party's payload is data, not a contract.** A missing field or an
unexpected type is normal — the CLIs change without warning. Deserialize
tolerantly and turn the unknown into `error` with the raw name, never into a
panic.

**No hardcoded platform paths.** Configuration, socket, cache, and log paths come
from functions that know all three systems. A literal `~/.config` in the code is
a defect ([portability](../../docs/en-US/specs/portability.md)).

**Secrets never reach the log.** `redact` runs before the template and before any
write to disk. If you are logging a raw payload to debug, it is temporary and
comes out before the commit.

**Prose in pt-BR, identifiers in en-US.** Comments, error messages, and module
documentation in Portuguese with full diacritics; function, type, and variable
names in English.

`cargo clippy -- -D warnings` passes before committing.
