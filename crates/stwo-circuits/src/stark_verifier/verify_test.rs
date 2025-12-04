use num_traits::One;
use rstest::rstest;
use stwo::core::fields::qm31::QM31;
use stwo::core::vcs::blake2_hash::Blake2sHash;

use crate::circuits::context::{Context, TraceContext};
use crate::circuits::ivalue::NoValue;
use crate::circuits::ops::Guess;
use crate::examples::simple_air::{LOG_N_INSTANCES, create_proof};
use crate::examples::simple_statement::SimpleStatement;
use crate::stark_verifier::fri_proof::FriConfig;
use crate::stark_verifier::proof::{ProofConfig, empty_proof};
use crate::stark_verifier::proof_from_stark_proof::proof_from_stark_proof;
use crate::stark_verifier::verify::verify;

enum ProofModifier {
    /// Keep the proof unchanged.
    None,
    /// Modify an element of the authentication path for one of the traces.
    WrongTraceAuthPath,
    /// Modify an element of the first layer Merkle authentication path (decommitment).
    WrongFriAuthPath,
    /// Modify the siblings in the last inner layer of FRI.
    WrongFriSibling,
}

#[rstest]
#[case::success(ProofModifier::None)]
#[case::wrong_trace_auth_path(ProofModifier::WrongTraceAuthPath)]
#[case::wrong_fri_auth_path(ProofModifier::WrongFriAuthPath)]
#[case::wrong_fri_sibling(ProofModifier::WrongFriSibling)]
fn test_verify(#[case] proof_modifier: ProofModifier) {
    let config = ProofConfig {
        n_proof_of_work_bits: 10,
        n_preprocessed_columns: 1,
        n_trace_columns: 4,
        n_interaction_columns: 4,
        n_components: 1,
        cumulative_sum_columns: vec![true; 4],
        fri: FriConfig {
            log_trace_size: LOG_N_INSTANCES.try_into().unwrap(),
            log_blowup_factor: 1,
            n_queries: 3,
            log_n_last_layer_coefs: 0,
        },
    };

    // Create a NoValue version.
    let novalue_circuit = {
        let empty_proof = empty_proof(&config);
        let mut novalue_context = Context::<NoValue>::default();
        let proof_vars = empty_proof.guess(&mut novalue_context);
        verify(&mut novalue_context, &proof_vars, &config, &SimpleStatement {});
        novalue_context.circuit
    };

    // Create a context with values from the proof.
    let (component, mut proof) = create_proof();
    match proof_modifier {
        ProofModifier::None => {}
        ProofModifier::WrongTraceAuthPath => {
            let first_query = proof.aux.unsorted_query_locations[0];
            // `trace_decommitment[1]` refers to the main trace.
            let first_layer_values = &mut proof.aux.trace_decommitment[1].all_node_values[1];
            let value: &mut Blake2sHash = first_layer_values.get_mut(&(first_query ^ 1)).unwrap();
            value.0[0] ^= 1;
        }
        ProofModifier::WrongFriAuthPath => {
            let first_query = proof.aux.unsorted_query_locations[0];
            let first_layer_values = &mut proof.aux.fri.first_layer.decommitment.all_node_values[2];
            let value: &mut Blake2sHash =
                first_layer_values.get_mut(&((first_query >> 1) ^ 1)).unwrap();
            value.0[0] ^= 1;
        }
        ProofModifier::WrongFriSibling => {
            let values = &mut proof.aux.fri.inner_layers.last_mut().unwrap().all_values[0];
            for (_, value) in values.iter_mut() {
                *value += QM31::one();
            }
        }
    }
    let mut context = TraceContext::default();
    let claimed_sums = vec![component.claimed_sum()];
    let proof = proof_from_stark_proof(&proof, &config, claimed_sums);
    let proof_vars = proof.guess(&mut context);
    verify(&mut context, &proof_vars, &config, &SimpleStatement {});

    // Make sure we got the same circuit.
    assert_eq!(context.circuit, novalue_circuit);

    let result = novalue_circuit.check(context.values());
    match proof_modifier {
        ProofModifier::None => {
            result.unwrap();
        }
        ProofModifier::WrongTraceAuthPath => {
            let err = result.unwrap_err();
            // The error should be when comparing the main trace root.
            let expected_value = context.get(proof_vars.trace_root.0);
            assert!(err.contains(&expected_value.to_string()));
        }
        ProofModifier::WrongFriAuthPath => {
            let err = result.unwrap_err();
            // The error should be when comparing the first layer Merkle root.
            let expected_value = context.get(proof_vars.fri.commit.layer_commitments[0].0);
            assert!(err.contains(&expected_value.to_string()));
        }
        ProofModifier::WrongFriSibling => {
            let err = result.unwrap_err();
            // The error should be when comparing the last layer Merkle root.
            let expected_value =
                context.get(proof_vars.fri.commit.layer_commitments.last().unwrap().0);
            assert!(err.contains(&expected_value.to_string()));
        }
    }

    novalue_circuit.check_yields();
    context.check_vars_used();
    println!("Stats: {:?}", context.stats);
}
