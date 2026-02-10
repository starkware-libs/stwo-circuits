pub use crate::circuit_air::ClaimedSum;
pub use crate::circuit_air::ComponentLogSize;
pub use crate::circuit_air::relations;
pub use crate::circuit_prover::witness::preprocessed::PreProcessedTrace;
pub use crate::circuit_prover::witness::utils::pack_values;
pub use itertools::Itertools;
pub use itertools::multizip;
pub use num_traits::One;
pub use num_traits::Zero;
pub use rayon::iter::IndexedParallelIterator;
pub use rayon::iter::IntoParallelIterator;
pub use rayon::iter::IntoParallelRefIterator;
pub use rayon::iter::IntoParallelRefMutIterator;
pub use rayon::iter::ParallelIterator;
pub use std::array::from_fn;
pub use std::simd::num::SimdInt;
pub use std::simd::num::SimdUint;
pub use std::simd::u32x16;
pub use std::sync::Arc;
use stwo::core::fields::m31::BaseField;
pub use stwo::core::fields::m31::M31;
pub use stwo::core::fields::qm31::QM31;
pub use stwo::core::fields::qm31::SecureField;
pub use stwo::core::poly::circle::CanonicCoset;
pub use stwo::core::vcs_lifted::blake2_merkle::Blake2sM31MerkleChannel;
pub use stwo::prover::TreeBuilder;
pub use stwo::prover::backend::Col;
pub use stwo::prover::backend::simd::SimdBackend;
pub use stwo::prover::backend::simd::column::BaseColumn;
pub use stwo::prover::backend::simd::m31::{LOG_N_LANES, N_LANES, PackedM31};
pub use stwo::prover::backend::simd::qm31::PackedQM31;
pub use stwo::prover::poly::BitReversedOrder;
pub use stwo::prover::poly::circle::CircleEvaluation;
pub use stwo_air_utils::trace::component_trace::ComponentTrace;
pub use stwo_air_utils_derive::{IterMut, ParIterMut, Uninitialized};
pub use stwo_constraint_framework::LogupTraceGenerator;
pub use stwo_constraint_framework::Relation;
pub use stwo_constraint_framework::preprocessed_columns::PreProcessedColumnId;

pub use serde::{Deserialize, Serialize};
pub use std::mem::transmute;
pub use std::ops::{Add, BitAnd, BitOr, BitXor, Rem, Shl, Shr, Sub};
pub use std::simd::Simd;
pub use std::sync::atomic::AtomicU32;
pub use std::sync::atomic::Ordering;
pub use stwo::prover::backend::simd::conversion::{Pack, Unpack};

#[derive(Copy, Clone, Debug, Default, Eq, PartialEq, Hash, Serialize, Deserialize)]
pub struct UInt16 {
    pub value: u16,
}

impl UInt16 {
    pub fn from_m31(felt: M31) -> Self {
        assert!(felt < M31::from_u32_unchecked(2_u32.pow(16)), "M31 value is not a u16");
        Self { value: felt.0 as u16 }
    }
}

impl Add for UInt16 {
    type Output = UInt16;
    fn add(self, other: UInt16) -> UInt16 {
        UInt16 { value: self.value.wrapping_add(other.value) }
    }
}

impl Sub for UInt16 {
    type Output = UInt16;
    fn sub(self, rhs: UInt16) -> UInt16 {
        UInt16 { value: self.value.wrapping_sub(rhs.value) }
    }
}

impl From<u16> for UInt16 {
    fn from(value: u16) -> UInt16 {
        UInt16 { value }
    }
}

impl Rem for UInt16 {
    type Output = UInt16;
    fn rem(self, other: UInt16) -> UInt16 {
        UInt16 { value: self.value % other.value }
    }
}
impl Shl for UInt16 {
    type Output = UInt16;
    fn shl(self, other: UInt16) -> UInt16 {
        UInt16 { value: self.value << other.value }
    }
}
impl Shr for UInt16 {
    type Output = UInt16;
    fn shr(self, other: UInt16) -> UInt16 {
        UInt16 { value: self.value >> other.value }
    }
}
impl BitAnd for UInt16 {
    type Output = UInt16;
    fn bitand(self, other: UInt16) -> UInt16 {
        UInt16 { value: self.value & other.value }
    }
}
impl BitOr for UInt16 {
    type Output = UInt16;
    fn bitor(self, other: UInt16) -> UInt16 {
        UInt16 { value: self.value | other.value }
    }
}
impl BitXor for UInt16 {
    type Output = UInt16;
    fn bitxor(self, other: UInt16) -> UInt16 {
        UInt16 { value: self.value ^ other.value }
    }
}

#[derive(Copy, Clone, Debug, Default, Eq, PartialEq, Hash, Serialize, Deserialize)]
pub struct UInt32 {
    pub value: u32,
}

impl UInt32 {
    pub fn low(&self) -> UInt16 {
        UInt16 { value: (self.value & 0xFFFF) as u16 }
    }

    pub fn high(&self) -> UInt16 {
        UInt16 { value: (self.value >> 16) as u16 }
    }

    pub fn from_m31(felt: M31) -> Self {
        Self { value: felt.0 }
    }

    pub fn from_limbs(low: M31, high: M31) -> Self {
        Self { value: (low.0 & 0xFFFF) | ((high.0 & 0xFFFF) << 16) }
    }
}

impl From<u32> for UInt32 {
    fn from(value: u32) -> UInt32 {
        UInt32 { value }
    }
}

impl Add for UInt32 {
    type Output = UInt32;
    fn add(self, other: UInt32) -> UInt32 {
        UInt32 { value: self.value.wrapping_add(other.value) }
    }
}
impl Rem for UInt32 {
    type Output = UInt32;
    fn rem(self, other: UInt32) -> UInt32 {
        UInt32 { value: self.value % other.value }
    }
}
impl Shl for UInt32 {
    type Output = UInt32;
    fn shl(self, other: UInt32) -> UInt32 {
        UInt32 { value: self.value << other.value }
    }
}
impl Shr for UInt32 {
    type Output = UInt32;
    fn shr(self, other: UInt32) -> UInt32 {
        UInt32 { value: self.value >> other.value }
    }
}
impl BitAnd for UInt32 {
    type Output = UInt32;
    fn bitand(self, other: UInt32) -> UInt32 {
        UInt32 { value: self.value & other.value }
    }
}
impl BitOr for UInt32 {
    type Output = UInt32;
    fn bitor(self, other: UInt32) -> UInt32 {
        UInt32 { value: self.value | other.value }
    }
}
impl BitXor for UInt32 {
    type Output = UInt32;
    fn bitxor(self, other: UInt32) -> UInt32 {
        UInt32 { value: self.value ^ other.value }
    }
}

pub trait PackedM31Type {
    fn as_m31(&self) -> PackedM31;
}

#[derive(Clone, Copy, Debug)]
pub struct PackedBool {
    pub(crate) value: Simd<i32, N_LANES>,
}
impl PackedBool {
    pub fn from_m31(value: PackedM31) -> Self {
        Self { value: value.into_simd().cast().bitand(Simd::splat(1)) }
    }
}

impl PackedM31Type for PackedBool {
    fn as_m31(&self) -> PackedM31 {
        // Safe.
        unsafe { PackedM31::from_simd_unchecked(self.value.cast()) }
    }
}

impl BitAnd for PackedBool {
    type Output = Self;

    fn bitand(self, rhs: Self) -> Self::Output {
        Self { value: self.value & rhs.value }
    }
}

#[derive(Copy, Clone, Debug, Default)]
pub struct PackedUInt16 {
    value: Simd<u16, N_LANES>,
}

impl PackedUInt16 {
    pub fn broadcast(value: UInt16) -> Self {
        Self { value: Simd::splat(value.value) }
    }
    pub fn from_array(arr: [UInt16; N_LANES]) -> Self {
        // Safe because UInt16 is u16.
        unsafe { Self { value: Simd::from_array(transmute::<[UInt16; 16], [u16; 16]>(arr)) } }
    }
    pub fn as_array(&self) -> [UInt16; N_LANES] {
        // Safe because UInt16 is u16.
        unsafe { transmute(self.value.to_array()) }
    }
    pub fn from_m31(val: PackedM31) -> Self {
        Self { value: val.into_simd().cast() }
    }
}

impl PackedM31Type for PackedUInt16 {
    fn as_m31(&self) -> PackedM31 {
        // Safe.
        unsafe { PackedM31::from_simd_unchecked(self.value.cast()) }
    }
}

impl Add for PackedUInt16 {
    type Output = Self;

    fn add(self, rhs: Self) -> Self::Output {
        Self { value: self.value + rhs.value }
    }
}

impl Rem for PackedUInt16 {
    type Output = Self;

    fn rem(self, rhs: Self) -> Self::Output {
        Self { value: self.value % rhs.value }
    }
}
impl Shl for PackedUInt16 {
    type Output = Self;

    fn shl(self, rhs: Self) -> Self::Output {
        Self { value: self.value << rhs.value }
    }
}
impl Shr for PackedUInt16 {
    type Output = Self;

    fn shr(self, rhs: Self) -> Self::Output {
        Self { value: self.value >> rhs.value }
    }
}
impl BitAnd for PackedUInt16 {
    type Output = Self;

    fn bitand(self, rhs: Self) -> Self::Output {
        Self { value: self.value & rhs.value }
    }
}
impl BitOr for PackedUInt16 {
    type Output = Self;

    fn bitor(self, rhs: Self) -> Self::Output {
        Self { value: self.value | rhs.value }
    }
}
impl BitXor for PackedUInt16 {
    type Output = Self;

    fn bitxor(self, rhs: Self) -> Self::Output {
        Self { value: self.value ^ rhs.value }
    }
}

#[derive(Copy, Clone, Debug, Default, PartialEq)]
pub struct PackedUInt32 {
    pub simd: Simd<u32, N_LANES>,
}

impl PackedUInt32 {
    pub fn broadcast(value: UInt32) -> Self {
        Self { simd: Simd::splat(value.value) }
    }
    pub fn from_array(arr: [UInt32; N_LANES]) -> Self {
        // Safe because UInt32 is u32.
        unsafe { Self { simd: Simd::from_array(transmute::<[UInt32; 16], [u32; 16]>(arr)) } }
    }

    pub fn from_simd(value: Simd<u32, N_LANES>) -> Self {
        Self { simd: value }
    }

    pub fn as_array(&self) -> [UInt32; N_LANES] {
        // Safe because UInt32 is u32.
        unsafe { transmute(self.simd.to_array()) }
    }

    pub fn from_m31(val: PackedM31) -> Self {
        Self { simd: val.into_simd() }
    }

    pub fn low(&self) -> PackedUInt16 {
        PackedUInt16 { value: (self.simd & Simd::splat(0xFFFF)).cast() }
    }

    pub fn high(&self) -> PackedUInt16 {
        PackedUInt16 { value: (self.simd >> 16).cast() }
    }

    pub fn from_limbs([low, high]: [PackedM31; 2]) -> Self {
        let [low, high] = [low, high].map(PackedM31::into_simd);
        Self { simd: low + (high << 16) }
    }
}

impl Rem for PackedUInt32 {
    type Output = Self;

    fn rem(self, rhs: Self) -> Self::Output {
        Self { simd: self.simd % rhs.simd }
    }
}
impl Shl for PackedUInt32 {
    type Output = Self;

    fn shl(self, rhs: Self) -> Self::Output {
        Self { simd: self.simd << rhs.simd }
    }
}
impl Shr for PackedUInt32 {
    type Output = Self;

    fn shr(self, rhs: Self) -> Self::Output {
        Self { simd: self.simd >> rhs.simd }
    }
}
impl BitAnd for PackedUInt32 {
    type Output = Self;

    fn bitand(self, rhs: Self) -> Self::Output {
        Self { simd: self.simd & rhs.simd }
    }
}
impl BitOr for PackedUInt32 {
    type Output = Self;

    fn bitor(self, rhs: Self) -> Self::Output {
        Self { simd: self.simd | rhs.simd }
    }
}
impl BitXor for PackedUInt32 {
    type Output = Self;

    fn bitxor(self, rhs: Self) -> Self::Output {
        Self { simd: self.simd ^ rhs.simd }
    }
}
impl Add for PackedUInt32 {
    type Output = Self;

    fn add(self, rhs: Self) -> Self::Output {
        Self { simd: self.simd + rhs.simd }
    }
}

// unsafe impl Zeroable for PackedUInt32 {
//     fn zeroed() -> Self {
//         Self {
//             simd: unsafe { core::mem::zeroed() },
//         }
//     }
// }

impl Pack for UInt32 {
    type SimdType = PackedUInt32;

    fn pack(inputs: [UInt32; N_LANES]) -> Self::SimdType {
        PackedUInt32::from_array(inputs)
    }
}

impl Unpack for PackedUInt32 {
    type CpuType = UInt32;

    fn unpack(self) -> [Self::CpuType; N_LANES] {
        self.as_array()
    }
}

#[derive(Clone)]
pub struct RelationUse {
    pub relation_id: &'static str,
    pub uses: u64,
}

/// The enabler column is a column of length `padding_offset.next_power_of_two()` where
/// 1. The first `padding_offset` elements are set to 1;
/// 2. The rest are set to 0.
#[derive(Debug, Clone)]
pub struct Enabler {
    pub padding_offset: usize,
}
impl Enabler {
    pub const fn new(padding_offset: usize) -> Self {
        Self { padding_offset }
    }

    pub fn packed_at(&self, vec_row: usize) -> PackedM31 {
        let row_offset = vec_row * N_LANES;
        if self.padding_offset <= row_offset {
            return PackedM31::zero();
        }
        if self.padding_offset >= row_offset + N_LANES {
            return PackedM31::one();
        }

        // The row is partially enabled.
        let mut res = [M31::zero(); N_LANES];
        for v in res.iter_mut().take(self.padding_offset - row_offset) {
            *v = M31::one();
        }
        PackedM31::from_array(res)
    }
}

/// A column of multiplicities for lookup arguments. Allows increasing the multiplicity at a given
/// index. This version uses atomic operations to increase the multiplicity, and is `Send`.
pub struct AtomicMultiplicityColumn {
    data: Vec<AtomicU32>,
}
impl AtomicMultiplicityColumn {
    /// Creates a new `AtomicMultiplicityColumn` with the given size. The elements are initialized
    /// to 0.
    pub fn new(size: usize) -> Self {
        Self { data: (0..size as u32).map(|_| AtomicU32::new(0)).collect() }
    }

    pub fn increase_at(&self, address: u32) {
        self.data[address as usize].fetch_add(1, Ordering::Relaxed);
    }

    /// Returns the internal data as a Vec<PackedM31>. The last element of the vector is padded with
    /// zeros if needed. This function performs a copy on the inner data, If atomics are not
    /// necessary, use [`MultiplicityColumn`] instead.
    pub fn into_simd_vec(self) -> Vec<PackedM31> {
        // Safe because the data is aligned to the size of PackedM31 and the size of the data is a
        // multiple of N_LANES.
        BaseColumn::from_iter(self.data.into_iter().map(|a| M31(a.load(Ordering::Relaxed)))).data
    }
}

pub use std::collections::HashMap;

/// Debug helper: prints each SIMD lane of a logup entry as unpacked M31 values.
/// `label` identifies the component and lookup name.
/// `mult` is the multiplicity for this entry per SIMD lane.
pub fn debug_logup(label: &str, values: &[PackedM31], mult: &[PackedM31]) {
    for lane in 0..N_LANES {
        let v: Vec<u32> = values.iter().map(|v| v.into_simd().to_array()[lane]).collect();
        let m: Vec<u32> = mult.iter().map(|v| v.into_simd().to_array()[lane]).collect();
        // eprintln!("{label}[{lane}]: mult={m:?} values={v:?}");
    }
}

/// Create the input_to_row map used in const-size components.
///
/// `preprocessed_trace` - The preprocessed trace.
/// `column_ids` - PreProcessedColumnId for each input column of the component.
///
/// Returns a mapping from input tuple to its row number. Used to find
/// out which multiplicity value to update for a given input.
pub fn make_input_to_row<const N: usize>(
    preprocessed_trace: &PreProcessedTrace,
    column_ids: [PreProcessedColumnId; N],
) -> HashMap<[M31; N], usize> {
    let mut result: HashMap<[M31; N], usize> = HashMap::new();

    let columns = column_ids.iter().map(|id| preprocessed_trace.get_column(id)).collect_vec();
    let log_size = columns[0].len().ilog2();
    assert!(
        columns.iter().all(|c| c.len().ilog2() == log_size),
        "input_to_row columns of different sizes"
    );

    for packed_row in 0..(1 << (log_size - LOG_N_LANES)) {
        let row_offset = packed_row * N_LANES;
        for i in 0..N_LANES {
            let key: [M31; N] = columns
                .iter()
                .map(|column| M31::from(column[row_offset + i]))
                .collect_vec()
                .try_into()
                .expect("Unexpected number of column values");
            result.insert(key, row_offset + i);
        }
    }

    result
}

pub fn pack_preprocessed_column(column: &[usize]) -> Vec<PackedM31> {
    let values = column.iter().map(|&v| M31::from(v)).collect_vec();
    pack_values(&values)
}

const NUM_INPUT_WORDS_G: usize = 6;
const NUM_OUTPUT_WORDS_G: usize = 4;
pub const G_STATE_INDICES: [[usize; 4]; 8] = [
    [0, 4, 8, 12],
    [1, 5, 9, 13],
    [2, 6, 10, 14],
    [3, 7, 11, 15],
    [0, 5, 10, 15],
    [1, 6, 11, 12],
    [2, 7, 8, 13],
    [3, 4, 9, 14],
];

/// Applies [`u32::rotate_right(N)`] to each element of the vector
#[inline(always)]
fn rotate<const N: u32>(x: u32x16) -> u32x16 {
    (x >> N) | (x << (u32::BITS - N))
}

#[derive(Debug)]
pub struct PackedBlakeG {}

impl PackedBlakeG {
    pub fn deduce_output(
        input: [PackedUInt32; NUM_INPUT_WORDS_G],
    ) -> [PackedUInt32; NUM_OUTPUT_WORDS_G] {
        PackedBlakeG::blake_g(input.map(|x| x.simd)).map(|simd| PackedUInt32 { simd })
    }

    pub fn blake_g(input: [u32x16; NUM_INPUT_WORDS_G]) -> [u32x16; NUM_OUTPUT_WORDS_G] {
        let [mut a, mut b, mut c, mut d, m0, m1] = input;

        a = a + b + m0;
        d ^= a;
        d = rotate::<16>(d);

        c += d;
        b ^= c;
        b = rotate::<12>(b);

        a = a + b + m1;
        d ^= a;
        d = rotate::<8>(d);

        c += d;
        b ^= c;
        b = rotate::<7>(b);

        [a, b, c, d]
    }
}

#[derive(Debug)]
pub struct PackedTripleXor32 {}

impl PackedTripleXor32 {
    pub fn deduce_output([a, b, c]: [PackedUInt32; 3]) -> PackedUInt32 {
        a ^ b ^ c
    }
}

pub const N_BLAKE_ROUNDS: usize = 10;
pub const N_BLAKE_SIGMA_COLS: usize = 16;

pub const BLAKE_SIGMA: [[u32; N_BLAKE_SIGMA_COLS]; N_BLAKE_ROUNDS] = [
    [0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15],
    [14, 10, 4, 8, 9, 15, 13, 6, 1, 12, 0, 2, 11, 7, 5, 3],
    [11, 8, 12, 0, 5, 2, 15, 13, 10, 14, 3, 6, 7, 1, 9, 4],
    [7, 9, 3, 1, 13, 12, 11, 14, 2, 6, 5, 10, 4, 0, 15, 8],
    [9, 0, 5, 7, 2, 4, 10, 15, 14, 1, 11, 12, 6, 8, 3, 13],
    [2, 12, 6, 10, 0, 11, 8, 3, 4, 13, 7, 5, 15, 14, 1, 9],
    [12, 5, 1, 15, 14, 13, 4, 10, 0, 7, 6, 3, 9, 2, 8, 11],
    [13, 11, 7, 14, 12, 1, 3, 9, 5, 0, 15, 4, 8, 6, 2, 10],
    [6, 15, 14, 9, 11, 3, 0, 8, 12, 2, 13, 7, 1, 4, 10, 5],
    [10, 2, 8, 4, 7, 6, 1, 5, 15, 11, 9, 14, 3, 12, 13, 0],
];

pub const BLAKE_SIGMA_TABLE: &str = "blake_sigma";
const LOG_N_ROWS: u32 = (N_BLAKE_ROUNDS as u32).next_power_of_two().ilog2();

pub fn sigma(round: usize) -> [u32; N_BLAKE_SIGMA_COLS] {
    BLAKE_SIGMA[round]
}

pub fn sigma_m31(round: usize, col: usize) -> M31 {
    assert!(col < N_BLAKE_SIGMA_COLS);
    assert!(round < N_BLAKE_ROUNDS);
    (sigma(round)[col]).into()
}

#[derive(Debug)]
pub struct BlakeSigma {
    pub col: usize,
}

impl BlakeSigma {
    pub fn new(col: usize) -> Self {
        Self { col }
    }
}
// Pads all rows below <padding_offset> with the first row. Uses the <get_m31> function to get the
// value in a given row and column.
pub fn pad<F>(get_m31: F, padding_offset: usize, col: usize) -> Vec<M31>
where
    F: Fn(usize, usize) -> M31,
{
    let n = padding_offset.next_power_of_two();
    (0..n).map(|i| if i < padding_offset { i } else { 0 }).map(|i| get_m31(i, col)).collect()
}

impl PreProcessedColumn for BlakeSigma {
    fn log_size(&self) -> u32 {
        LOG_N_ROWS
    }

    // #[cfg(feature = "prover")]
    fn packed_at(&self, vec_row: usize) -> PackedM31 {
        assert!(vec_row == 0, "Accessing BlakeSigma out of bounds row {vec_row}");
        PackedM31::from_array(pad(sigma_m31, N_BLAKE_ROUNDS, self.col).try_into().unwrap())
    }

    // #[cfg(feature = "prover")]
    fn gen_column_simd(&self) -> CircleEvaluation<SimdBackend, BaseField, BitReversedOrder> {
        CircleEvaluation::new(
            CanonicCoset::new(LOG_N_ROWS).circle_domain(),
            BaseColumn::from_iter(pad(sigma_m31, N_BLAKE_ROUNDS, self.col)),
        )
    }

    fn id(&self) -> PreProcessedColumnId {
        PreProcessedColumnId { id: format!("{}_{}", BLAKE_SIGMA_TABLE, self.col) }
    }
}

#[derive(Debug)]
pub struct PackedBlakeRoundSigma {}

impl PackedBlakeRoundSigma {
    pub fn deduce_output(round: PackedM31) -> [PackedM31; N_BLAKE_SIGMA_COLS] {
        Self::packed_sigma(round.into_simd()).map(|v| unsafe { PackedM31::from_simd_unchecked(v) })
    }

    pub fn packed_sigma(round: u32x16) -> [u32x16; N_BLAKE_SIGMA_COLS] {
        from_fn(|i| u32x16::from(round.to_array().map(|x| BLAKE_SIGMA[x as usize][i])))
    }
}

// pub struct BlakeRound {
//     memory: Arc<Memory>,
// }

// impl BlakeRound {
//     pub fn new(memory: Arc<Memory>) -> Self {
//         Self { memory }
//     }
//     pub fn deduce_output(
//         &self,
//         chain: PackedM31,
//         round: PackedM31,
//         (state, message_pointer): ([PackedUInt32; 16], PackedM31),
//     ) -> (PackedM31, PackedM31, ([PackedUInt32; 16], PackedM31)) {
//         let (chain, round, (state, message_pointer)) = self.blake_round(
//             chain.into_simd(),
//             round.into_simd(),
//             (state.map(|x| x.simd), message_pointer.into_simd()),
//         );

//         unsafe {
//             (
//                 PackedM31::from_simd_unchecked(chain),
//                 PackedM31::from_simd_unchecked(round),
//                 (
//                     state.map(|simd| PackedUInt32 { simd }),
//                     PackedM31::from_simd_unchecked(message_pointer),
//                 ),
//             )
//         }
//     }
//     fn blake_round(
//         &self,
//         chain: u32x16,
//         round: u32x16,
//         (state, message_pointer): ([u32x16; 16], u32x16),
//     ) -> (u32x16, u32x16, ([u32x16; 16], u32x16)) {
//         let sigma = PackedBlakeRoundSigma::packed_sigma(round);

//         let message: [_; N_LANES] = from_fn(|i| {
//             u32x16::from(from_fn(|j| {
//                 self.memory.get(message_pointer[j] + sigma[i][j]).as_small() as u32
//             }))
//         });

//         let mut state = state;
//         for (row_index, &[i0, i1, i2, i3]) in G_STATE_INDICES.iter().enumerate() {
//             [state[i0], state[i1], state[i2], state[i3]] = PackedBlakeG::blake_g([
//                 state[i0],
//                 state[i1],
//                 state[i2],
//                 state[i3],
//                 message[row_index * 2],
//                 message[row_index * 2 + 1],
//             ]);
//         }

//         (chain, round + u32x16::splat(1), (state, message_pointer))
//     }
// }
pub const SIMD_ENUMERATION_0: Simd<u32, N_LANES> =
    Simd::from_array([0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15]);

/// A column with the numbers [0..(2^log_size)-1].
#[derive(Debug, Clone)]
pub struct Seq {
    pub log_size: u32,
}
impl Seq {
    pub const fn new(log_size: u32) -> Self {
        Self { log_size }
    }
}
impl PreProcessedColumn for Seq {
    fn log_size(&self) -> u32 {
        self.log_size
    }
    // #[cfg(feature = "prover")]
    fn packed_at(&self, vec_row: usize) -> PackedM31 {
        PackedM31::broadcast(M31::from(vec_row * N_LANES))
            + unsafe { PackedM31::from_simd_unchecked(SIMD_ENUMERATION_0) }
    }
    // #[cfg(feature = "prover")]
    fn gen_column_simd(&self) -> CircleEvaluation<SimdBackend, BaseField, BitReversedOrder> {
        let col = Col::<SimdBackend, BaseField>::from_iter(
            (0..(1 << self.log_size)).map(BaseField::from),
        );
        CircleEvaluation::new(CanonicCoset::new(self.log_size).circle_domain(), col)
    }

    fn id(&self) -> PreProcessedColumnId {
        PreProcessedColumnId { id: format!("seq_{}", self.log_size) }
    }
}

pub trait PreProcessedColumn: Send + Sync {
    // #[cfg(feature = "prover")]
    fn packed_at(&self, vec_row: usize) -> PackedM31;
    fn log_size(&self) -> u32;
    fn id(&self) -> PreProcessedColumnId;
    // #[cfg(feature = "prover")]
    fn gen_column_simd(&self) -> CircleEvaluation<SimdBackend, BaseField, BitReversedOrder>;
}

// #[derive(Debug)]
// pub struct PackedBlakeG {}

// impl PackedBlakeG {
//     pub fn deduce_output(
//         input: [PackedUInt32; NUM_INPUT_WORDS_G],
//     ) -> [PackedUInt32; NUM_OUTPUT_WORDS_G] {
//         PackedBlakeG::blake_g(input.map(|x| x.simd)).map(|simd| PackedUInt32 { simd })
//     }

//     fn blake_g(input: [u32x16; NUM_INPUT_WORDS_G]) -> [u32x16; NUM_OUTPUT_WORDS_G] {
//         let [mut a, mut b, mut c, mut d, m0, m1] = input;

//         a = a + b + m0;
//         d ^= a;
//         d = rotate::<16>(d);

//         c += d;
//         b ^= c;
//         b = rotate::<12>(b);

//         a = a + b + m1;
//         d ^= a;
//         d = rotate::<8>(d);

//         c += d;
//         b ^= c;
//         b = rotate::<7>(b);

//         [a, b, c, d]
//     }
// }
