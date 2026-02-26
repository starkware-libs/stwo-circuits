use circuits::context::TraceContext;
use circuits::ivalue::qm31_from_u32s;
use circuits::ops::Guess;
use circuits::simd::Simd;
use circuits::test_utils::simd_from_u32s;
use circuits_stark_verifier::fri::fri_decommit;
use circuits_stark_verifier::fri_proof::{FriCommitProof, FriProof, compute_all_line_fold_steps};
use circuits_stark_verifier::merkle::{AuthPath, AuthPaths};
use circuits_stark_verifier::proof::ProofConfig;
use circuits_stark_verifier::select_queries::select_queries;
use itertools::{chain, zip_eq};
use num_traits::One;
use stwo::core::channel::Blake2sM31Channel;
use stwo::core::circle::Coset;
use stwo::core::fields::m31::BaseField;
use stwo::core::fields::qm31::{QM31, SecureField};
use stwo::core::fri::{ExtendedFriProof, FriConfig};
use stwo::core::poly::circle::CircleDomain;
use stwo::core::queries::Queries;
use stwo::core::vcs::blake2_hash::Blake2sM31Hasher;
use stwo::core::vcs_lifted::blake2_merkle::{Blake2sM31MerkleChannel, Blake2sM31MerkleHasher};
use stwo::prover::backend::CpuBackend;
use stwo::prover::backend::cpu::CpuCirclePoly;
use stwo::prover::fri::FriProver;
use stwo::prover::poly::BitReversedOrder;
use stwo::prover::poly::circle::{PolyOps, SecureEvaluation};

#[test]
fn test_fri_decommit_with_jumps() {
    let fri_proof = create_fri_proof();
    let log_domain_size = 8;

    let mut context = TraceContext::default();
    // Make a dummy config.
    let config = ProofConfig {
        n_proof_of_work_bits: 0,
        n_preprocessed_columns: 0,
        n_trace_columns: 0,
        n_interaction_columns: 0,
        trace_columns_per_component: vec![],
        interaction_columns_per_component: vec![],
        cumulative_sum_columns: vec![],
        n_components: 0,
        fri: circuits_stark_verifier::fri_proof::FriConfig {
            log_trace_size: 6,
            log_blowup_factor: 2,
            n_queries: 7,
            log_n_last_layer_coefs: 0,
            line_fold_step: 2,
            circle_fold_step: 2,
        },
        interaction_pow_bits: 0,
    };

    // Compute FRI input.
    let query_indices: Vec<usize> = vec![0, 12, 34, 56, 78, 89, 101];
    let input = simd_from_u32s(&mut context, query_indices.iter().map(|x| *x as u32).collect());
    let queries = select_queries(&mut context, &input, log_domain_size);
    let fri_input: Vec<_> = query_indices
        .iter()
        .map(|query| context.constant(fri_proof.aux.first_layer.all_values[0][query]))
        .collect();
    let bits: Vec<Vec<_>> =
        queries.bits.iter().map(|simd| Simd::unpack(&mut context, simd)).collect();

    // Compute the circuit FriProof.
    let all_line_fold_steps = compute_all_line_fold_steps(
        config.fri.log_trace_size - config.fri.circle_fold_step - config.fri.log_n_last_layer_coefs,
        config.fri.line_fold_step,
    );
    let all_fold_steps = [&[config.fri.circle_fold_step], all_line_fold_steps.as_slice()].concat();
    let witness = test_construct_fri_witness(&fri_proof, &all_fold_steps, &query_indices);
    let auth_paths =
        test_construct_fri_auth_paths(&fri_proof, &config, &query_indices, &all_line_fold_steps);

    let circuit_fri_proof = FriProof {
        commit: FriCommitProof {
            layer_commitments: chain!(
                [fri_proof.proof.first_layer.commitment.into()],
                fri_proof.proof.inner_layers.iter().map(|layer| layer.commitment.into()),
            )
            .collect(),
            last_layer_coefs: fri_proof.proof.last_layer_poly.into_ordered_coefficients(),
        },
        auth_paths,
        circle_fri_siblings: vec![],
        witness_per_query_per_tree: witness,
    };
    let circuit_fri_proof = circuit_fri_proof.guess(&mut context);

    // Folding alphas.
    let alpha_values = [
        qm31_from_u32s(1011730217, 238354028, 1321702146, 1634795701),
        qm31_from_u32s(1837551196, 996059164, 541247751, 126611986),
        qm31_from_u32s(1889251375, 1206270873, 523653533, 1863366183),
    ];
    let alphas: Vec<_> = alpha_values.iter().map(|x| context.constant(*x)).collect();

    fri_decommit(
        &mut context,
        &circuit_fri_proof,
        &config.fri,
        &fri_input,
        &bits,
        &queries.bits,
        &queries.points,
        &alphas,
    );
    context.validate_circuit();
    println!("Stats: {:?}", context.stats);
}
/// Returns an evaluation of a random polynomial with degree `2^log_degree`.
///
/// The evaluation domain size is `2^(log_degree + log_blowup_factor)`.
/// Copied from [`stwo::core::fri::tests::polynomial_evaluation`].
fn polynomial_evaluation(
    log_degree: u32,
    log_blowup_factor: u32,
) -> SecureEvaluation<CpuBackend, BitReversedOrder> {
    let poly = CpuCirclePoly::new(vec![BaseField::one(); 1 << log_degree]);
    let coset = Coset::half_odds(log_degree + log_blowup_factor - 1);
    let domain = CircleDomain::new(coset);
    let values = poly.evaluate(domain);
    SecureEvaluation::new(domain, values.into_iter().map(SecureField::from).collect())
}

fn create_fri_proof() -> ExtendedFriProof<Blake2sM31Hasher> {
    const LOG_BLOWUP_FACTOR: u32 = 2;
    const LOG_TRACE_SIZE: u32 = 6;

    let config = FriConfig::new(0, LOG_BLOWUP_FACTOR, 3, 2, 2);
    let column = polynomial_evaluation(LOG_TRACE_SIZE, LOG_BLOWUP_FACTOR);
    let twiddles = CpuBackend::precompute_twiddles(column.domain.half_coset);
    let prover = FriProver::<CpuBackend, Blake2sM31MerkleChannel>::commit(
        &mut Blake2sM31Channel::default(),
        config,
        &column,
        &twiddles,
    );
    let queries = Queries::new(&[0, 12, 34, 56, 78, 89, 101], LOG_TRACE_SIZE + LOG_BLOWUP_FACTOR);
    prover.decommit_on_queries(&queries)
}

fn test_construct_fri_witness(
    proof: &ExtendedFriProof<Blake2sM31MerkleHasher>,
    all_fold_steps: &[usize],
    query_locations: &[usize],
) -> Vec<Vec<Vec<QM31>>> {
    let all_layers =
        [&[&proof.aux.first_layer], proof.aux.inner_layers.iter().collect::<Vec<_>>().as_slice()]
            .concat();
    let mut witness_per_query_per_tree = vec![vec![]; all_layers.len()];

    for query in query_locations {
        let mut pos = *query;
        for (tree_idx, (layer, step)) in zip_eq(&all_layers, all_fold_steps).enumerate() {
            let start = (pos >> step) << step;
            eprintln!("Tree: {tree_idx}. Decommitment positions: {:?}", start..start + (1 << step));
            let coset_vals: Vec<_> =
                (start..start + (1 << step)).map(|i| layer.all_values[0][&i]).collect();
            witness_per_query_per_tree[tree_idx].push(coset_vals);
            pos >>= step;
        }
    }
    witness_per_query_per_tree
}

/// Constructs [AuthPaths] for the FRI trees with the values from the given proof
/// ([ExtendedStarkProof]).
pub fn test_construct_fri_auth_paths(
    proof: &ExtendedFriProof<Blake2sM31MerkleHasher>,
    config: &ProofConfig,
    query_locations: &[usize],
    all_line_fold_steps: &[usize],
) -> AuthPaths<QM31> {
    let unsorted_query_locations = &query_locations;
    let layers = chain!([&proof.aux.first_layer], &proof.aux.inner_layers);
    let n = config.log_evaluation_domain_size();
    // Gather the layer log sizes.
    let mut layer_log_sizes = vec![n, n - config.fri.circle_fold_step];
    for step in all_line_fold_steps.iter().take(proof.aux.inner_layers.len() - 1) {
        layer_log_sizes.push(layer_log_sizes.last().unwrap() - step);
    }
    let mut all_fold_steps = vec![config.fri.circle_fold_step];
    all_fold_steps.extend_from_slice(all_line_fold_steps);
    let res = zip_eq(zip_eq(layer_log_sizes, layers), all_fold_steps)
        .map(|((log_size, layer_proof), step)| {
            unsorted_query_locations
                .iter()
                .map(|query| {
                    let mut pos = *query;
                    pos >>= n - log_size + step;
                    let mut auth_path: AuthPath<QM31> = AuthPath(vec![]);
                    for j in step..log_size {
                        let hash = layer_proof.decommitment.all_node_values[j][&(pos ^ 1)];
                        auth_path.0.push(hash.into());
                        pos >>= 1;
                    }
                    auth_path
                })
                .collect()
        })
        .collect();

    AuthPaths { data: res }
}
