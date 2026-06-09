macro_rules! define_component_list {
    ($($variant:ident),* $(,)?) => {
        pub enum ComponentList {
            $($variant),*
        }
        impl ComponentList {
            /// The index of this component, in the static circuit components array.
            pub const fn idx(self) -> usize {
                self as usize
            }
        }
        pub const N_COMPONENTS: usize = [$(stringify!($variant)),*].len();
    };
}

define_component_list! {
    Eq,
    Qm31Ops,
    TripleXor,
    M31ToU32,
    BlakeGGate,
    VerifyBitwiseXor8,
    VerifyBitwiseXor12,
    VerifyBitwiseXor4,
    VerifyBitwiseXor7,
    VerifyBitwiseXor9,
    RangeCheck16,
}
