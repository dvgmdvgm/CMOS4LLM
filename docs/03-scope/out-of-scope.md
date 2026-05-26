# Out of Scope — Things CMOS Will Never Be

> The items below are **not deferred to a later phase**. They are explicit non-goals — fundamental to what CMOS is and is not. If a feature request maps to one of these, the answer is no, regardless of phase.
>
> See [docs/00-charter.md](../00-charter.md) Non-goals section for the corresponding charter-level statement.

---

## Categorical non-goals

### NG.1. CMOS does not modify LLM models

- No fine-tuning, no LoRA training as part of the base product (V3 LoRA-as-memory is research, opt-in, not the core product).
- No custom inference engines beyond what we use to host open-weight Sub-LMs.
- Why: the value of CMOS is being **provider-portable**. Modifying models couples us to a vendor or to a custom training pipeline neither of which is sustainable.

### NG.2. CMOS does not write code for the user

- Code generation is the LLM's job. CMOS prepares context, validates output, persists decisions.
- We do not ship "code-generation as a feature" — the LLM client (Claude Code, Cursor, etc.) does that.
- Why: separation of concerns. CMOS is substrate. Letting it generate code makes it a competing IDE-assistant, which is a different product.

### NG.3. CMOS does not replace documentation or design docs

- READMEs, ADRs (project-level, not CMOS's own), wikis, design docs remain by-humans-for-humans.
- CMOS extracts facts from them, references them, points to them — but does not replace them.
- Why: human-authored documentation has nuance, narrative, audience awareness that machine-extracted facts lack.

### NG.4. CMOS does not replace code review

- Drift Monitor flags violations. It does not approve or reject changes. Human review remains the authoritative checkpoint.
- We do not produce "auto-merge if all CMOS checks pass."
- Why: reviewing intent, not just rule-conformance, is fundamentally human work. Tooling assists; tooling does not decide.

### NG.5. CMOS does not auto-pilot refactors

- Counterfactual mode is **analysis**, not optimization. It shows what would happen; it does not commit.
- "Refactor my entire codebase to satisfy this new policy" is explicitly out — owner makes the calls.
- Why: large-scale automatic mutations are how rope-burns happen. We refuse to be the rope.

### NG.6. CMOS does not compete with MCP / Claude / Cursor / Continue

- We are a layer **under** these clients, not alongside.
- We do not ship our own chat UI as a primary interface (the GUI is for inspection, not for conversational interaction).
- Why: building yet another chat surface is wasted effort. The ecosystem has chat surfaces; what's missing is the substrate.

### NG.7. CMOS is not general-purpose chat memory

- We are not "remember everything I ever told ChatGPT."
- Scope is **project-aware development**. Personal preferences leaking into project memory are a bug, not a feature.
- Why: the design pivots on project as the primary unit of memory (ADR-004). Personal-assistant memory is a different product.

### NG.8. CMOS is not a database

- L4 contains memory, not arbitrary user data.
- We do not offer to store the user's CRM data, customer records, or operational state.
- Why: scope creep into application data muddles persistence semantics.

### NG.9. CMOS does not ship as a toy / proof-of-concept

- Production-grade quality is the bar. "Demo-ware" is not.
- Acceptance criteria in MVP / V1 / V2 / V3 reflect this — they're product, not prototype standards.

### NG.10. CMOS does not silently accept LLM hallucinations as facts

- All facts written to L4 carry a `created_by` (human / Sub-LM / cloud-LM) and an evidence trail.
- LM-generated facts go through validation (cross-reference with code / docs / declared sources) before being marked authoritative.
- Why: a memory system that uncritically stores hallucinations becomes a trusted source of lies.

### NG.11. CMOS does not bypass user authority on its own state

- Every memory mutation is observable. Every policy change is reviewable.
- We do not ship "auto-accept high-confidence drift suggestions" — owner approves explicitly.
- Why: if the system can rewrite its own constitution without owner consent, it's no longer the owner's substrate.

### NG.12. CMOS is not free of cost

- Owner pays in: setup time, hardware (Sub-LM GPU), monitoring discipline (wake-up resilience ritual).
- We do not pretend the cost is zero.
- Why: honest framing prevents disappointment.

---

## What goes here vs. in [future.md](./future.md)

- **Future**: things we may build later. Path is open.
- **Out of scope**: things we will not build, period. Path is closed unless a charter-level revision happens.

If a request feels like it could go either way, default to Future. Out-of-scope is rare and load-bearing.
