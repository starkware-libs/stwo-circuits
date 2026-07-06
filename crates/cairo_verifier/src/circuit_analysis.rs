//! Analyzes the Cairo-verifier [`circuits::circuit::Circuit`].

use circuits::context::Var;
use hashbrown::{HashMap, HashSet};
use std::ops::Deref;

use circuits::circuit::Circuit;
use circuits::ivalue::{IValue, qm31_from_u32s};
use num_traits::{One, Zero};
use stwo::core::fields::qm31::QM31;

/// Helper function to construct a QM31 value from 4 u32s.
fn qm(a: u32, b: u32, c: u32, d: u32) -> QM31 {
    qm31_from_u32s(a, b, c, d)
}

/// Simd constants for 1/2.
fn simd_half() -> QM31 {
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
    let mut const_values =
        HashMap::from([(0, QM31::zero()), (1, QM31::one()), (2, qm(0, 0, 1, 0))]);
    let mut work = vec![0, 1, 2];
    while let Some(var) = work.pop() {
        let val = const_values[&var];
        // Go over all the variables that are equal to the current variable, register their value
        // and add them to `work`.
        for &other_var in eq_adj.get(&var).into_iter().flatten() {
            register_constant(&mut const_values, &mut work, other_var, val);
        }

        // Go over all the consumers of the current variable, and compute the value of the output.
        for &gate_idx in consumers.get(&var).into_iter().flatten() {
            let arith = &c.arith[gate_idx];

            // If both inputs are constants, registers the output.
            let (Some(&va), Some(&vb)) = (const_values.get(&arith.a), const_values.get(&arith.b))
            else {
                continue;
            };

            register_constant(&mut const_values, &mut work, arith.out, apply(arith.op, va, vb));
        }
    }
    const_values
}

/// Helper function for `propagate_constants`.
/// Registers the value `val` for `var` in `const_values`.
/// If `var` was not seen before, it is added to `work` for further propagation.
/// If it was seen, asserts that the new value matches the old one.
fn register_constant(
    const_values: &mut HashMap<usize, QM31>,
    work: &mut Vec<usize>,
    var: usize,
    val: QM31,
) {
    let insert_res = const_values.insert(var, val);
    if let Some(old_val) = insert_res {
        assert!(old_val == val, "Value mismatch for {var} during constant propagation");
    } else {
        work.push(var);
    }
}

// Idioms.

/// A recognized higher-level meaning of a variable, layered over the raw producers.
#[derive(Clone, Copy)]
enum Idiom {
    // X = 1/Y, from the `div(1, Y)` idiom.
    Inverse { y: usize },
    // One of the bits in `value`'s binary decomposition.
    Bit { value: usize },
}

/// Returns every bit-constrained variable: variable with a constraint forcing `v == v²`
/// (hence v in {0, 1}). One constraint of the form `[s] = [v] x [v]` and another constraint which
/// is either:
///   (a) a direct equality `[v] = [s]`; or
///   (b) the difference `[d] = [v] - [s]`, pointwise-masked by a constant, constrained to zero.
///
/// As some of the coordinates of the constant in (b) may be zero, the check here is not full.
/// This is acceptable for this test.
fn find_bits(c: &CircuitEx<'_>, const_values: &HashMap<usize, QM31>) -> HashSet<usize> {
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
    c: &CircuitEx<'_>,
    const_values: &HashMap<usize, QM31>,
    idiom: &mut HashMap<usize, Idiom>,
) {
    let bits = find_bits(c, const_values);

    // chain: r -> (b, (r - b) x (1/2)).
    // The variable representing (r - b) x (1/2) is added to `is_next`.
    let mut chain: HashMap<usize, (usize, usize)> = HashMap::new();
    let mut is_next: HashSet<usize> = HashSet::new();
    for g in &c.arith {
        if !(g.op == Op::Pointwise && const_values.get(&g.b) == Some(&simd_half())) {
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

    for &value in chain.keys() {
        // If it's not the root of a chain, skip it.
        if is_next.contains(&value) {
            continue;
        }
        // Every bit on the chain, and the final running value, must be bit-constrained.
        let mut cur = value;
        while let Some(&(b, next)) = chain.get(&cur) {
            assert!(bits.contains(&b), "chain bit [{b}] (value [{value}]) is not a bit");
            idiom.insert(b, Idiom::Bit { value });
            cur = next;
        }
        assert!(
            bits.contains(&cur),
            "top of bit-decomposition chain (value [{value}]) [{cur}] is not a bit"
        );
        idiom.insert(cur, Idiom::Bit { value });
    }
}

/// Detects the `div(1, Y)` idiom: `[1] = [X] * [Y]` with `X` guessed, so `X = 1/Y`. Fills `idiom`
/// with `Inverse`.
fn detect_inverses(
    c: &CircuitEx<'_>,
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

// Groundedness.

/// Computes groundedness against a challenge. A value is considered grounded to a challenge if it
/// is in the challenge's dependency closure or it is a deterministic function of such values.
/// The challenge is not considered grounded to itself.
struct Ground<'a> {
    c: &'a CircuitEx<'a>,
    const_values: &'a HashMap<usize, QM31>,
    idiom: &'a HashMap<usize, Idiom>,
}

/// Cache for [`Ground::grounded`] for a single challenge.
struct GroundingCache {
    data: HashMap<usize, bool>,
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
        // TODO(lior): Add gates: blake_g, xor, m31_to_u32.
        if let Some(&(_, a, b)) = self.c.producers.get(&v) { Some(vec![a, b]) } else { None }
    }

    /// Returns a list of variables that `v` deterministically depends on, or `None` on failure.
    /// If every one of them is grounded, so is `v`.
    ///
    /// This differs from [`Self::gate_inputs`] for idioms. For example, a bit is grounded once its
    /// base value is, yet if the bit is grounded it doesn't follow that the value is.
    /// A guess is considered a leaf.
    fn grounding_inputs(&self, v: usize) -> Option<Vec<usize>> {
        match self.idiom.get(&v) {
            Some(Idiom::Bit { value }) => Some(vec![*value]),
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

    /// A cache for [`Self::grounded`] for a single challenge: its dependency closure and
    /// the constants are grounded (`true`); the challenge itself is not (`false`).
    fn init_grounding_cache(&self, challenge: usize) -> GroundingCache {
        let closure = self.closure(challenge);
        let mut cache: HashMap<usize, bool> = closure.iter().map(|&v| (v, true)).collect();
        cache.extend(self.const_values.keys().map(|&v| (v, true)));
        cache.insert(challenge, false);
        GroundingCache { data: cache }
    }

    /// Whether `root` is a deterministic function of the challenge's dependencies. `cache` must be
    /// seeded by [`Self::init_grounding_cache`]; it caches results for that challenge across calls.
    #[allow(unused)]
    fn grounded(&self, root: usize, cache: &mut GroundingCache) -> bool {
        let mut stack = vec![root];
        let cache = &mut cache.data;
        while let Some(v) = stack.pop() {
            if cache.contains_key(&v) {
                continue;
            }
            let Some(inputs) = self.grounding_inputs(v) else {
                cache.insert(v, false);
                continue;
            };
            // Check if there are children that were not resolved yet.
            let pending: Vec<usize> =
                inputs.iter().copied().filter(|i| !cache.contains_key(i)).collect();
            if pending.is_empty() {
                // All children were resolved. Update the status of `v`.
                let grounded = inputs.iter().all(|i| cache[i]);
                cache.insert(v, grounded);
            } else {
                // Re-add v to the stack. We will return to it once all its children were resolved.
                stack.push(v);
                stack.extend(pending);
            }
        }
        cache[&root]
    }
}

// Sum tree and classification.

/// Expands the add/sub tree at `root` into its leaf summands (left to right). A leaf is any
/// variable not produced by a genuine `+`/`-` gate; zero-valued leaves are dropped.
fn find_summands(
    c: &CircuitEx<'_>,
    const_values: &HashMap<usize, QM31>,
    root: usize,
) -> Vec<usize> {
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
    let mut _int_z_cache = ground.init_grounding_cache(int_z);

    let composition_coef = debug_info["composition_polynomial_coeff"].idx;
    let mut _composition_coef_cache = ground.init_grounding_cache(composition_coef);

    let logup_sum = debug_info["logup_sum"].idx;
    let summands = find_summands(&c, &const_values, logup_sum);

    for _ in summands {
        // TODO(lior): Validate each summand.
    }
}
