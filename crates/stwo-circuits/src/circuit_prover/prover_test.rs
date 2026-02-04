use crate::circuit_air::statement::CircuitStatement;
use crate::circuit_prover::prover::{CircuitProof, finalize_context, prove_circuit};
use crate::circuits::blake::blake;
use crate::circuits::context::TraceContext;
use crate::circuits::ivalue::{IValue, qm31_from_u32s};
use crate::circuits::ops::{Guess, permute};
use crate::circuits::{context::Context, ops::guess};
use crate::eval;
use crate::stark_verifier::proof::ProofConfig;
use crate::stark_verifier::proof_from_stark_proof::proof_from_stark_proof;
use expect_test::expect;
use num_traits::{One, Zero};
use stwo::core::air::Component;
use stwo::core::channel::Blake2sM31Channel;
use stwo::core::fields::qm31::QM31;
use stwo::core::pcs::{CommitmentSchemeVerifier, TreeVec};
use stwo::core::vcs_lifted::blake2_merkle::Blake2sM31MerkleChannel;

// Not a power of 2 so that we can test component padding.
const N: usize = 1030;

pub fn build_fibonacci_context() -> Context<QM31> {
    let mut context = Context::<QM31>::default();

    let (mut a, mut b) = (guess(&mut context, QM31::zero()), guess(&mut context, QM31::one()));
    for _ in 2..N {
        (a, b) = (b, eval!(&mut context, (a) + (b)));
    }

    expect![[r#"
        (809871181 + 0i) + (0 + 0i)u
    "#]]
    .assert_debug_eq(&context.get(b));

    context
}

pub fn build_permutation_context() -> Context<QM31> {
    let mut context = Context::<QM31>::default();

    let a = guess(&mut context, qm31_from_u32s(0, 2, 0, 2));
    let b = guess(&mut context, qm31_from_u32s(1, 1, 1, 1));

    let outputs = permute(&mut context, &[a, b], IValue::sort_by_u_coordinate);
    let _outputs = permute(&mut context, &outputs, IValue::sort_by_u_coordinate);

    context
}

pub fn build_blake_gate_context() -> Context<QM31> {
    let mut context = Context::<QM31>::default();
    context.enable_assert_eq_on_eval();

    let input0 = guess(&mut context, qm31_from_u32s(0, 1, 2, 3));
    let input1 = guess(&mut context, qm31_from_u32s(0, 1, 2, 3));
    let input2 = guess(&mut context, qm31_from_u32s(0, 1, 2, 3));
    let input3 = guess(&mut context, qm31_from_u32s(0, 1, 2, 3));
    let input4 = guess(&mut context, qm31_from_u32s(0, 1, 2, 3));
    let input5 = guess(&mut context, qm31_from_u32s(0, 1, 2, 3));
    let input6 = guess(&mut context, qm31_from_u32s(0, 1, 2, 3));
    let input7 = guess(&mut context, qm31_from_u32s(0, 1, 2, 3));
    let input8 = guess(&mut context, qm31_from_u32s(0, 1, 2, 3));
    let input9 = guess(&mut context, qm31_from_u32s(0, 1, 2, 3));
    let input10 = guess(&mut context, qm31_from_u32s(0, 1, 2, 3));
    let input11 = guess(&mut context, qm31_from_u32s(0, 1, 2, 3));
    let input12 = guess(&mut context, qm31_from_u32s(0, 1, 2, 3));
    let input13 = guess(&mut context, qm31_from_u32s(0, 1, 2, 3));
    let input14 = guess(&mut context, qm31_from_u32s(0, 1, 2, 3));
    let input15 = guess(&mut context, qm31_from_u32s(0, 1, 2, 3));
    let input16 = guess(&mut context, qm31_from_u32s(0, 1, 2, 3));
    let input17 = guess(&mut context, qm31_from_u32s(0, 1, 2, 3));
    let input18 = guess(&mut context, qm31_from_u32s(0, 1, 2, 3));
    let input19 = guess(&mut context, qm31_from_u32s(0, 1, 2, 3));
    let input20 = guess(&mut context, qm31_from_u32s(0, 1, 2, 3));
    let input21 = guess(&mut context, qm31_from_u32s(0, 1, 2, 3));
    let input22 = guess(&mut context, qm31_from_u32s(0, 1, 2, 3));
    let input23 = guess(&mut context, qm31_from_u32s(0, 1, 2, 3));
    let input24 = guess(&mut context, qm31_from_u32s(0, 1, 2, 3));
    let input25 = guess(&mut context, qm31_from_u32s(0, 1, 2, 3));
    let input26 = guess(&mut context, qm31_from_u32s(0, 1, 2, 3));
    let input27 = guess(&mut context, qm31_from_u32s(0, 1, 2, 3));
    let input28 = guess(&mut context, qm31_from_u32s(0, 1, 2, 3));
    let input29 = guess(&mut context, qm31_from_u32s(0, 1, 2, 3));
    let input30 = guess(&mut context, qm31_from_u32s(0, 1, 2, 3));
    let input31 = guess(&mut context, qm31_from_u32s(0, 1, 2, 3));
    let input32 = guess(&mut context, qm31_from_u32s(0, 1, 2, 3));
    let input33 = guess(&mut context, qm31_from_u32s(0, 1, 2, 3));
    let input34 = guess(&mut context, qm31_from_u32s(0, 1, 2, 3));
    let input35 = guess(&mut context, qm31_from_u32s(0, 1, 2, 3));
    let input36 = guess(&mut context, qm31_from_u32s(0, 1, 2, 3));
    let input37 = guess(&mut context, qm31_from_u32s(0, 1, 2, 3));
    let input38 = guess(&mut context, qm31_from_u32s(0, 1, 2, 3));
    let input39 = guess(&mut context, qm31_from_u32s(0, 1, 2, 3));
    let input40 = guess(&mut context, qm31_from_u32s(0, 1, 2, 3));
    let input41 = guess(&mut context, qm31_from_u32s(0, 1, 2, 3));
    let input42 = guess(&mut context, qm31_from_u32s(0, 1, 2, 3));
    let input43 = guess(&mut context, qm31_from_u32s(0, 1, 2, 3));
    let input44 = guess(&mut context, qm31_from_u32s(0, 1, 2, 3));
    let input45 = guess(&mut context, qm31_from_u32s(0, 1, 2, 3));
    let input46 = guess(&mut context, qm31_from_u32s(0, 1, 2, 3));
    let input47 = guess(&mut context, qm31_from_u32s(0, 1, 2, 3));
    let input48 = guess(&mut context, qm31_from_u32s(0, 1, 2, 3));
    let input49 = guess(&mut context, qm31_from_u32s(0, 1, 2, 3));
    let input50 = guess(&mut context, qm31_from_u32s(0, 1, 2, 3));
    let input51 = guess(&mut context, qm31_from_u32s(0, 1, 2, 3));
    let input52 = guess(&mut context, qm31_from_u32s(0, 1, 2, 3));
    let input53 = guess(&mut context, qm31_from_u32s(0, 1, 2, 3));
    let input54 = guess(&mut context, qm31_from_u32s(0, 1, 2, 3));
    let input55 = guess(&mut context, qm31_from_u32s(0, 1, 2, 3));
    let input56 = guess(&mut context, qm31_from_u32s(0, 1, 2, 3));
    let input57 = guess(&mut context, qm31_from_u32s(0, 1, 2, 3));
    let input58 = guess(&mut context, qm31_from_u32s(0, 1, 2, 3));
    let input59 = guess(&mut context, qm31_from_u32s(0, 1, 2, 3));
    let input60 = guess(&mut context, qm31_from_u32s(0, 1, 2, 3));
    let input61 = guess(&mut context, qm31_from_u32s(0, 1, 2, 3));
    let input62 = guess(&mut context, qm31_from_u32s(0, 1, 2, 3));
    let input63 = guess(&mut context, qm31_from_u32s(0, 1, 2, 3));
    let input64 = guess(&mut context, qm31_from_u32s(0, 1, 2, 3));
    let input65 = guess(&mut context, qm31_from_u32s(0, 1, 2, 3));
    let input66 = guess(&mut context, qm31_from_u32s(0, 1, 2, 3));
    let input67 = guess(&mut context, qm31_from_u32s(0, 1, 2, 3));
    let input68 = guess(&mut context, qm31_from_u32s(0, 1, 2, 3));
    let input69 = guess(&mut context, qm31_from_u32s(0, 1, 2, 3));
    let input70 = guess(&mut context, qm31_from_u32s(0, 1, 2, 3));
    let input71 = guess(&mut context, qm31_from_u32s(0, 1, 2, 3));
    let input72 = guess(&mut context, qm31_from_u32s(0, 1, 2, 3));
    let input73 = guess(&mut context, qm31_from_u32s(0, 1, 2, 3));
    let input74 = guess(&mut context, qm31_from_u32s(0, 1, 2, 3));
    let input75 = guess(&mut context, qm31_from_u32s(0, 1, 2, 3));
    let input76 = guess(&mut context, qm31_from_u32s(0, 1, 2, 3));
    let input77 = guess(&mut context, qm31_from_u32s(0, 1, 2, 3));
    let input78 = guess(&mut context, qm31_from_u32s(0, 1, 2, 3));
    let input79 = guess(&mut context, qm31_from_u32s(0, 1, 2, 3));
    let input80 = guess(&mut context, qm31_from_u32s(0, 1, 2, 3));
    let input81 = guess(&mut context, qm31_from_u32s(0, 1, 2, 3));
    let input82 = guess(&mut context, qm31_from_u32s(0, 1, 2, 3));
    let input83 = guess(&mut context, qm31_from_u32s(0, 1, 2, 3));
    let input84 = guess(&mut context, qm31_from_u32s(0, 1, 2, 3));
    let input85 = guess(&mut context, qm31_from_u32s(0, 1, 2, 3));
    let input86 = guess(&mut context, qm31_from_u32s(0, 1, 2, 3));
    let input87 = guess(&mut context, qm31_from_u32s(0, 1, 2, 3));
    let input88 = guess(&mut context, qm31_from_u32s(0, 1, 2, 3));
    let input89 = guess(&mut context, qm31_from_u32s(0, 1, 2, 3));
    let input90 = guess(&mut context, qm31_from_u32s(0, 1, 2, 3));
    let input91 = guess(&mut context, qm31_from_u32s(0, 1, 2, 3));
    let input92 = guess(&mut context, qm31_from_u32s(0, 1, 2, 3));
    let input93 = guess(&mut context, qm31_from_u32s(0, 1, 2, 3));
    let input94 = guess(&mut context, qm31_from_u32s(0, 1, 2, 3));
    let input95 = guess(&mut context, qm31_from_u32s(0, 1, 2, 3));
    let input96 = guess(&mut context, qm31_from_u32s(0, 1, 2, 3));
    let input97 = guess(&mut context, qm31_from_u32s(0, 1, 2, 3));
    let input98 = guess(&mut context, qm31_from_u32s(0, 1, 2, 3));
    let input99 = guess(&mut context, qm31_from_u32s(0, 1, 2, 3));
    let input100 = guess(&mut context, qm31_from_u32s(0, 1, 2, 3));
    let input101 = guess(&mut context, qm31_from_u32s(0, 1, 2, 3));
    let input102 = guess(&mut context, qm31_from_u32s(0, 1, 2, 3));
    let input103 = guess(&mut context, qm31_from_u32s(0, 1, 2, 3));
    let input104 = guess(&mut context, qm31_from_u32s(0, 1, 2, 3));
    let input105 = guess(&mut context, qm31_from_u32s(0, 1, 2, 3));
    let input106 = guess(&mut context, qm31_from_u32s(0, 1, 2, 3));
    let input107 = guess(&mut context, qm31_from_u32s(0, 1, 2, 3));
    let input108 = guess(&mut context, qm31_from_u32s(0, 1, 2, 3));
    let input109 = guess(&mut context, qm31_from_u32s(0, 1, 2, 3));
    let input110 = guess(&mut context, qm31_from_u32s(0, 1, 2, 3));
    let input111 = guess(&mut context, qm31_from_u32s(0, 1, 2, 3));
    let input112 = guess(&mut context, qm31_from_u32s(0, 1, 2, 3));
    let input113 = guess(&mut context, qm31_from_u32s(0, 1, 2, 3));
    let input114 = guess(&mut context, qm31_from_u32s(0, 1, 2, 3));
    let input115 = guess(&mut context, qm31_from_u32s(0, 1, 2, 3));
    let input116 = guess(&mut context, qm31_from_u32s(0, 1, 2, 3));
    let input117 = guess(&mut context, qm31_from_u32s(0, 1, 2, 3));
    let input118 = guess(&mut context, qm31_from_u32s(0, 1, 2, 3));
    let input119 = guess(&mut context, qm31_from_u32s(0, 1, 2, 3));
    let input120 = guess(&mut context, qm31_from_u32s(0, 1, 2, 3));
    let input121 = guess(&mut context, qm31_from_u32s(0, 1, 2, 3));
    let input122 = guess(&mut context, qm31_from_u32s(0, 1, 2, 3));
    let input123 = guess(&mut context, qm31_from_u32s(0, 1, 2, 3));
    let input124 = guess(&mut context, qm31_from_u32s(0, 1, 2, 3));
    let input125 = guess(&mut context, qm31_from_u32s(0, 1, 2, 3));
    let input126 = guess(&mut context, qm31_from_u32s(0, 1, 2, 3));
    let input127 = guess(&mut context, qm31_from_u32s(0, 1, 2, 3));
    let input128 = guess(&mut context, qm31_from_u32s(0, 1, 2, 3));
    let input129 = guess(&mut context, qm31_from_u32s(0, 1, 2, 3));
    let input130 = guess(&mut context, qm31_from_u32s(0, 1, 2, 3));
    let input131 = guess(&mut context, qm31_from_u32s(0, 1, 2, 3));
    let input132 = guess(&mut context, qm31_from_u32s(0, 1, 2, 3));
    let input133 = guess(&mut context, qm31_from_u32s(0, 1, 2, 3));
    let input134 = guess(&mut context, qm31_from_u32s(0, 1, 2, 3));
    let input135 = guess(&mut context, qm31_from_u32s(0, 1, 2, 3));
    let input136 = guess(&mut context, qm31_from_u32s(0, 1, 2, 3));
    let input137 = guess(&mut context, qm31_from_u32s(0, 1, 2, 3));
    let input138 = guess(&mut context, qm31_from_u32s(0, 1, 2, 3));
    let input139 = guess(&mut context, qm31_from_u32s(0, 1, 2, 3));
    let input140 = guess(&mut context, qm31_from_u32s(0, 1, 2, 3));
    let input141 = guess(&mut context, qm31_from_u32s(0, 1, 2, 3));
    let input142 = guess(&mut context, qm31_from_u32s(0, 1, 2, 3));
    let input143 = guess(&mut context, qm31_from_u32s(0, 1, 2, 3));
    let input144 = guess(&mut context, qm31_from_u32s(0, 1, 2, 3));
    let input145 = guess(&mut context, qm31_from_u32s(0, 1, 2, 3));
    let input146 = guess(&mut context, qm31_from_u32s(0, 1, 2, 3));
    let input147 = guess(&mut context, qm31_from_u32s(0, 1, 2, 3));
    let input148 = guess(&mut context, qm31_from_u32s(0, 1, 2, 3));
    let input149 = guess(&mut context, qm31_from_u32s(0, 1, 2, 3));
    let input150 = guess(&mut context, qm31_from_u32s(0, 1, 2, 3));
    let input151 = guess(&mut context, qm31_from_u32s(0, 1, 2, 3));
    let input152 = guess(&mut context, qm31_from_u32s(0, 1, 2, 3));
    let input153 = guess(&mut context, qm31_from_u32s(0, 1, 2, 3));
    let input154 = guess(&mut context, qm31_from_u32s(0, 1, 2, 3));
    let input155 = guess(&mut context, qm31_from_u32s(0, 1, 2, 3));
    let input156 = guess(&mut context, qm31_from_u32s(0, 1, 2, 3));
    let input157 = guess(&mut context, qm31_from_u32s(0, 1, 2, 3));
    let input158 = guess(&mut context, qm31_from_u32s(0, 1, 2, 3));
    let input159 = guess(&mut context, qm31_from_u32s(0, 1, 2, 3));
    let input160 = guess(&mut context, qm31_from_u32s(0, 1, 2, 3));
    let input161 = guess(&mut context, qm31_from_u32s(0, 1, 2, 3));
    let input162 = guess(&mut context, qm31_from_u32s(0, 1, 2, 3));
    let input163 = guess(&mut context, qm31_from_u32s(0, 1, 2, 3));
    let input164 = guess(&mut context, qm31_from_u32s(0, 1, 2, 3));
    let input165 = guess(&mut context, qm31_from_u32s(0, 1, 2, 3));
    let input166 = guess(&mut context, qm31_from_u32s(0, 1, 2, 3));
    let input167 = guess(&mut context, qm31_from_u32s(0, 1, 2, 3));
    let input168 = guess(&mut context, qm31_from_u32s(0, 1, 2, 3));
    let input169 = guess(&mut context, qm31_from_u32s(0, 1, 2, 3));
    let input170 = guess(&mut context, qm31_from_u32s(0, 1, 2, 3));
    let input171 = guess(&mut context, qm31_from_u32s(0, 1, 2, 3));
    let input172 = guess(&mut context, qm31_from_u32s(0, 1, 2, 3));
    let input173 = guess(&mut context, qm31_from_u32s(0, 1, 2, 3));
    let input174 = guess(&mut context, qm31_from_u32s(0, 1, 2, 3));
    let input175 = guess(&mut context, qm31_from_u32s(0, 1, 2, 3));
    let input176 = guess(&mut context, qm31_from_u32s(0, 1, 2, 3));
    let input177 = guess(&mut context, qm31_from_u32s(0, 1, 2, 3));
    let input178 = guess(&mut context, qm31_from_u32s(0, 1, 2, 3));
    let input179 = guess(&mut context, qm31_from_u32s(0, 1, 2, 3));
    let input180 = guess(&mut context, qm31_from_u32s(0, 1, 2, 3));
    let input181 = guess(&mut context, qm31_from_u32s(0, 1, 2, 3));
    let input182 = guess(&mut context, qm31_from_u32s(0, 1, 2, 3));
    let input183 = guess(&mut context, qm31_from_u32s(0, 1, 2, 3));
    let input184 = guess(&mut context, qm31_from_u32s(0, 1, 2, 3));
    let input185 = guess(&mut context, qm31_from_u32s(0, 1, 2, 3));
    let input186 = guess(&mut context, qm31_from_u32s(0, 1, 2, 3));
    let input187 = guess(&mut context, qm31_from_u32s(0, 1, 2, 3));
    let input188 = guess(&mut context, qm31_from_u32s(0, 1, 2, 3));
    let input189 = guess(&mut context, qm31_from_u32s(0, 1, 2, 3));
    let input190 = guess(&mut context, qm31_from_u32s(0, 1, 2, 3));
    let input191 = guess(&mut context, qm31_from_u32s(0, 1, 2, 3));
    let input192 = guess(&mut context, qm31_from_u32s(0, 1, 2, 3));
    let input193 = guess(&mut context, qm31_from_u32s(0, 1, 2, 3));
    let input194 = guess(&mut context, qm31_from_u32s(0, 1, 2, 3));
    let input195 = guess(&mut context, qm31_from_u32s(0, 1, 2, 3));
    let input196 = guess(&mut context, qm31_from_u32s(0, 1, 2, 3));
    let input197 = guess(&mut context, qm31_from_u32s(0, 1, 2, 3));
    let input198 = guess(&mut context, qm31_from_u32s(0, 1, 2, 3));
    let input199 = guess(&mut context, qm31_from_u32s(0, 1, 2, 3));
    let input200 = guess(&mut context, qm31_from_u32s(0, 1, 2, 3));
    let input201 = guess(&mut context, qm31_from_u32s(0, 1, 2, 3));
    let input202 = guess(&mut context, qm31_from_u32s(0, 1, 2, 3));
    let input203 = guess(&mut context, qm31_from_u32s(0, 1, 2, 3));
    let input204 = guess(&mut context, qm31_from_u32s(0, 1, 2, 3));
    let input205 = guess(&mut context, qm31_from_u32s(0, 1, 2, 3));
    let input206 = guess(&mut context, qm31_from_u32s(0, 1, 2, 3));
    let input207 = guess(&mut context, qm31_from_u32s(0, 1, 2, 3));
    let input208 = guess(&mut context, qm31_from_u32s(0, 1, 2, 3));
    let input209 = guess(&mut context, qm31_from_u32s(0, 1, 2, 3));
    let input210 = guess(&mut context, qm31_from_u32s(0, 1, 2, 3));
    let input211 = guess(&mut context, qm31_from_u32s(0, 1, 2, 3));
    let input212 = guess(&mut context, qm31_from_u32s(0, 1, 2, 3));
    let input213 = guess(&mut context, qm31_from_u32s(0, 1, 2, 3));
    let input214 = guess(&mut context, qm31_from_u32s(0, 1, 2, 3));
    let input215 = guess(&mut context, qm31_from_u32s(0, 1, 2, 3));
    let input216 = guess(&mut context, qm31_from_u32s(0, 1, 2, 3));
    let input217 = guess(&mut context, qm31_from_u32s(0, 1, 2, 3));
    let input218 = guess(&mut context, qm31_from_u32s(0, 1, 2, 3));
    let input219 = guess(&mut context, qm31_from_u32s(0, 1, 2, 3));
    let input220 = guess(&mut context, qm31_from_u32s(0, 1, 2, 3));
    let input221 = guess(&mut context, qm31_from_u32s(0, 1, 2, 3));
    let input222 = guess(&mut context, qm31_from_u32s(0, 1, 2, 3));
    let input223 = guess(&mut context, qm31_from_u32s(0, 1, 2, 3));
    let input224 = guess(&mut context, qm31_from_u32s(0, 1, 2, 3));
    let input225 = guess(&mut context, qm31_from_u32s(0, 1, 2, 3));
    let input226 = guess(&mut context, qm31_from_u32s(0, 1, 2, 3));
    let input227 = guess(&mut context, qm31_from_u32s(0, 1, 2, 3));
    let input228 = guess(&mut context, qm31_from_u32s(0, 1, 2, 3));
    let input229 = guess(&mut context, qm31_from_u32s(0, 1, 2, 3));
    let input230 = guess(&mut context, qm31_from_u32s(0, 1, 2, 3));
    let input231 = guess(&mut context, qm31_from_u32s(0, 1, 2, 3));
    let input232 = guess(&mut context, qm31_from_u32s(0, 1, 2, 3));
    let input233 = guess(&mut context, qm31_from_u32s(0, 1, 2, 3));
    let input234 = guess(&mut context, qm31_from_u32s(0, 1, 2, 3));
    let input235 = guess(&mut context, qm31_from_u32s(0, 1, 2, 3));
    let input236 = guess(&mut context, qm31_from_u32s(0, 1, 2, 3));
    let input237 = guess(&mut context, qm31_from_u32s(0, 1, 2, 3));
    let input238 = guess(&mut context, qm31_from_u32s(0, 1, 2, 3));
    let input239 = guess(&mut context, qm31_from_u32s(0, 1, 2, 3));
    let input240 = guess(&mut context, qm31_from_u32s(0, 1, 2, 3));
    let input241 = guess(&mut context, qm31_from_u32s(0, 1, 2, 3));
    let input242 = guess(&mut context, qm31_from_u32s(0, 1, 2, 3));
    let input243 = guess(&mut context, qm31_from_u32s(0, 1, 2, 3));
    let input244 = guess(&mut context, qm31_from_u32s(0, 1, 2, 3));
    let input245 = guess(&mut context, qm31_from_u32s(0, 1, 2, 3));
    let input246 = guess(&mut context, qm31_from_u32s(0, 1, 2, 3));
    let input247 = guess(&mut context, qm31_from_u32s(0, 1, 2, 3));
    let input248 = guess(&mut context, qm31_from_u32s(0, 1, 2, 3));
    let input249 = guess(&mut context, qm31_from_u32s(0, 1, 2, 3));
    let input250 = guess(&mut context, qm31_from_u32s(0, 1, 2, 3));
    let input251 = guess(&mut context, qm31_from_u32s(0, 1, 2, 3));
    let input252 = guess(&mut context, qm31_from_u32s(0, 1, 2, 3));
    let input253 = guess(&mut context, qm31_from_u32s(0, 1, 2, 3));
    let input254 = guess(&mut context, qm31_from_u32s(0, 1, 2, 3));
    let input255 = guess(&mut context, qm31_from_u32s(0, 1, 2, 3));
    // let input17 = guess(&mut context, qm31_from_u32s(0, 1, 2, 3));
    // let input18 = guess(&mut context, qm31_from_u32s(0, 1, 2, 3));
    // let input19 = guess(&mut context, qm31_from_u32s(0, 1, 2, 3));
    let _output = blake(
        &mut context,
        &[
            input0, input1, input2, input3, input4, input5, input6, input7, input8, input9,
            input10, input11, input12, input13, input14, input15, input16, input17, input18,
            input19, input20, input21, input22, input23, input24, input25, input26, input27,
            input28, input29, input30, input31, input32, input33, input34, input35, input36,
            input37, input38, input39, input40, input41, input42, input43, input44, input45,
            input46, input47, input48, input49, input50, input51, input52, input53, input54,
            input55, input56, input57, input58, input59, input60, input61, input62, input63,
            input64, input65, input66, input67, input68, input69, input70, input71, input72,
            input73, input74, input75, input76, input77, input78, input79, input80, input81,
            input82, input83, input84, input85, input86, input87, input88, input89, input90,
            input91, input92, input93, input94, input95, input96, input97, input98, input99,
            input100, input101, input102, input103, input104, input105, input106, input107,
            input108, input109, input110, input111, input112, input113, input114, input115,
            input116, input117, input118, input119, input120, input121, input122, input123,
            input124, input125, input126, input127, input128, input129, input130, input131,
            input132, input133, input134, input135, input136, input137, input138, input139,
            input140, input141, input142, input143, input144, input145, input146, input147,
            input148, input149, input150, input151, input152, input153, input154, input155,
            input156, input157, input158, input159, input160, input161, input162, input163,
            input164, input165, input166, input167, input168, input169, input170, input171,
            input172, input173, input174, input175, input176, input177, input178, input179,
            input180, input181, input182, input183, input184, input185, input186, input187,
            input188, input189, input190, input191, input192, input193, input194, input195,
            input196, input197, input198, input199, input200, input201, input202, input203,
            input204, input205, input206, input207, input208, input209, input210, input211,
            input212, input213, input214, input215, input216, input217, input218, input219,
            input220, input221, input222, input223, input224, input225, input226, input227,
            input228, input229, input230, input231, input232, input233, input234, input235,
            input236, input237, input238, input239, input240, input241, input242, input243,
            input244, input245, input246, input247, input248, input249, input250, input251,
            input252, input253, input254, input255,
        ],
        256 * 16,
    );
    // let n_bytes = guess(&mut context, QM31::from(16));
    // let output = guess(&mut context, qm31_from_u32s(4, 5, 6, 7));

    context
}

#[test]
fn test_prove_and_stark_verify_blake_gate_context() {
    let mut blake_gate_context = build_blake_gate_context();
    blake_gate_context.finalize_guessed_vars();
    blake_gate_context.validate_circuit();

    let CircuitProof { components, claim, interaction_claim, pcs_config, stark_proof } =
        prove_circuit(&mut blake_gate_context);
    assert!(stark_proof.is_ok());
}

#[test]
fn test_prove_and_stark_verify_permutation_context() {
    let mut permutation_context = build_permutation_context();
    permutation_context.finalize_guessed_vars();
    permutation_context.validate_circuit();

    let CircuitProof { components, claim, interaction_claim, pcs_config, stark_proof } =
        prove_circuit(&mut permutation_context);
    assert!(stark_proof.is_ok());
    let proof = stark_proof.unwrap();

    // Verify.
    let verifier_channel = &mut Blake2sM31Channel::default();
    let commitment_scheme =
        &mut CommitmentSchemeVerifier::<Blake2sM31MerkleChannel>::new(pcs_config);

    // Retrieve the expected column sizes in each commitment interaction, from the AIR.
    let sizes = TreeVec::concat_cols(components.iter().map(|c| c.trace_log_degree_bounds()));

    commitment_scheme.commit(proof.proof.commitments[0], &sizes[0], verifier_channel);
    claim.mix_into(verifier_channel);
    commitment_scheme.commit(proof.proof.commitments[1], &sizes[1], verifier_channel);
    // TODO(Gali): Draw interaction element?
    interaction_claim.mix_into(verifier_channel);
    commitment_scheme.commit(proof.proof.commitments[2], &sizes[2], verifier_channel);
    stwo::core::verifier::verify(
        &components.iter().map(|c| c.as_ref()).collect::<Vec<&dyn Component>>(),
        verifier_channel,
        commitment_scheme,
        proof.proof,
    )
    .unwrap();
}

#[test]
fn test_prove_and_stark_verify_fibonacci_context() {
    let mut fibonacci_context = build_fibonacci_context();
    fibonacci_context.finalize_guessed_vars();
    fibonacci_context.validate_circuit();

    let CircuitProof { components, claim, interaction_claim, pcs_config, stark_proof } =
        prove_circuit(&mut fibonacci_context);
    assert!(stark_proof.is_ok());
    let proof = stark_proof.unwrap();

    // Verify.
    let verifier_channel = &mut Blake2sM31Channel::default();
    let commitment_scheme =
        &mut CommitmentSchemeVerifier::<Blake2sM31MerkleChannel>::new(pcs_config);

    // Retrieve the expected column sizes in each commitment interaction, from the AIR.
    let sizes = TreeVec::concat_cols(components.iter().map(|c| c.trace_log_degree_bounds()));

    commitment_scheme.commit(proof.proof.commitments[0], &sizes[0], verifier_channel);
    claim.mix_into(verifier_channel);
    commitment_scheme.commit(proof.proof.commitments[1], &sizes[1], verifier_channel);
    // TODO(Gali): Draw interaction element?
    interaction_claim.mix_into(verifier_channel);
    commitment_scheme.commit(proof.proof.commitments[2], &sizes[2], verifier_channel);
    stwo::core::verifier::verify(
        &components.iter().map(|c| c.as_ref()).collect::<Vec<&dyn Component>>(),
        verifier_channel,
        commitment_scheme,
        proof.proof,
    )
    .unwrap();
}

#[test]
fn test_prove_and_circuit_verify_fibonacci_context() {
    let mut fibonacci_context = build_fibonacci_context();
    fibonacci_context.finalize_guessed_vars();
    fibonacci_context.validate_circuit();

    let CircuitProof { components: _, claim, interaction_claim, pcs_config, stark_proof } =
        prove_circuit(&mut fibonacci_context);
    assert!(stark_proof.is_ok());
    let proof = stark_proof.unwrap();

    // Verify.
    let log_trace_size = claim.log_sizes.iter().max().unwrap();
    let statement = CircuitStatement::default();
    let config = ProofConfig::from_statement(&statement, *log_trace_size as usize, &pcs_config);

    let mut context = TraceContext::default();
    let proof = proof_from_stark_proof(
        &proof,
        &config,
        claim.log_sizes.to_vec(),
        interaction_claim.claimed_sums.to_vec(),
    );
    let proof_vars = proof.guess(&mut context);

    crate::stark_verifier::verify::verify(&mut context, &proof_vars, &config, &statement);
    context.check_vars_used();
    context.finalize_guessed_vars();
    context.circuit.check_yields();
    context.validate_circuit();
}

#[test]
fn test_finalize_context() {
    let mut context = build_fibonacci_context();
    finalize_context(&mut context);

    assert!(context.circuit.add.len().is_power_of_two());
    context.validate_circuit();
}
