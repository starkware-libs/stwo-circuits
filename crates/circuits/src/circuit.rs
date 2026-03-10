use std::collections::HashMap;

use itertools::chain;
use stwo::core::fields::qm31::QM31;

use crate::blake::blake_qm31;
use crate::ivalue::IValue;

#[cfg(test)]
#[path = "circuit_test.rs"]
pub mod test;

fn check_eq(a: QM31, b: QM31) -> Result<(), String> {
    if a != b {
        return Err(format!("{a} != {b}"));
    }
    Ok(())
}

pub trait Gate: std::fmt::Debug {
    fn check(&self, values: &[QM31]) -> Result<(), String>;

    /// Returns the variables that are "used" by the gate (in the context of lookup terms).
    fn uses(&self) -> Vec<usize>;

    /// Returns the variables that are "yielded" by the gate (in the context of lookup terms).
    fn yields(&self) -> Vec<usize>;
}

/// Represents an addition gate in the circuit: `[in0] + [in1] = [out]`.
#[derive(PartialEq, Eq)]
pub struct Add {
    pub in0: usize,
    pub in1: usize,
    pub out: usize,
}
impl Gate for Add {
    fn check(&self, values: &[QM31]) -> Result<(), String> {
        check_eq(values[self.in0] + values[self.in1], values[self.out])
    }

    fn uses(&self) -> Vec<usize> {
        vec![self.in0, self.in1]
    }

    fn yields(&self) -> Vec<usize> {
        vec![self.out]
    }
}

impl std::fmt::Debug for Add {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "[{}] = [{}] + [{}]", self.out, self.in0, self.in1)
    }
}

/// Represents a subtraction gate in the circuit: `[in0] - [in1] = [out]`.
#[derive(PartialEq, Eq)]
pub struct Sub {
    pub in0: usize,
    pub in1: usize,
    pub out: usize,
}
impl Gate for Sub {
    fn check(&self, values: &[QM31]) -> Result<(), String> {
        check_eq(values[self.in0] - values[self.in1], values[self.out])
    }

    fn uses(&self) -> Vec<usize> {
        vec![self.in0, self.in1]
    }

    fn yields(&self) -> Vec<usize> {
        vec![self.out]
    }
}

impl std::fmt::Debug for Sub {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "[{}] = [{}] - [{}]", self.out, self.in0, self.in1)
    }
}

/// Represents a multiplication gate in the circuit: `[in0] * [in1] = [out]`.
#[derive(PartialEq, Eq)]
pub struct Mul {
    pub in0: usize,
    pub in1: usize,
    pub out: usize,
}
impl Gate for Mul {
    fn check(&self, values: &[QM31]) -> Result<(), String> {
        check_eq(values[self.in0] * values[self.in1], values[self.out])
    }

    fn uses(&self) -> Vec<usize> {
        vec![self.in0, self.in1]
    }

    fn yields(&self) -> Vec<usize> {
        vec![self.out]
    }
}

impl std::fmt::Debug for Mul {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "[{}] = [{}] * [{}]", self.out, self.in0, self.in1)
    }
}

/// Represents a pointwise multiplication gate in the circuit.
/// See [IValue::pointwise_mul] for more details.
#[derive(PartialEq, Eq)]
pub struct PointwiseMul {
    pub in0: usize,
    pub in1: usize,
    pub out: usize,
}
impl Gate for PointwiseMul {
    fn check(&self, values: &[QM31]) -> Result<(), String> {
        check_eq(QM31::pointwise_mul(values[self.in0], values[self.in1]), values[self.out])
    }

    fn uses(&self) -> Vec<usize> {
        vec![self.in0, self.in1]
    }

    fn yields(&self) -> Vec<usize> {
        vec![self.out]
    }
}

impl std::fmt::Debug for PointwiseMul {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "[{}] = [{}] x [{}]", self.out, self.in0, self.in1)
    }
}

/// Represents an equality gate in the circuit: `[in0] = [in1]`.
#[derive(PartialEq, Eq)]
pub struct Eq {
    pub in0: usize,
    pub in1: usize,
}
impl Gate for Eq {
    fn check(&self, values: &[QM31]) -> Result<(), String> {
        check_eq(values[self.in0], values[self.in1])
    }

    fn uses(&self) -> Vec<usize> {
        vec![self.in0, self.in1]
    }

    fn yields(&self) -> Vec<usize> {
        vec![]
    }
}

impl std::fmt::Debug for Eq {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "[{}] = [{}]", self.in0, self.in1)
    }
}

/// Represents a blake hash gate in the circuit: `([out0], [out1]) = blake([input]; n_bytes)`.
#[derive(PartialEq, Eq)]
pub struct Blake {
    pub input: Vec<[usize; 4]>,
    pub n_bytes: usize,
    pub out0: usize,
    pub out1: usize,
}
impl Gate for Blake {
    fn check(&self, values: &[QM31]) -> Result<(), String> {
        let input = self.input.iter().flatten().map(|idx| values[*idx]).collect::<Vec<_>>();
        let n_effective_vars = self.n_bytes.div_ceil(16);
        let main_part = &input[..n_effective_vars];
        let remaining_part = &input[n_effective_vars..];

        let expected_output = blake_qm31(main_part, self.n_bytes);
        check_eq(values[self.out0], expected_output.0)?;
        check_eq(values[self.out1], expected_output.1)?;

        // Sanity check: Check that the remaining input is zero.
        for val in remaining_part {
            check_eq(*val, 0.into())?;
        }

        Ok(())
    }

    fn uses(&self) -> Vec<usize> {
        self.input.iter().flatten().copied().collect()
    }

    fn yields(&self) -> Vec<usize> {
        vec![self.out0, self.out1]
    }
}

impl std::fmt::Debug for Blake {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "([{}], [{}]) = blake({:?}; {})", self.out0, self.out1, self.input, self.n_bytes)
    }
}

/// Represents a Blake2s G mixing function gate.
/// State words `a, b, c, d` are each a single QM31 wire with packed limbs: `(low_u16, high_u16,
/// 0, 0)`. Message words `m0, m1` are single M31 wires `(value, 0, 0, 0)`.
#[derive(PartialEq, Eq)]
pub struct BlakeGGate {
    pub a: usize,
    pub b: usize,
    pub c: usize,
    pub d: usize,
    pub m0: usize,
    pub m1: usize,
    pub out_a: usize,
    pub out_b: usize,
    pub out_c: usize,
    pub out_d: usize,
}
impl BlakeGGate {
    /// Reconstructs a u32 from a packed-limbs QM31: `low + high * 65536`.
    fn unpack_u32(v: QM31) -> u32 {
        v.0.0.0 + v.0.1.0 * 65536
    }

    /// Creates a packed-limbs QM31 from a u32: `(low_u16, high_u16, 0, 0)`.
    fn pack_u32(v: u32) -> QM31 {
        use crate::ivalue::qm31_from_u32s;
        qm31_from_u32s(v & 0xFFFF, v >> 16, 0, 0)
    }
}
impl Gate for BlakeGGate {
    fn check(&self, values: &[QM31]) -> Result<(), String> {
        let mut a = Self::unpack_u32(values[self.a]);
        let mut b = Self::unpack_u32(values[self.b]);
        let mut c = Self::unpack_u32(values[self.c]);
        let mut d = Self::unpack_u32(values[self.d]);
        let m0 = Self::unpack_u32(values[self.m0]);
        let m1 = Self::unpack_u32(values[self.m1]);

        a = a.wrapping_add(b).wrapping_add(m0);
        d = (d ^ a).rotate_right(16);
        c = c.wrapping_add(d);
        b = (b ^ c).rotate_right(12);
        a = a.wrapping_add(b).wrapping_add(m1);
        d = (d ^ a).rotate_right(8);
        c = c.wrapping_add(d);
        b = (b ^ c).rotate_right(7);

        check_eq(values[self.out_a], Self::pack_u32(a))?;
        check_eq(values[self.out_b], Self::pack_u32(b))?;
        check_eq(values[self.out_c], Self::pack_u32(c))?;
        check_eq(values[self.out_d], Self::pack_u32(d))?;
        Ok(())
    }

    fn uses(&self) -> Vec<usize> {
        vec![self.a, self.b, self.c, self.d, self.m0, self.m1]
    }

    fn yields(&self) -> Vec<usize> {
        vec![self.out_a, self.out_b, self.out_c, self.out_d]
    }
}

impl std::fmt::Debug for BlakeGGate {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "([{}],[{}],[{}],[{}]) = G([{}],[{}],[{}],[{}],[{}],[{}])",
            self.out_a,
            self.out_b,
            self.out_c,
            self.out_d,
            self.a,
            self.b,
            self.c,
            self.d,
            self.m0,
            self.m1
        )
    }
}

/// Represents a triple XOR gate: `out = a ^ b ^ c` (u32 XOR).
/// All operands are single QM31 wires with packed limbs `(low_u16, high_u16, 0, 0)`.
#[derive(PartialEq, Eq)]
pub struct TripleXorGate {
    pub a: usize,
    pub b: usize,
    pub c: usize,
    pub out: usize,
}
impl Gate for TripleXorGate {
    fn check(&self, values: &[QM31]) -> Result<(), String> {
        let a = BlakeGGate::unpack_u32(values[self.a]);
        let b = BlakeGGate::unpack_u32(values[self.b]);
        let c = BlakeGGate::unpack_u32(values[self.c]);
        check_eq(values[self.out], BlakeGGate::pack_u32(a ^ b ^ c))
    }

    fn uses(&self) -> Vec<usize> {
        vec![self.a, self.b, self.c]
    }

    fn yields(&self) -> Vec<usize> {
        vec![self.out]
    }
}

impl std::fmt::Debug for TripleXorGate {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "[{}] = [{}] ^ [{}] ^ [{}]", self.out, self.a, self.b, self.c)
    }
}

/// Converts an M31 value `(x, 0, 0, 0)` to a [`U32Wrapper`](crate::wrappers::U32Wrapper)
/// packed-limbs representation `(low_u16, high_u15, 0, 0)`, proving the value fits in 31 bits.
#[derive(PartialEq, Eq)]
pub struct M31ToU32Gate {
    pub input: usize,
    pub out: usize,
}
impl Gate for M31ToU32Gate {
    fn check(&self, values: &[QM31]) -> Result<(), String> {
        let input = values[self.input];
        if input.0.1.0 != 0 || input.1.0.0 != 0 || input.1.1.0 != 0 {
            return Err(format!("SplitM31: input coords 1-3 not zero, got {input}"));
        }
        let x = input.0.0.0;
        let out = values[self.out];
        if out.1.0.0 != 0 || out.1.1.0 != 0 {
            return Err(format!("SplitM31: output coords 2-3 not zero, got {out}"));
        }
        let low = out.0.0.0;
        let high = out.0.1.0;
        if low + high * 65536 != x {
            return Err(format!("SplitM31: {low} + {high} * 65536 != {x}"));
        }
        if low >= (1 << 16) {
            return Err(format!("SplitM31: low {low} >= 2^16"));
        }
        if high >= (1 << 15) {
            return Err(format!("SplitM31: high {high} >= 2^15"));
        }
        Ok(())
    }

    fn uses(&self) -> Vec<usize> {
        vec![self.input]
    }

    fn yields(&self) -> Vec<usize> {
        vec![self.out]
    }
}

impl std::fmt::Debug for M31ToU32Gate {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "[{}] = m31_to_u32([{}])", self.out, self.input)
    }
}

/// Represents a permutation gate in the circuit.
/// The gate enforces that the input values as a multi-set are equal to the output values
/// as a multi-set.
#[derive(PartialEq, Eq)]
pub struct Permutation {
    pub inputs: Vec<usize>,
    pub outputs: Vec<usize>,
}
impl Gate for Permutation {
    fn check(&self, values: &[QM31]) -> Result<(), String> {
        let mut multi_set = HashMap::new();

        for input in &self.inputs {
            *multi_set.entry(values[*input]).or_insert(0) += 1;
        }

        for output in &self.outputs {
            *multi_set.entry(values[*output]).or_insert(0) -= 1;
        }

        for count in multi_set.values() {
            if *count != 0 {
                return Err("Permutation is not valid".to_string());
            }
        }

        Ok(())
    }

    fn uses(&self) -> Vec<usize> {
        self.inputs.to_vec()
    }

    fn yields(&self) -> Vec<usize> {
        self.outputs.to_vec()
    }
}

impl std::fmt::Debug for Permutation {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "({:?}) = ({:?})", self.outputs, self.inputs)
    }
}

/// Represents a gate that receives an input wire and "marks" it as output.
#[derive(PartialEq, Eq)]
pub struct Output {
    pub in0: usize,
}
impl Gate for Output {
    fn check(&self, _values: &[QM31]) -> Result<(), String> {
        Ok(())
    }

    fn uses(&self) -> Vec<usize> {
        vec![self.in0]
    }

    fn yields(&self) -> Vec<usize> {
        vec![]
    }
}

impl std::fmt::Debug for Output {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "output [{}]", self.in0)
    }
}

/// Represents a circuit.
#[derive(Default, PartialEq, Eq)]
pub struct Circuit {
    pub n_vars: usize,
    pub add: Vec<Add>,
    pub sub: Vec<Sub>,
    pub mul: Vec<Mul>,
    pub pointwise_mul: Vec<PointwiseMul>,
    pub eq: Vec<Eq>,
    pub blake: Vec<Blake>,
    pub blake_g: Vec<BlakeGGate>,
    pub triple_xor: Vec<TripleXorGate>,
    pub m31_to_u32: Vec<M31ToU32Gate>,
    pub permutation: Vec<Permutation>,
    pub output: Vec<Output>,
}

impl Circuit {
    /// Returns an iterator over all the gates in the circuit.
    pub fn all_gates(&self) -> impl Iterator<Item = &dyn Gate> {
        let Circuit {
            n_vars: _,
            add,
            sub,
            mul,
            pointwise_mul,
            eq,
            blake,
            blake_g,
            triple_xor,
            m31_to_u32,
            permutation,
            output,
        } = self;
        chain!(
            add.iter().map(|g| g as &dyn Gate),
            sub.iter().map(|g| g as &dyn Gate),
            mul.iter().map(|g| g as &dyn Gate),
            pointwise_mul.iter().map(|g| g as &dyn Gate),
            eq.iter().map(|g| g as &dyn Gate),
            blake.iter().map(|g| g as &dyn Gate),
            blake_g.iter().map(|g| g as &dyn Gate),
            triple_xor.iter().map(|g| g as &dyn Gate),
            m31_to_u32.iter().map(|g| g as &dyn Gate),
            permutation.iter().map(|g| g as &dyn Gate),
            output.iter().map(|g| g as &dyn Gate),
        )
    }

    /// Checks if the circuit is satisfied by the given values.
    pub fn check(&self, values: &[QM31]) -> Result<(), String> {
        for gate in self.all_gates() {
            gate.check(values)?;
        }
        Ok(())
    }

    /// Returns the number of uses and number of yields for each variable (in the context of lookup
    /// terms).
    pub fn compute_multiplicities(&self) -> (Vec<usize>, Vec<usize>) {
        let mut n_uses = vec![0; self.n_vars];
        let mut n_yields = vec![0; self.n_vars];

        for gate in self.all_gates() {
            for use_var in gate.uses() {
                n_uses[use_var] += 1;
            }
            for yield_var in gate.yields() {
                n_yields[yield_var] += 1;
            }
        }

        (n_uses, n_yields)
    }

    /// Verifies that each variable appears exactly once as a yield.
    pub fn check_yields(&self) {
        for (idx, n_yields) in self.compute_multiplicities().1.iter().enumerate() {
            assert!(*n_yields == 1, "Variable {idx} appears {n_yields} times as a yield");
        }
    }
}

impl std::fmt::Debug for Circuit {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        for gate in self.all_gates() {
            writeln!(f, "{gate:?}")?;
        }
        Ok(())
    }
}
