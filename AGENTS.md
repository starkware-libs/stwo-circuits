# STWO Circuits Agent Playbook

This document defines how autonomous and human-assisted agents should operate in this
repository. When in doubt, optimize for soundness, reproducibility, and reviewability.

## Priority Order (Non-Negotiable)

1. **Soundness and security**
2. **Production quality (tests and CI health)**
3. **Performance**

## Agent Roles

| Role | Responsibility | Hard Boundaries |
|------|----------------|-----------------|
| Orchestrator | Classifies work, routes tasks, integrates outputs | Must not author proof-system logic directly |
| Math Reviewer | Reviews soundness/security-critical reasoning | Must not implement; reviews and escalates |
| Implementer | Executes tests, docs, refactors, infra changes | Must not modify `[SOUNDNESS-CRITICAL]` paths |
| Crypto Specialist | Implements proof-system changes | Requires Math Reviewer sign-off before and after |
| Perf Specialist | Benchmarks and optimizes performance | Must preserve semantics and security parameters |

## Change Routing

### Standard change (non-crypto)

```
User request
  -> Orchestrator classifies
  -> Implementer executes (tests/docs/refactor/infra)
  -> CI verification
```

### Soundness-critical change

```
User request
  -> Orchestrator classifies as soundness-critical
  -> Math Reviewer identifies paper anchor + invariant
  -> Crypto Specialist implements with review guidance
  -> Math Reviewer reviews implementation
  -> Human approval
  -> CI verification
```

### Performance change

```
User request
  -> Orchestrator classifies as performance-sensitive
  -> Perf Specialist records baseline + optimization
  -> Math Reviewer confirms semantic equivalence
  -> CI verification
```

## Escalation Protocol

Escalate to a human immediately when any of the following occur:

1. Undocumented paper-implementation divergence
2. No test coverage on a modified soundness-critical path
3. A proposed change cannot be tied to a paper or formal invariant
4. `unsafe` appears in a soundness-critical path without clear justification
5. Confidence in mathematical correctness drops below acceptable threshold

### Soundness escalation format

```text
SOUNDNESS-ESCALATION:
  File: [path]
  Change: [what is proposed]
  Invariant at risk: [mathematical invariant]
  Paper reference: [Circle STARKs / Cairo / STWO Whitepaper section]
  Code location: [file:line]
  Confidence: [percentage]
  Reason: [why escalation is required]
```

### Security escalation format

```text
SECURITY-ESCALATION:
  File: [path]
  Attack surface: [what could be exploited]
  Mitigation: [existing protection]
  Recommendation: [required next action]
```

## Sensitive Path Ownership

### Math Reviewer required

- `crates/circuit_air/src/components/`
- `crates/circuit_air/src/circuit_eval_components/`
- `crates/circuit_prover/src/prover.rs`
- `crates/circuit_prover/src/witness/`
- `crates/stark_verifier/src/fri.rs`
- `crates/stark_verifier/src/oods.rs`
- `crates/stark_verifier/src/merkle.rs`
- `crates/stark_verifier/src/statement.rs`
- `crates/cairo_air/src/verify.rs`

### Implementer autonomous scope

- `crates/stark_verifier_examples/`
- `scripts/`
- Documentation and comments
- Test additions (never silent test removals)

### Perf Specialist scope (with Math Reviewer verification)

- `crates/circuit_prover/`
- `crates/circuits/src/simd.rs`
- `crates/circuits/src/blake.rs`

## Agent Execution Checklist

Before editing:

1. Classify change type (standard / soundness-critical / performance)
2. Confirm touched files are in allowed scope for the acting role
3. Define validation commands to run after edits

Before merge:

1. Ensure the final diff is minimal and reviewable
2. Run relevant tests and checks
3. Document residual risk and assumptions in PR notes

## Skill Requirements by Role

The [stwo](https://github.com/starkware-libs/stwo) repository contains skill files in
`.claude/skills/` that should be consulted for theory-grounded work.

| Role | Required Skills Before Acting |
|------|-------------------------------|
| Math Reviewer | `soundness-review-checklist`, `air-constraint-engineering`, `paper-implementation-divergence-log` |
| Crypto Specialist | Relevant math skill + paper section review, `air-constraint-engineering` |
| Implementer | `rust-codebase-conventions`, `testing-strategy` |
| Perf Specialist | `performance-optimization` |
| All | `paper-implementation-divergence-log` for theory-grounded changes; `debugging-zkp` for proof failures |
