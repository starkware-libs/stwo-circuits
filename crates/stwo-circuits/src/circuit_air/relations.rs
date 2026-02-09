#![allow(non_camel_case_types)]
use stwo_constraint_framework::relation;

use stwo::core::fields::m31::M31;

pub const GATE_RELATION_ID: M31 = M31::from_u32_unchecked(0);
pub const VERIFY_BITWISE_XOR_12_RELATION_ID: M31 = M31::from_u32_unchecked(1); // Done
pub const VERIFY_BITWISE_XOR_4_RELATION_ID: M31 = M31::from_u32_unchecked(2); 
pub const VERIFY_BITWISE_XOR_7_RELATION_ID: M31 = M31::from_u32_unchecked(3); // Done
pub const VERIFY_BITWISE_XOR_8_RELATION_ID: M31 = M31::from_u32_unchecked(4); // Done
pub const VERIFY_BITWISE_XOR_9_RELATION_ID: M31 = M31::from_u32_unchecked(5);
pub const BLAKE_MESSAGE_RELATION_ID: M31 = M31::from_u32_unchecked(1492981981);

// The number of lookup elements computed. These are used by all relations, so the size
// should be at least the size of the largest relation.
const COMMON_LOOKUP_ELEMENTS_SIZE: usize = 128;
relation!(CommonLookupElements, COMMON_LOOKUP_ELEMENTS_SIZE);
