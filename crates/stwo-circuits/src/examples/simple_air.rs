use itertools::{Itertools, zip_eq};
use num_traits::One;
use stwo::core::ColumnVec;
use stwo::core::air::Component;
use stwo::core::channel::{Blake2sM31Channel, Channel};
use stwo::core::fields::FieldExpOps;
use stwo::core::fields::m31::BaseField;
use stwo::core::fields::qm31::{QM31, SecureField};
use stwo::core::pcs::PcsConfig;
use stwo::core::poly::circle::CanonicCoset;
use stwo::core::proof::ExtendedStarkProof;
use stwo::core::vcs_lifted::blake2_merkle::{Blake2sM31MerkleChannel, Blake2sM31MerkleHasher};
use stwo::prover::backend::Col;
use stwo::prover::backend::Column;
use stwo::prover::backend::simd::SimdBackend;
use stwo::prover::backend::simd::column::BaseColumn;
use stwo::prover::backend::simd::m31::{LOG_N_LANES, PackedBaseField};
use stwo::prover::backend::simd::qm31::PackedSecureField;
use stwo::prover::poly::BitReversedOrder;
use stwo::prover::poly::circle::{CircleEvaluation, PolyOps};
use stwo::prover::{CommitmentSchemeProver, prove_ex};
use stwo_constraint_framework::logup::LookupElements;
use stwo_constraint_framework::preprocessed_columns::PreProcessedColumnId;
use stwo_constraint_framework::{
    EvalAtRow, FrameworkComponent, FrameworkEval, LogupTraceGenerator, RelationEntry,
    TraceLocationAllocator, relation,
};

#[cfg(test)]
#[path = "simple_air_test.rs"]
pub mod simple_air_test;

pub const FIB_SEQUENCE_LENGTH: usize = 4;

relation!(SimpleRelation, 2);

pub type SimpleComponent = FrameworkComponent<Eval>;

pub struct FibInput {
    a: PackedBaseField,
    b: PackedBaseField,
}

/// A component that enforces a variation of the Fibonacci sequence:
///    `a_{i + 2} = a_{i + 1}^2 + a_{i}^2 + row_const`.
///
/// The `row_const` is a different constant for each row.
///
/// The last two elements of the sequence are added to the relation.
/// Each row contains a separate sequence of length `N`.
#[derive(Clone)]
pub struct Eval {
    pub lookup_elements: SimpleRelation,
    pub preprocessed_column_id: PreProcessedColumnId,
    pub log_instances: u32,
}
impl FrameworkEval for Eval {
    fn log_size(&self) -> u32 {
        self.log_instances
    }
    fn max_constraint_log_degree_bound(&self) -> u32 {
        self.log_instances + 1
    }
    fn evaluate<E: EvalAtRow>(&self, mut eval: E) -> E {
        let row_const = eval.get_preprocessed_column(self.preprocessed_column_id.clone());
        let mut a = eval.next_trace_mask();
        let mut b = eval.next_trace_mask();
        for _ in 2..FIB_SEQUENCE_LENGTH {
            let c = eval.next_trace_mask();
            eval.add_constraint(c.clone() - (a.square() + b.square() + row_const.clone()));
            a = b;
            b = c;
        }

        eval.add_to_relation(RelationEntry::new(&self.lookup_elements, E::EF::one(), &[a, b]));
        eval.finalize_logup();

        eval
    }
}

/// Generates a trace for the test.
fn generate_trace(
    log_instances: u32,
) -> ColumnVec<CircleEvaluation<SimdBackend, BaseField, BitReversedOrder>> {
    let inputs = (0..(1 << (log_instances - LOG_N_LANES)))
        .map(|i| FibInput {
            a: PackedBaseField::one(),
            b: PackedBaseField::from_array(std::array::from_fn(|j| {
                BaseField::from_u32_unchecked((i * 16 + j) as u32)
            })),
        })
        .collect_vec();

    let mut trace = (0..FIB_SEQUENCE_LENGTH)
        .map(|_| Col::<SimdBackend, BaseField>::zeros(1 << log_instances))
        .collect_vec();
    let row_const: BaseColumn = (0..(1 << log_instances)).map(|i| i.into()).collect();
    for (vec_index, (input, row_const)) in zip_eq(inputs, row_const.data).enumerate() {
        let mut a = input.a;
        let mut b = input.b;
        trace[0].data[vec_index] = a;
        trace[1].data[vec_index] = b;
        trace.iter_mut().skip(2).for_each(|col| {
            (a, b) = (b, a.square() + b.square() + row_const);
            col.data[vec_index] = b;
        });
    }
    let domain = CanonicCoset::new(log_instances).circle_domain();
    trace
        .into_iter()
        .map(|eval| CircleEvaluation::<SimdBackend, _, BitReversedOrder>::new(domain, eval))
        .collect_vec()
}

/// Generates the interaction trace for the test.
pub fn generate_interaction_trace(
    trace: &ColumnVec<CircleEvaluation<SimdBackend, BaseField, BitReversedOrder>>,
    lookup_elements: &LookupElements<2>,
) -> (ColumnVec<CircleEvaluation<SimdBackend, BaseField, BitReversedOrder>>, SecureField) {
    let log_instances = trace[0].values.length.ilog2();
    eprintln!("log_instances: {}", log_instances);
    let mut logup_gen = LogupTraceGenerator::new(log_instances);

    let mut col_gen = logup_gen.new_col();
    for vec_row in 0..(1 << (log_instances - LOG_N_LANES)) {
        let denom: PackedSecureField = lookup_elements.combine(&[
            trace[FIB_SEQUENCE_LENGTH - 2].values.data[vec_row],
            trace[FIB_SEQUENCE_LENGTH - 1].values.data[vec_row],
        ]);
        col_gen.write_frac(vec_row, PackedSecureField::one(), denom);
    }
    col_gen.finalize_col();

    logup_gen.finalize_last()
}

/// Creates a proof for the simple AIR. See documentation in [Eval].
pub fn create_proof()
-> (Vec<Box<dyn Component>>, Vec<QM31>, ExtendedStarkProof<Blake2sM31MerkleHasher>) {
    const LOG_SIZE_SHORT: u32 = 4;
    const LOG_SIZE_LONG: u32 = 5;

    let config = PcsConfig::default();
    // Precompute twiddles.
    let twiddles = SimdBackend::precompute_twiddles(
        CanonicCoset::new(LOG_SIZE_LONG + 1 + config.fri_config.log_blowup_factor)
            .circle_domain()
            .half_coset,
    );

    // Setup protocol.
    let prover_channel = &mut Blake2sM31Channel::default();
    let mut commitment_scheme =
        CommitmentSchemeProver::<SimdBackend, Blake2sM31MerkleChannel>::new(config, &twiddles);
    commitment_scheme.set_store_polynomials_coefficients();

    // Preprocessed trace
    let mut tree_builder = commitment_scheme.tree_builder();
    let domain = CanonicCoset::new(LOG_SIZE_SHORT).circle_domain();
    let preprocessed_column: BaseColumn =
        (0..2_u32.pow(LOG_SIZE_SHORT)).map(|i| i.into()).collect();
    let preprocessed_column_eval_1 = CircleEvaluation::new(domain, preprocessed_column);
    let domain = CanonicCoset::new(LOG_SIZE_LONG).circle_domain();
    let preprocessed_column: BaseColumn = (0..2_u32.pow(LOG_SIZE_LONG)).map(|i| i.into()).collect();
    let preprocessed_column_eval_2 = CircleEvaluation::new(domain, preprocessed_column);
    tree_builder.extend_evals([preprocessed_column_eval_1, preprocessed_column_eval_2]);
    tree_builder.commit(prover_channel);

    // Trace.
    let mut tree_builder = commitment_scheme.tree_builder();
    let trace_1 = generate_trace(LOG_SIZE_SHORT);
    let trace_2 = generate_trace(LOG_SIZE_LONG);
    let trace = [trace_1.clone(), trace_2.clone()].concat();
    tree_builder.extend_evals(trace);
    tree_builder.commit(prover_channel);

    // TODO(lior): Add proof of work before drawing the lookup elements.

    // Draw lookup element.
    let lookup_elements = SimpleRelation::draw(prover_channel);

    // Interaction trace.
    let (interaction_trace_1, claimed_sum_1) =
        generate_interaction_trace(&trace_1, &lookup_elements.0);
    let (interaction_trace_2, claimed_sum_2) =
        generate_interaction_trace(&trace_2, &lookup_elements.0);
    prover_channel.mix_felts(&[claimed_sum_1, claimed_sum_2]);
    let mut tree_builder = commitment_scheme.tree_builder();
    tree_builder.extend_evals([interaction_trace_1.clone(), interaction_trace_2.clone()].concat());
    tree_builder.commit(prover_channel);

    let preprocessed_column_id_1 = PreProcessedColumnId { id: "row_const".into() };
    let preprocessed_column_id_2 = PreProcessedColumnId { id: "row_const2".into() };

    let mut trace_alloc = TraceLocationAllocator::new_with_preprocessed_columns(&[
        preprocessed_column_id_1.clone(),
        preprocessed_column_id_2.clone(),
    ]);

    // Prove constraints.
    let component_1 = SimpleComponent::new(
        &mut trace_alloc,
        Eval {
            log_instances: LOG_SIZE_SHORT,
            lookup_elements: lookup_elements.clone(),
            preprocessed_column_id: preprocessed_column_id_1,
        },
        claimed_sum_1,
    );

    let component_2 = SimpleComponent::new(
        &mut trace_alloc,
        Eval {
            log_instances: LOG_SIZE_LONG,
            lookup_elements,
            preprocessed_column_id: preprocessed_column_id_2,
        },
        claimed_sum_2,
    );

    let proof = prove_ex::<SimdBackend, Blake2sM31MerkleChannel>(
        &[&component_1, &component_2],
        prover_channel,
        commitment_scheme,
    )
    .unwrap();

    let components: Vec<Box<dyn Component>> = vec![Box::new(component_1), Box::new(component_2)];

    (components, vec![claimed_sum_1, claimed_sum_2], proof)
}
