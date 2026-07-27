//! A circuit implementation of the Cairo dict API (`starkware/cairo/common/dict.cairo`).
//!
//! A [Dict] records its accesses as `(key, prev_value, new_value)` triples, exactly like Cairo's
//! `DictAccess` segment. Reads and writes are unconstrained when recorded (the previous value is
//! guessed by the prover); all the soundness comes from [Dict::squash], which verifies the whole
//! access log at once:
//!
//! 1. Each access is packed into a single `QM31` as `prev + new * i + (key * 2^idx_bits + idx) * u
//!    + key * iu`, where `idx` is the access index.
//! 2. A [Permutation](crate::circuit::Permutation) gate sorts the packed accesses by the `u`
//!    coordinate, i.e. by `(key, access index)`. The sorted values are unpacked into guessed `M31`
//!    coordinates (the `QM31` coordinate decomposition is unique, so an equality constraint on the
//!    recombination binds the guesses), the keys are range-checked, and range checks on the sort
//!    key differences enforce the sorted order (see `QuerySorter` in the stark verifier for the
//!    same technique).
//! 3. For each pair of adjacent sorted accesses with equal keys, the second access's `prev_value`
//!    must equal the first one's `new_value` (the "chain" condition).
//! 4. The squashed dict — one `(key, prev_value, new_value)` per key, summarizing the first and
//!    last value of the key — is guessed, and verified against the first/last access of each key
//!    group using two more permutation gates.
//!
//! Like in Cairo, the initial values (the `prev_value`s of the squashed dict) are *not*
//! constrained; the caller is responsible for verifying them (e.g. asserting that they all equal a
//! default value).
//!
//! Keys and values must be `M31` values (the [M31Wrapper] contract); keys must additionally be
//! smaller than `2^(30 - ceil(log2(n_accesses)))`, which is enforced in-circuit by a range check.

use itertools::{Itertools, zip_eq};

use crate::context::{Context, Var};
use crate::eval;
use crate::extract_bits::extract_bits;
use crate::ivalue::{IValue, qm31_from_u32s};
use crate::ops::{eq, from_partial_evals, guess, guess_m31, mul, permute, sub};
use crate::simd::Simd;
use crate::wrappers::M31Wrapper;

#[cfg(test)]
#[path = "dict_test.rs"]
pub mod test;

/// Number of bits of a sort key `key * 2^idx_bits + access_idx`. Must be at most 30 for the
/// sortedness range check to be sound: a descending pair of sort keys yields a difference of at
/// least `P - (2^30 - 1) = 2^30`, which fails a 30-bit range check.
const SORT_KEY_BITS: u32 = 30;

/// A single dict access, analogous to Cairo's `DictAccess` struct.
#[derive(Clone, Debug)]
pub struct DictAccess {
    pub key: M31Wrapper<Var>,
    pub prev_value: M31Wrapper<Var>,
    pub new_value: M31Wrapper<Var>,
}

/// A dictionary of `M31` keys and values, verified by [Dict::squash].
///
/// Note that until `squash` is called, nothing constrains the recorded accesses.
pub struct Dict<Value: IValue> {
    /// Witness only: the value returned when reading a key that was never written.
    default_value: Value,
    accesses: Vec<DictAccess>,
}

impl<Value: IValue> Dict<Value> {
    /// Creates a new dict whose unwritten keys hold `default_value`.
    pub fn new(default_value: Value) -> Self {
        Self { default_value, accesses: vec![] }
    }

    /// Witness only: the current value of `key`, i.e. the `new_value` of the last access with
    /// this key, or the default value if there is none.
    fn current_value(&self, context: &Context<Value>, key: &M31Wrapper<Var>) -> Value {
        self.accesses
            .iter()
            .rev()
            .find(|access| context.get(*access.key.get()) == context.get(*key.get()))
            .map(|access| context.get(*access.new_value.get()))
            .unwrap_or(self.default_value)
    }

    /// Reads the current value of `key` (guessed by the prover, verified at [Self::squash]).
    pub fn read(&mut self, context: &mut Context<Value>, key: M31Wrapper<Var>) -> M31Wrapper<Var> {
        let value = self.current_value(context, &key);
        let value = guess_m31(context, M31Wrapper::new_unsafe(value));
        self.accesses.push(DictAccess { key, prev_value: value.clone(), new_value: value.clone() });
        value
    }

    /// Writes `new_value` to `key`, overriding the existing value.
    pub fn write(
        &mut self,
        context: &mut Context<Value>,
        key: M31Wrapper<Var>,
        new_value: M31Wrapper<Var>,
    ) {
        let prev_value = self.current_value(context, &key);
        let prev_value = guess_m31(context, M31Wrapper::new_unsafe(prev_value));
        self.accesses.push(DictAccess { key, prev_value, new_value });
    }

    /// Updates a value in the dict. `prev_value` must be the current value of `key` — otherwise
    /// the circuit becomes unsatisfiable at [Self::squash]. A standalone read with no write can
    /// be performed by passing the same value for both.
    pub fn update(
        &mut self,
        key: M31Wrapper<Var>,
        prev_value: M31Wrapper<Var>,
        new_value: M31Wrapper<Var>,
    ) {
        self.accesses.push(DictAccess { key, prev_value, new_value });
    }

    /// Verifies the access log and returns the squashed dict: one [DictAccess] per key, sorted by
    /// key, whose `prev_value` is the value before the first access of the key and whose
    /// `new_value` is the value after the last access.
    ///
    /// The returned `prev_value`s are unconstrained (like in Cairo's `squash_dict`); the caller
    /// is responsible for verifying the initial values.
    ///
    /// `n_keys` is the number of distinct keys. It determines the circuit topology and therefore
    /// must be supplied by the caller (it cannot be derived in a `NoValue` context); a mismatch
    /// with the actual accesses makes the circuit unsatisfiable.
    pub fn squash(self, context: &mut Context<Value>, n_keys: usize) -> Vec<DictAccess> {
        let n = self.accesses.len();
        if n == 0 {
            assert_eq!(n_keys, 0, "A dict with no accesses has no keys");
            return vec![];
        }
        assert!((1..=n).contains(&n_keys), "n_keys must be in [1, n_accesses]");

        let idx_bits = n.next_power_of_two().ilog2();
        assert!(idx_bits < SORT_KEY_BITS, "Too many accesses");
        let key_bits = SORT_KEY_BITS - idx_bits;

        // Tag each access with its packed representation
        //   `prev + new * i + (key * 2^idx_bits + idx) * u + key * iu`,
        // so that sorting by the `u` coordinate sorts by `(key, access index)`.
        let i = context.constant(qm31_from_u32s(0, 1, 0, 0));
        // Contributes `key * 2^idx_bits` to the `u` coordinate and `key` to the `iu` coordinate.
        let key_coef = context.constant(qm31_from_u32s(0, 0, 1 << idx_bits, 1));
        let tagged = self
            .accesses
            .iter()
            .enumerate()
            .map(|(idx, access)| {
                let idx_tag = context.constant(qm31_from_u32s(0, 0, idx as u32, 0));
                eval!(
                    context,
                    (((*access.prev_value.get()) + ((*access.new_value.get()) * (i)))
                        + ((*access.key.get()) * (key_coef)))
                        + (idx_tag)
                )
            })
            .collect_vec();

        let sorted = permute(context, &tagged, IValue::sort_by_u_coordinate);

        // Unpack each sorted value into four guessed M31 coordinates. The coordinate
        // decomposition of a QM31 into M31 values is unique, so the recombination equality binds
        // the guesses to the coordinates of the sorted value.
        let sorted_accesses = sorted
            .iter()
            .map(|&var| {
                let [prev_value, new_value, sort_key, key] = coordinate_values(context, var)
                    .map(|value| guess_m31(context, M31Wrapper::new_unsafe(value)));
                let recombined = from_partial_evals(
                    context,
                    [*prev_value.get(), *new_value.get(), *sort_key.get(), *key.get()],
                );
                eq(context, var, recombined);
                SortedAccess { prev_value, new_value, sort_key, key }
            })
            .collect_vec();

        // Range-check the keys to `key_bits` bits. This bounds the sort keys by `2^30` (making
        // the sortedness check below sound) and guarantees that `key * 2^idx_bits + idx` does not
        // wrap around, so that equal keys are adjacent in the sorted order.
        let keys = sorted_accesses.iter().map(|access| access.key.clone()).collect_vec();
        let packed_keys = Simd::pack(context, &keys);
        extract_bits(context, &packed_keys, key_bits);

        // Verify that the sort keys are ascending. The sort keys are distinct (the access
        // indices are distinct) and lie in `[0, 2^30)`, so 30-bit range checks on the
        // differences enforce a strictly ascending order.
        if n > 1 {
            let diffs = sorted_accesses
                .windows(2)
                .map(|pair| {
                    let diff = sub(context, *pair[1].sort_key.get(), *pair[0].sort_key.get());
                    M31Wrapper::new_unsafe(diff)
                })
                .collect_vec();
            let packed_diffs = Simd::pack(context, &diffs);
            extract_bits(context, &packed_diffs, SORT_KEY_BITS);
        }

        // For each pair of adjacent sorted accesses, compute a `same_key` indicator and enforce
        // the chain condition: within a key group, each access's `prev_value` equals the previous
        // access's `new_value`.
        let one = context.one();
        let zero = context.zero();
        // is_first[j] == 1 iff sorted access j is the first access of its key.
        let mut is_first = vec![one];
        for j in 1..n {
            let key_diff = eval!(
                context,
                (*sorted_accesses[j].key.get()) - (*sorted_accesses[j - 1].key.get())
            );
            // `same_key = 1 - key_diff * key_diff_inv` together with `same_key * key_diff = 0`
            // forces `same_key` to be exactly the indicator of `key_diff == 0`: if the keys are
            // equal then `same_key = 1` by construction, and otherwise `key_diff` is invertible,
            // forcing `same_key = 0`.
            let key_diff_inv_value = context.get(key_diff).pointwise_inv_or_zero();
            let key_diff_inv = guess(context, key_diff_inv_value);
            let same_key = eval!(context, (one) - ((key_diff) * (key_diff_inv)));
            let same_key_times_key_diff = mul(context, same_key, key_diff);
            eq(context, same_key_times_key_diff, zero);

            let prev_diff = eval!(
                context,
                (*sorted_accesses[j].prev_value.get()) - (*sorted_accesses[j - 1].new_value.get())
            );
            let chain = mul(context, same_key, prev_diff);
            eq(context, chain, zero);

            is_first.push(eval!(context, (one) - (same_key)));
        }
        // is_last[j] == 1 iff sorted access j is the last access of its key.
        let is_last = is_first.iter().skip(1).copied().chain([one]).collect_vec();

        // Compute the squashed entries (witness only). In a `NoValue` context all values compare
        // equal, so the scan sees a single group; resizing to `n_keys` keeps the circuit topology
        // independent of the values.
        let mut squashed_values: Vec<[Value; 3]> = vec![];
        for (j, access) in sorted_accesses.iter().enumerate() {
            let key_value = context.get(*access.key.get());
            let new_value = context.get(*access.new_value.get());
            if j == 0 || key_value != context.get(*sorted_accesses[j - 1].key.get()) {
                let prev_value = context.get(*access.prev_value.get());
                squashed_values.push([key_value, prev_value, new_value]);
            } else {
                squashed_values.last_mut().unwrap()[2] = new_value;
            }
        }
        // With concrete values, a group count different from `n_keys` is a caller bug. A single
        // group is indistinguishable from the `NoValue` case, so in that case a mismatch is
        // caught by the (unsatisfiable) circuit instead.
        if squashed_values.len() != 1 {
            assert_eq!(squashed_values.len(), n_keys, "n_keys does not match the accesses");
        }
        squashed_values.resize(n_keys, [Value::placeholder(); 3]);

        let squashed = squashed_values
            .iter()
            .map(|[key, prev_value, new_value]| DictAccess {
                key: guess_m31(context, M31Wrapper::new_unsafe(*key)),
                prev_value: guess_m31(context, M31Wrapper::new_unsafe(*prev_value)),
                new_value: guess_m31(context, M31Wrapper::new_unsafe(*new_value)),
            })
            .collect_vec();

        // The multiset of `(key, prev_value)` pairs of first accesses must equal the squashed
        // `(key, prev_value)` pairs, and similarly for last accesses and `new_value`s. Group
        // keys are distinct (equal keys are adjacent and groups are maximal), so this matches
        // each squashed entry with exactly one key group.
        let first_entries = zip_eq(&is_first, &sorted_accesses)
            .map(|(selector, access)| (*selector, access.key.clone(), access.prev_value.clone()))
            .collect_vec();
        let squashed_prev_pairs = squashed
            .iter()
            .map(|access| (access.key.clone(), access.prev_value.clone()))
            .collect_vec();
        constrain_gathered_pairs(context, &first_entries, &squashed_prev_pairs);

        let last_entries = zip_eq(&is_last, &sorted_accesses)
            .map(|(selector, access)| (*selector, access.key.clone(), access.new_value.clone()))
            .collect_vec();
        let squashed_new_pairs = squashed
            .iter()
            .map(|access| (access.key.clone(), access.new_value.clone()))
            .collect_vec();
        constrain_gathered_pairs(context, &last_entries, &squashed_new_pairs);

        // Canonicalize the squashed dict order: keys strictly ascending. The squashed keys are
        // distinct and range-checked to `key_bits` bits (via the multiset checks above), so
        // `key_bits`-bit range checks on the differences suffice.
        if n_keys > 1 {
            let diffs = squashed
                .windows(2)
                .map(|pair| {
                    let diff = sub(context, *pair[1].key.get(), *pair[0].key.get());
                    M31Wrapper::new_unsafe(diff)
                })
                .collect_vec();
            let packed_diffs = Simd::pack(context, &diffs);
            extract_bits(context, &packed_diffs, key_bits);
        }

        squashed
    }
}

/// A sorted access, unpacked into its four guessed M31 coordinates.
struct SortedAccess {
    prev_value: M31Wrapper<Var>,
    new_value: M31Wrapper<Var>,
    sort_key: M31Wrapper<Var>,
    key: M31Wrapper<Var>,
}

/// Witness only: the four `M31` coordinates of the value of `var`, each embedded in the first
/// coordinate.
fn coordinate_values<Value: IValue>(context: &Context<Value>, var: Var) -> [Value; 4] {
    let unit_vecs = [
        qm31_from_u32s(1, 0, 0, 0),
        qm31_from_u32s(0, 1, 0, 0),
        qm31_from_u32s(0, 0, 1, 0),
        qm31_from_u32s(0, 0, 0, 1),
    ];
    let value = context.get(var);
    // Mask the coordinate, then divide by the unit vector to move it to the first coordinate.
    unit_vecs
        .map(|unit| Value::pointwise_mul(value, Value::from_qm31(unit)) / Value::from_qm31(unit))
}

/// Constrains the multiset of `(key, value)` pairs of `entries` whose selector is 1 to equal the
/// multiset of `gathered` pairs.
///
/// Each pair is packed as `key + value * i`; entries with selector 0 are replaced by the dummy
/// value `u`, which cannot collide with a packed pair (whose `u` coordinate is 0). A permutation
/// gate then matches the packed entries against the packed `gathered` pairs padded with dummies.
///
/// Assumes the selectors are constrained to be bits and that keys and values are `M31`.
fn constrain_gathered_pairs<Value: IValue>(
    context: &mut Context<Value>,
    entries: &[(Var, M31Wrapper<Var>, M31Wrapper<Var>)],
    gathered: &[(M31Wrapper<Var>, M31Wrapper<Var>)],
) {
    assert!(gathered.len() <= entries.len());
    let i = context.constant(qm31_from_u32s(0, 1, 0, 0));
    let dummy = context.u();

    let inputs = entries
        .iter()
        .map(|(selector, key, value)| {
            let packed = eval!(context, (*key.get()) + ((*value.get()) * (i)));
            eval!(context, (dummy) + ((*selector) * ((packed) - (dummy))))
        })
        .collect_vec();

    // Witness for the permutation outputs: the packed gathered pairs, followed by dummies.
    let i_value = Value::from_qm31(qm31_from_u32s(0, 1, 0, 0));
    let dummy_value = Value::from_qm31(qm31_from_u32s(0, 0, 1, 0));
    let output_values = gathered
        .iter()
        .map(|(key, value)| context.get(*key.get()) + context.get(*value.get()) * i_value)
        .chain(std::iter::repeat_n(dummy_value, entries.len() - gathered.len()))
        .collect_vec();
    let outputs = permute(context, &inputs, move |_| output_values);

    for (idx, out) in outputs.iter().enumerate() {
        if let Some((key, value)) = gathered.get(idx) {
            let packed = eval!(context, (*key.get()) + ((*value.get()) * (i)));
            eq(context, *out, packed);
        } else {
            eq(context, *out, dummy);
        }
    }
}
