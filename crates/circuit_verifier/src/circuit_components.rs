macro_rules! define_component_list {
    ($($variant:ident),* $(,)?) => {
        pub enum ComponentList {
            $($variant),*
        }
        pub const N_COMPONENTS: usize = [$(stringify!($variant)),*].len();
    };
}

define_component_list! {
    Eq,
    Qm31Ops,
    BlakeGate,
    BlakeRound,
    BlakeRoundSigma,
    BlakeG,
    BlakeOutput,
    TripleXor32,
    TripleXor,
    M31ToU32,
    VerifyBitwiseXor8,
    VerifyBitwiseXor12,
    VerifyBitwiseXor4,
    VerifyBitwiseXor7,
    VerifyBitwiseXor9,
    RangeCheck15,
    RangeCheck16,
}
