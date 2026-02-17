#![allow(non_camel_case_types)]
use stwo_constraint_framework::relation;

use stwo::core::fields::m31::M31;

pub const GATE_RELATION_ID: M31 = M31::from_u32_unchecked(0);

// The number of lookup elements computed. These are used by all relations, so the size
// should be at least the size of the largest relation.
const COMMON_LOOKUP_ELEMENTS_SIZE: usize = 128;
relation!(CommonLookupElements, COMMON_LOOKUP_ELEMENTS_SIZE);
