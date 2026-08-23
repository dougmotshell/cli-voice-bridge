> Translation of [`SKILL.md`](SKILL.md), the source of truth. The generator
> ignores this file; only `SKILL.md` is projected into the CLI surfaces.

# New spec

One file per **capability** — not per module, not per source file. If you cannot
state the capability in one sentence that means something to a user, it probably
is not a spec: it is an implementation detail.

## Steps

1. **Copy `templates/spec.md`** to `docs/pt-BR/specs/<kebab-name>.md`. Name in
   en-US, like the existing ones (`speech-output`, `event-normalization`). Then
   create the en-US sibling with the same file name under `docs/en-US/specs/`.

2. **Start with the problem**, not the solution. A capability whose problem you
   cannot write in three sentences is not ready for a spec.

3. **Write "Out of scope" as seriously as "In scope".** It is the part that keeps
   the spec from growing without control, and where you point at whichever other
   spec covers what was left out.

4. **Alternatives considered are mandatory**, with the reason for rejection.

5. **Test plan.** If you do not yet know how to test it, write `TODO:` and say
   what would need to exist in order to know. A spec with no test plan becomes
   code with no tests.

6. **Open questions go at the end, named.** "X is still undecided" is useful
   information; silence about X is hidden debt.

7. **Cross-link:** the spec lists the ADRs that constrain it and the matching C4
   level; go back into those ADRs and add the spec to their *Specs this decision
   moves* line.

## What does not go in a spec

An architecture decision with rejected alternatives — that is an ADR
(`/new-adr`). The spec **cites** the ADR; it does not repeat its reasoning.
