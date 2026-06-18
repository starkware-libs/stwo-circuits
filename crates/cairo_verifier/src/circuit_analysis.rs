//! Analyzes the Cairo-verifier [`circuits::circuit::Circuit`].

use hashbrown::{HashMap, HashSet};
use std::ops::Deref;

use circuits::circuit::{BlakeGGate, Circuit};
use circuits::context::Var;
use circuits::ivalue::{IValue, qm31_from_u32s};
use num_traits::{One, Zero};
use stwo::core::fields::qm31::QM31;

/// Helper function to construct a QM31 value from 4 u32s.
fn qm(a: u32, b: u32, c: u32, d: u32) -> QM31 {
    qm31_from_u32s(a, b, c, d)
}

/// Simd constants for 1/2.
fn half() -> QM31 {
    qm(1 << 30, 1 << 30, 1 << 30, 1 << 30)
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
    /// Raw producer of each arithmetic output: `out -> (op, a, b)`.
    producers: HashMap<usize, (Op, usize, usize)>,
    /// Guessed (unconstrained) variables, yielded by the trivial gate `[v] = [v] + [0]`.
    guessed: HashSet<usize>,
    /// Map `out` -> `in` where `[out] = m31_to_u32([in])`.
    m31_out_to_in: HashMap<usize, usize>,
    // blake output -> index into `circuit.blake_g_gate`
    blake_of: HashMap<usize, usize>,
    // TripleXor output -> index into `circuit.triple_xor`.
    xor_of: HashMap<usize, usize>,
}

impl Deref for CircuitEx<'_> {
    type Target = Circuit;
    fn deref(&self) -> &Circuit {
        self.circuit
    }
}

/// The six inputs of a Blake G gate, in `(a, b, c, d, f0, f1)` order.
fn blake_gate_inputs(g: &BlakeGGate) -> [usize; 6] {
    [g.input_a, g.input_b, g.input_c, g.input_d, g.input_f0, g.input_f1]
}

/// Records one arithmetic gate `[out] = [a] op [b]`.
fn add_arith(c: &mut CircuitEx, op: Op, a: usize, b: usize, out: usize) {
    c.arith.push(Arith { out, op, a, b });
    c.producers.insert(out, (op, a, b));
    if op == Op::Add && out == a && b == 0 {
        // The trivial yield gate `[v] = [v] + [0]`.
        c.guessed.insert(out);
    }
}

/// Builds the analysis view from the verifier circuit's typed gates. Output gates are ignored
/// (they carry no dependency we need).
fn build_analysis(circuit: &Circuit) -> CircuitEx<'_> {
    let mut c = CircuitEx {
        circuit,
        arith: Vec::new(),
        producers: HashMap::new(),
        guessed: HashSet::new(),
        m31_out_to_in: HashMap::new(),
        blake_of: HashMap::new(),
        xor_of: HashMap::new(),
    };

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

    for g in &circuit.m31_to_u32 {
        c.m31_out_to_in.insert(g.out, g.input);
    }

    for (i, g) in circuit.blake_g_gate.iter().enumerate() {
        for o in [g.out_a, g.out_b, g.out_c, g.out_d] {
            c.blake_of.insert(o, i);
        }
    }

    for (i, g) in circuit.triple_xor.iter().enumerate() {
        c.xor_of.insert(g.out, i);
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
fn propagate_constants(c: &CircuitEx) -> HashMap<usize, QM31> {
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

// Idioms.

/// A recognized higher-level meaning of a variable, layered over the raw producers.
#[derive(Clone, Copy)]
enum Idiom {
    // X = 1/Y, from the `div(1, Y)` idiom.
    Inverse { y: usize },
    // A bit of `root`'s binary decomposition.
    Bit { root: usize },
}

/// Returns every bit-constrained variable: variable with a constraint forcing `v == v²`
/// (hence v in {0, 1}). One constraint of the form `[s] = [v] x [v]` and another constraint which
/// is either:
///   (a) a direct equality `[v] = [s]`; or
///   (b) the difference `[d] = [v] - [s]`, pointwise-masked by a constant, constrained to zero.
///
/// As some of the coordinates of the constant in (b) may be zero, the check here is not full.
/// This is acceptable for this test.
fn find_bits(c: &CircuitEx, const_values: &HashMap<usize, QM31>) -> HashSet<usize> {
    // A map `v` -> `s` from `[s] = [v] x [v]`.
    let mut square_of: HashMap<usize, usize> = HashMap::new();
    // A map `(a, b)` -> `[a] - [b]`.
    let mut sub_out: HashMap<(usize, usize), usize> = HashMap::new();
    // A map `var` -> outputs of `var x const`.
    let mut masked: HashMap<usize, Vec<usize>> = HashMap::new();

    for arith in &c.arith {
        match arith.op {
            Op::Pointwise if arith.a == arith.b => {
                square_of.entry(arith.a).or_insert(arith.out);
            }
            Op::Pointwise if const_values.get(&arith.b).is_some_and(|c| !c.is_zero()) => {
                masked.entry(arith.a).or_default().push(arith.out);
            }
            Op::Sub => {
                sub_out.entry((arith.a, arith.b)).or_insert(arith.out);
            }
            _ => {}
        }
    }

    let eq_set: HashSet<(usize, usize)> = c.eq.iter().map(|e| (e.in0, e.in1)).collect();
    // Variables constrained to equal the zero constant.
    let eq_zero: HashSet<usize> =
        c.eq.iter().filter_map(|e| (e.in1 == 0).then_some(e.in0)).collect();
    // Whether `[d]` is constrained to zero through a pointwise mask `[d] x [const]`.
    let zero_checked =
        |d: usize| masked.get(&d).is_some_and(|outs| outs.iter().any(|m| eq_zero.contains(m)));

    // Keep the squared values whose `v == v²` is actually constrained.
    square_of
        .iter()
        .filter_map(|(&v, &s)| {
            let constrained =
                eq_set.contains(&(v, s)) || sub_out.get(&(v, s)).is_some_and(|&d| zero_checked(d));
            constrained.then_some(v)
        })
        .collect()
}

/// Detects bit decompositions. A value is split lsb-first by repeated halving:
/// `[rnext] = ([r] - [b]) x half` with `b` a guessed bit. Fills `idiom` with `Bit`.
fn detect_bits(
    c: &CircuitEx,
    const_values: &HashMap<usize, QM31>,
    idiom: &mut HashMap<usize, Idiom>,
) {
    let bits = find_bits(c, const_values);

    // chain: r -> (b, (r - b) x (1/2)).
    // The variable representing (r - b) x (1/2) is added to `is_next`.
    let mut chain: HashMap<usize, (usize, usize)> = HashMap::new();
    let mut is_next: HashSet<usize> = HashSet::new();
    for g in &c.arith {
        if !(g.op == Op::Pointwise && const_values.get(&g.b) == Some(&half())) {
            continue;
        }
        let diff = g.a;
        if let Some(&(Op::Sub, r, b)) = c.producers.get(&diff)
            && c.guessed.contains(&b)
        {
            chain.insert(r, (b, g.out));
            is_next.insert(g.out);
        }
    }

    for &root in chain.keys() {
        // If it's not the root of a chain, skip it.
        if is_next.contains(&root) {
            continue;
        }
        // Every bit on the chain, and the final running value, must be bit-constrained.
        let mut cur = root;
        while let Some(&(b, next)) = chain.get(&cur) {
            assert!(bits.contains(&b), "chain bit [{b}] (root [{root}]) is not a bit");
            idiom.insert(b, Idiom::Bit { root });
            cur = next;
        }
        assert!(
            bits.contains(&cur),
            "top of bit-decomposition chain (root [{root}]) [{cur}] is not a bit"
        );
        idiom.insert(cur, Idiom::Bit { root });
    }
}

/// Detects the `div(1, Y)` idiom: `[1] = [X] * [Y]` with `X` guessed, so `X = 1/Y`. Fills `idiom`
/// with `Inverse`.
fn detect_inverses(
    c: &CircuitEx,
    const_values: &HashMap<usize, QM31>,
    idiom: &mut HashMap<usize, Idiom>,
) {
    for g in &c.arith {
        if g.op != Op::Mul || const_values.get(&g.out) != Some(&QM31::one()) {
            continue;
        }
        let (output, input) = if c.guessed.contains(&g.b) {
            (g.b, g.a)
        } else if c.guessed.contains(&g.a) {
            (g.a, g.b)
        } else {
            continue;
        };
        idiom.entry(output).or_insert(Idiom::Inverse { y: input });
    }
}

/// Variables reachable from `root` through `+`/`-`/`*`, skipping the indeterminate α and constants.
/// With `add_intermediates`, every reachable variable is returned; otherwise returns only the
/// leaves.
fn add_mul_cone(
    c: &CircuitEx,
    const_values: &HashMap<usize, QM31>,
    root: usize,
    add_intermediates: bool,
    int_alpha: usize,
) -> HashSet<usize> {
    let mut res = HashSet::new();
    let mut seen = HashSet::new();
    let mut stack = vec![root];
    while let Some(v) = stack.pop() {
        if v == int_alpha || const_values.contains_key(&v) {
            continue;
        }
        let is_first_visit = seen.insert(v);
        if !is_first_visit {
            continue;
        }
        // `v != a` keeps a guessed leaf `[v] = [v] + [0]` from expanding into itself.
        if let Some(&(Op::Add | Op::Sub | Op::Mul, a, b)) = c.producers.get(&v)
            && v != a
        {
            stack.push(a);
            stack.push(b);
            if add_intermediates {
                res.insert(v);
            }
        } else {
            // A leaf.
            res.insert(v);
        }
    }
    res
}

// Groundedness.

/// Computes groundedness against a challenge. A value is considered grounded to a challenge if it
/// is in the challenge's dependency closure or it is a deterministic function of such values.
/// The challenge is not considered grounded to itself.
struct Ground<'a> {
    c: &'a CircuitEx<'a>,
    const_values: &'a HashMap<usize, QM31>,
    idiom: &'a HashMap<usize, Idiom>,
}

impl Ground<'_> {
    /// Returns variables that `root` depends on, including `root`.
    /// Note that it's possible that only a subset of the variables will be returned.
    fn closure(&self, root: usize) -> HashSet<usize> {
        let mut seen = HashSet::new();
        let mut stack = vec![root];
        while let Some(v) = stack.pop() {
            let is_first_visit = seen.insert(v);
            if is_first_visit {
                stack.extend(self.gate_inputs(v).unwrap_or_default());
            }
        }
        seen
    }

    /// Returns the input variables of the gate that produces `v`, or `None` if not known.
    /// These inputs are considered grounded to `v`.
    fn gate_inputs(&self, v: usize) -> Option<Vec<usize>> {
        if let Some(&(_, a, b)) = self.c.producers.get(&v) {
            Some(vec![a, b])
        } else if let Some(&i) = self.c.m31_out_to_in.get(&v) {
            Some(vec![i])
        } else if let Some(&bi) = self.c.blake_of.get(&v) {
            Some(blake_gate_inputs(&self.c.blake_g_gate[bi]).to_vec())
        } else if let Some(&xi) = self.c.xor_of.get(&v) {
            let g = &self.c.triple_xor[xi];
            Some(vec![g.input_a, g.input_b, g.input_c])
        } else {
            None
        }
    }

    /// Returns a list of variables that `v` deterministically depends on, or `None` on failure.
    /// If every one of them is grounded, so is `v`.
    ///
    /// This differs from [`Self::gate_inputs`] for idioms. For example, a bit is grounded once its
    /// root is, yet if the bit is grounded it doesn't follow that the root is.
    /// A guess is considered a leaf.
    fn grounding_inputs(&self, v: usize) -> Option<Vec<usize>> {
        match self.idiom.get(&v) {
            Some(Idiom::Bit { root }) => Some(vec![*root]),
            Some(Idiom::Inverse { y }) => Some(vec![*y]),
            None => {
                if self.c.guessed.contains(&v) {
                    None
                } else {
                    self.gate_inputs(v)
                }
            }
        }
    }

    /// A memo for [`Self::grounded`] for a single challenge: its dependency closure and
    /// the constants are grounded (`true`); the challenge itself is not (`false`).
    fn init_memo(&self, challenge: usize) -> HashMap<usize, bool> {
        let closure = self.closure(challenge);
        let mut memo: HashMap<usize, bool> = closure.iter().map(|&v| (v, true)).collect();
        memo.extend(self.const_values.keys().map(|&v| (v, true)));
        memo.insert(challenge, false);
        memo
    }

    /// Whether `root` is a deterministic function of the challenge's dependencies. `memo` must be
    /// seeded by [`Self::init_memo`]; it caches results for that challenge across calls.
    fn grounded(&self, root: usize, memo: &mut HashMap<usize, bool>) -> bool {
        let mut stack = vec![root];
        while let Some(&v) = stack.last() {
            if memo.contains_key(&v) {
                stack.pop();
                continue;
            }
            let Some(inputs) = self.grounding_inputs(v) else {
                memo.insert(v, false);
                stack.pop();
                continue;
            };
            // Check if there are children that were not resolved yet.
            let pending: Vec<usize> =
                inputs.iter().copied().filter(|i| !memo.contains_key(i)).collect();
            if pending.is_empty() {
                // All children were resolved. Update the status of `v`.
                let grounded = inputs.iter().all(|i| memo[i]);
                memo.insert(v, grounded);
                stack.pop();
            } else {
                // Keep v in the stack. We will return to it once all its children were resolved.
                stack.extend(pending);
            }
        }
        memo[&root]
    }

    /// Whether every value in `vars` is grounded to the challenge (and there is at least one).
    fn all_grounded(&self, vars: &[usize], memo: &mut HashMap<usize, bool>) -> bool {
        !vars.is_empty() && vars.iter().all(|&v| self.grounded(v, memo))
    }
}

// Sum tree and classification.

/// Expands the add/sub tree at `root` into its leaf summands (left to right). A leaf is any
/// variable not produced by a genuine `+`/`-` gate; zero-valued leaves are dropped.
fn find_summands(c: &CircuitEx, const_values: &HashMap<usize, QM31>, root: usize) -> Vec<usize> {
    let mut summands = Vec::new();
    let mut stack = vec![root];
    while let Some(v) = stack.pop() {
        if let Some(&(Op::Add | Op::Sub, a, b)) = c.producers.get(&v)
            && v != a
            && v != b
        {
            // Push `b` first, so the left child expands first.
            stack.push(b);
            stack.push(a);
        } else if const_values.get(&v) != Some(&QM31::zero()) {
            // A non-zero leaf.
            summands.push(v);
        }
    }
    summands
}

/// Validates a logup summand `s`, asserting the expected groundedness of its values.
///
/// Arguments:
/// - `composition_vars`: the `composition_eval` cone.
/// - `int_z`: the `interaction_z` variable index.
/// - `int_z_memo`: Memo for [`Ground::grounded`] for `int_z`.
/// - `composition_coef_memo`: Memo for [`Ground::grounded`] for `composition_coef`.
fn validate_logup_summand(
    ground: &Ground,
    int_z: usize,
    int_z_memo: &mut HashMap<usize, bool>,
    composition_coef_memo: &mut HashMap<usize, bool>,
    composition_vars: &HashSet<usize>,
    s: usize,
) {
    let c = ground.c;

    // Case I: s = 1 / (X - int_z).
    if let Some(&Idiom::Inverse { y: denom }) = ground.idiom.get(&s)
        && let Some(&(Op::Sub, x, y)) = c.producers.get(&denom)
    {
        assert!(y == int_z, "Summand [{s}] is 1/(X-Y) with Y=[{y}], expected int_z=[{int_z}].");
        // Check that X is grounded to int_z.
        assert!(ground.all_grounded(&[x], int_z_memo), "Summand [{s}] = 1 / ([{x}] - [{y}]) is not grounded.");
        return;
    }

    // Case II: s is a guess.
    if c.guessed.contains(&s) {
        // Make sure s participates in the constraints.
        assert!(composition_vars.contains(&s));
        // Make sure s is grounded to composition_coef.
        assert!(ground.all_grounded(&[s], composition_coef_memo));
        return;
    }

    panic!("Summand [{s}] is not a valid logup summand.");
}

/// Analyzes the circuit and writes the classified logup-sum summands to `summands.txt`.
pub fn analyze(circuit: &Circuit, debug_info: &HashMap<String, Var>) {
    let c = build_analysis(circuit);
    let const_values = propagate_constants(&c);

    // Recognize idioms layered over the raw producers.
    let mut idiom: HashMap<usize, Idiom> = HashMap::new();
    detect_bits(&c, &const_values, &mut idiom);
    detect_inverses(&c, &const_values, &mut idiom);

    let ground = Ground { c: &c, const_values: &const_values, idiom: &idiom };

    // Seed each challenge's `grounded` cache with its dependency closure.
    let int_z = debug_info["interaction_z"].idx;
    let mut int_z_memo = ground.init_memo(int_z);

    let composition_coef = debug_info["composition_polynomial_coeff"].idx;
    let mut composition_coef_memo = ground.init_memo(composition_coef);

    let logup_sum = debug_info["logup_sum"].idx;
    let summands = find_summands(&c, &const_values, logup_sum);

    let composition_eval = debug_info["composition_eval"].idx;
    let int_alpha = debug_info["interaction_alpha"].idx;
    let composition_vars = add_mul_cone(&c, &const_values, composition_eval, true, int_alpha);

    for &s in &summands {
        validate_logup_summand(
            &ground,
            int_z,
            &mut int_z_memo,
            &mut composition_coef_memo,
            &composition_vars,
            s,
        );
    }
}
