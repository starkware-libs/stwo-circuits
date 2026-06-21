//! Analyzes the Cairo-verifier [`circuits::circuit::Circuit`].

use circuits::context::Var;
use hashbrown::HashMap;
use std::ops::Deref;

use circuits::circuit::Circuit;
use circuits::ivalue::{IValue, qm31_from_u32s};
use num_traits::{One, Zero};
use stwo::core::fields::qm31::QM31;

/// Helper function to construct a QM31 value from 4 u32s.
fn qm(a: u32, b: u32, c: u32, d: u32) -> QM31 {
    qm31_from_u32s(a, b, c, d)
}

#[derive(Clone, Copy, PartialEq)]
enum Op {
    Add,
    Sub,
    Mul,
    Pointwise,
}

/// A binary arithmetic gate `[out] = [a] op [b]`.
struct Arith {
    out: usize,
    op: Op,
    a: usize,
    b: usize,
}

/// The verifier circuit plus the lookup tables the analysis needs.
/// Derefs to the wrapped [`Circuit`], so its gate vectors (`add`, `eq`, `blake_g_gate`, ...)
/// can be accessed directly.
struct CircuitEx<'a> {
    circuit: &'a Circuit,
    /// All arithmetic gates (add/sub/mul/pointwise).
    arith: Vec<Arith>,
}

impl Deref for CircuitEx<'_> {
    type Target = Circuit;
    fn deref(&self) -> &Circuit {
        self.circuit
    }
}

/// Records one arithmetic gate `[out] = [a] op [b]`.
fn add_arith(c: &mut CircuitEx<'_>, op: Op, a: usize, b: usize, out: usize) {
    c.arith.push(Arith { out, op, a, b });
}

/// Builds the analysis view from the verifier circuit's typed gates. Output gates are ignored
/// (they carry no dependency we need).
fn build_analysis(circuit: &Circuit) -> CircuitEx<'_> {
    let mut c = CircuitEx { circuit, arith: Vec::new() };

    for g in &circuit.add {
        add_arith(&mut c, Op::Add, g.in0, g.in1, g.out);
    }
    for g in &circuit.sub {
        add_arith(&mut c, Op::Sub, g.in0, g.in1, g.out);
    }
    for g in &circuit.mul {
        add_arith(&mut c, Op::Mul, g.in0, g.in1, g.out);
    }
    for g in &circuit.pointwise_mul {
        add_arith(&mut c, Op::Pointwise, g.in0, g.in1, g.out);
    }

    c
}

// Constant propagation.

fn apply(op: Op, a: QM31, b: QM31) -> QM31 {
    match op {
        Op::Add => a + b,
        Op::Sub => a - b,
        Op::Mul => a * b,
        Op::Pointwise => QM31::pointwise_mul(a, b),
    }
}

/// Propagates the seed constants (`[0]=0`, `[1]=1`, `[2]=u`) through arithmetic and equality
/// gates: an arithmetic output is constant once both inputs are; an equality propagates a known
/// side to the other. Returns each constant variable's QM31 value.
fn propagate_constants(c: &CircuitEx<'_>) -> HashMap<usize, QM31> {
    // input variable -> arithmetic gates consuming it.
    // For example, if the first arithmetic gate (idx=0) is [6] = [4] + [5],
    // then consumers[4] = consumers[5] = [0].
    let mut consumers: HashMap<usize, Vec<usize>> = HashMap::new();
    for (idx, arith) in c.arith.iter().enumerate() {
        consumers.entry(arith.a).or_default().push(idx);
        if arith.b != arith.a {
            consumers.entry(arith.b).or_default().push(idx);
        }
    }

    // Equality adjacency: for every equality gate, each side points to the other.
    let mut eq_adj: HashMap<usize, Vec<usize>> = HashMap::new();
    for eq in &c.eq {
        eq_adj.entry(eq.in0).or_default().push(eq.in1);
        eq_adj.entry(eq.in1).or_default().push(eq.in0);
    }

    // Initialize the values of the seed constants.
    let mut values = HashMap::from([(0, QM31::zero()), (1, QM31::one()), (2, qm(0, 0, 1, 0))]);
    let mut work = vec![0, 1, 2];
    while let Some(var) = work.pop() {
        let val = values[&var];
        // Go over all the variables that are equal to the current variable, register their value
        // and add them to `work`.
        for &other_var in eq_adj.get(&var).into_iter().flatten() {
            let insert_res = values.insert(other_var, val);
            if insert_res.is_none() {
                work.push(other_var);
            }
        }

        // Go over all the consumers of the current variable, and compute the value of the output.
        for &gate_idx in consumers.get(&var).into_iter().flatten() {
            let arith = &c.arith[gate_idx];
            // If the output is already a known constant, continue.
            if values.contains_key(&arith.out) {
                continue;
            }

            // If both inputs are constants, registers the output.
            if let (Some(&va), Some(&vb)) = (values.get(&arith.a), values.get(&arith.b)) {
                values.insert(arith.out, apply(arith.op, va, vb));
                work.push(arith.out);
            }
        }
    }
    values
}

/// Analyzes the circuit and writes the classified logup-sum summands to `summands.txt`.
pub fn analyze(circuit: &Circuit, _debug_info: &HashMap<String, Var>) {
    let c = build_analysis(circuit);
    let _const_values = propagate_constants(&c);

    // TODO(lior): complete the test.
}
