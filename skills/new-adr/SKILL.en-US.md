> Translation of [`SKILL.md`](SKILL.md), the source of truth. The generator
> ignores this file; only `SKILL.md` is projected into the CLI surfaces.

# New ADR

Records an architecture decision. One file, one decision, never rewritten.

## Steps

1. **Confirm it really is an architecture decision.** An ADR is for a choice that
   constrains the future and whose alternative was defensible. Picking a variable
   name is not an ADR; choosing a local socket over a TCP port is.

2. **Find the next number.** `ls docs/pt-BR/decisions/` and add one to the
   highest. Numbers are never reused, not even from an abandoned ADR.

3. **Copy `templates/adr.md`** to
   `docs/pt-BR/decisions/NNNN-kebab-case-title.md`. Follow the naming style of
   the directory: descriptive pt-BR without accents, like the existing ones. Then
   create the en-US sibling with **the same file name** under
   `docs/en-US/decisions/`.

4. **Fill it in without inventing.** What you do not know becomes `TODO:`, never
   a guess.
   - **Context** is what was true when it was decided, including measurements and
     constraints. Someone reading a year from now needs to understand the
     pressure of the moment.
   - **Decision** in the present tense, affirmative.
   - **Consequences** include the bad ones and what the decision now forbids.
   - **Alternatives** must be real and carry the reason they were rejected. This
     is the field that gives an ADR its value.

5. **Cross-link both ways.** The ADR names the C4 level it moves and the specs it
   constrains; go back into those specs and add the ADR to their list. A one-way
   reference rots.

6. **Update the table** in `docs/pt-BR/decisions/README.md` and its en-US sibling.

## If the decision supersedes another

Do not edit the old ADR beyond one line: `Status: substituído por NNNN`. The
history stays. The new ADR explains what changed since the old one — usually a
new measurement or a constraint that fell away.
