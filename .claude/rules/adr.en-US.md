> Translation of [`adr.md`](adr.md), the source of truth. No frontmatter on
> purpose: only the pt-BR file carries `paths:` and is loaded as a rule.

# ADRs

MADR format, template in [`templates/adr.md`](../../templates/adr.md).

**Append-only.** An accepted ADR is never rewritten to change the decision. If the
decision changed, write a new ADR and mark the old one
`Status: substituído por NNNN`. Numbers are never reused, not even after an ADR
is abandoned.

Correcting a factual error inside an accepted ADR is allowed, but write the
revision **inside it**, dated, instead of erasing what was there.

**Every ADR names the C4 level it moves and the specs it constrains**, with a
link. And the matching spec names the ADR back — the reference goes both ways,
otherwise one end rots without anyone noticing.

**Alternatives are mandatory and must be real.** "Do nothing" and an option
nobody considered do not count. An ADR's value is in recording what was rejected
and why — that is what keeps the discussion from coming back in six months.

**Consequences include the bad ones.** An ADR that lists only advantages was not
thought through.

**Both languages.** An ADR is only done when the sibling with the same file name
exists under `docs/en-US/decisions/`.

After creating or renumbering, update the table in
[`docs/pt-BR/decisions/README.md`](../../docs/pt-BR/decisions/README.md) and its
en-US sibling.
