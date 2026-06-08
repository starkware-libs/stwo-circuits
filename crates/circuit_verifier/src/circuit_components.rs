macro_rules! define_component_list {
    ($($variant:ident => $name:literal),* $(,)?) => {
        pub enum ComponentList {
            $($variant),*
        }

        pub const N_COMPONENTS: usize = [$(stringify!($variant)),*].len();
        /// Canonical component names, in `ComponentList` order. These are the keys used for
        /// the per-component log-size map and match the keys of `all_circuit_components`.
        pub const COMPONENT_NAMES: [&str; N_COMPONENTS] = [$($name),*];

        impl ComponentList {
            /// The index of this component, in the static circuit components array.
            pub const fn idx(self) -> usize {
                self as usize
            }

            /// The canonical name of this component, used as the key in the per-component
            /// log-size map.
            pub const fn name(self) -> &'static str {
                match self {
                    $(Self::$variant => $name),*
                }
            }
        }
    };
}

define_component_list! {
    Eq => "eq",
    Qm31Ops => "qm31_ops",
    TripleXor => "triple_xor",
    M31ToU32 => "m_31_to_u_32",
    BlakeGGate => "blake_g_gate",
    VerifyBitwiseXor8 => "verify_bitwise_xor_8",
    VerifyBitwiseXor12 => "verify_bitwise_xor_12",
    VerifyBitwiseXor4 => "verify_bitwise_xor_4",
    VerifyBitwiseXor7 => "verify_bitwise_xor_7",
    VerifyBitwiseXor9 => "verify_bitwise_xor_9",
    RangeCheck16 => "range_check_16",
}

/// Returns the component indices ordered by ascending trace log size, with ties broken by the
/// component's position in [`ComponentList`] (the sort is stable).
///
/// This mirrors the order in which stwo commits trace/interaction columns to its Merkle tree
/// (stwo sorts columns by length with a stable sort; see
/// `stwo::prover::vcs_lifted::prover::MerkleProverLifted::commit`). Committing and iterating the
/// circuit components in this order makes the committed column layout coincide with the natural
/// component order, which lets the verifier set `Statement::sorting_required` to `false` and skip
/// the in-circuit query-column sort during decommitment.
///
/// `log_sizes` is keyed by canonical component name (see [`COMPONENT_NAMES`]); the returned indices
/// are into [`ComponentList`] order. The same `log_sizes` must be used by the prover and the
/// verifier so they derive an identical permutation.
pub fn sorted_component_order(
    log_sizes: &circuits_stark_verifier::order_hash_map::OrderedHashMap<&'static str, u32>,
) -> [usize; N_COMPONENTS] {
    // Look up each component's log size by its canonical name, indexed by `ComponentList`.
    let sizes: [u32; N_COMPONENTS] = std::array::from_fn(|i| log_sizes[COMPONENT_NAMES[i]]);
    let mut order: [usize; N_COMPONENTS] = std::array::from_fn(|i| i);
    // `sort_by_key` is stable, so equal log sizes keep their `ComponentList` order, matching the
    // (log_size, column_index) tie-break used by the verifier's query sorter.
    order.sort_by_key(|&i| sizes[i]);
    order
}
