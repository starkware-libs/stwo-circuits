# STWO Circuits Agent Architecture

## Roles

| Role | Model Tier | Responsibility | Hard Boundaries |
|------|-----------|----------------|-----------------|
| Orchestrator | Frontier | Task decomposition, delegation, integration | NEVER writes proof-system code directly |
| Math Reviewer | Frontier | Soundness/security review of crypto code | NEVER implements — reviews and escalates |
| Implementer | Frontier | Tests, docs, refactoring, non-crypto code | NEVER touches [SOUNDNESS-CRITICAL] files |
| Crypto Specialist | Frontier | Changes to proof system code | Only operates with Math Reviewer sign-off |
| Perf Specialist | Frontier | Benchmarking, profiling, optimization | NEVER changes algorithmic correctness |

## Workflow

### Standard Change (non-crypto)

```
User Request
  → Orchestrator: classify task
  → Implementer: execute (tests, docs, refactoring, infra)
  → CI verification
```

### Soundness-Critical Change

```
User Request
  → Orchestrator: classify as soundness-critical
  → Math Reviewer: identify paper reference, assess invariants
  → Crypto Specialist: implement change (with Math Reviewer guidance)
  → Math Reviewer: review change
  → Human: final approval
  → CI verification
```

### Performance Change

```
User Request
  → Orchestrator: classify as performance
  → Perf Specialist: benchmark baseline, implement optimization
  → Math Reviewer: verify optimization preserves semantics
  → CI verification
```

## Escalation Protocol

Escalate to human IMMEDIATELY when:

1. Any undocumented paper-implementation divergence is discovered
2. A soundness-critical component has zero test coverage for the modified path
3. A proposed change cannot be grounded in a paper definition
4. Any `unsafe` block is found in a soundness-critical path without documented justification
5. Confidence in mathematical correctness of any change drops

### Escalation Format

```
SOUNDNESS-ESCALATION:
  File: [path]
  Change: [what is proposed]
  Invariant at risk: [which mathematical invariant]
  Paper reference: [Circle STARKs paper / Cairo paper / STWO Whitepaper anchor]
  Code location: [file:line]
  Confidence: [percentage]
  Reason: [why escalation is needed]
```

For security (non-soundness) issues:
```
SECURITY-ESCALATION:
  File: [path]
  Attack surface: [what could be exploited]
  Mitigation: [existing protection]
  Recommendation: [what should be done]
```

## File Ownership

### Math Reviewer Must Review

- `crates/circuit_air/src/components/` — Circuit constraint definitions
- `crates/circuit_air/src/circuit_eval_components/` — Constraint evaluators
- `crates/circuit_prover/src/prover.rs` — Core proving logic
- `crates/circuit_prover/src/witness/` — Witness and interaction trace generation
- `crates/stark_verifier/src/fri.rs` — FRI protocol verification
- `crates/stark_verifier/src/oods.rs` — Out-of-domain sampling
- `crates/stark_verifier/src/merkle.rs` — Merkle commitment verification
- `crates/stark_verifier/src/statement.rs` — Statement verification trait
- `crates/cairo_air/src/verify.rs` — Cairo AIR verification entry point

### Implementer Can Modify Autonomously

- `crates/stark_verifier_examples/` — Example verifiers and test utilities
- `scripts/` — Build and lint scripts
- Documentation and comments
- Test additions (never removals)

### Perf Specialist Can Modify (with Math Reviewer for correctness)

- `crates/circuit_prover/` — Proof generation performance
- `crates/circuits/src/simd.rs` — SIMD vectorization
- `crates/circuits/src/blake.rs` — Blake2s hashing performance

## Skill Requirements by Role

The [stwo](https://github.com/starkware-libs/stwo) repo contains detailed skill
files in `.claude/skills/` covering the core proof system. Agents working on
stwo-circuits should reference those skills when touching theory-grounded code.

| Role | Required Skills Before Acting |
|------|-------------------------------|
| Math Reviewer | soundness-review-checklist, air-constraint-engineering, paper-implementation-divergence-log |
| Crypto Specialist | Relevant math skill + paper section read, air-constraint-engineering |
| Implementer | rust-codebase-conventions, testing-strategy |
| Perf Specialist | performance-optimization |
| All | paper-implementation-divergence-log (when touching theory-grounded code), debugging-zkp (when proofs fail) |
