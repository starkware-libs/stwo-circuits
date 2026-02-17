use crate::circuit_prover::witness::components::prelude::*;

pub type PackedInputType = ([[PackedUInt32; 8]; 2], [PackedM31; 16]);

pub type PackedBlakeMessageLimb = PackedUInt32;

#[derive(Default)]
pub struct ClaimGenerator {
    pub msg_per_id: Vec<[u32; 16]>,
}
impl ClaimGenerator {
    pub fn deduce_output(&self, [id, index]: [PackedM31; 2]) -> PackedBlakeMessageLimb {
        let ids = id.into_simd().to_array();
        let indexes = index.into_simd().to_array();

        let values = from_fn(|lane| {
            let id = ids[lane] as usize;
            let idx = indexes[lane] as usize;
            UInt32::from(self.msg_per_id[id][idx])
        });

        PackedBlakeMessageLimb::from_array(values)
    }

    pub fn add_packed_inputs(&mut self, id: PackedM31, messages: [PackedUInt32; 16]) {
        let ids = id.into_simd().to_array();

        // TODO(alonf): we assume it comes in order, if we want to parallelize need to start a new
        // vec with capacity initialized to 0 when creating the ClaimGenerator and then push
        // to the vec when adding the inputs.
        for (j, _id) in ids.iter().enumerate() {
            let mut vec: [u32; 16] = [0; 16];
            for i in 0..16 {
                let messages_limb_i = messages[i].as_array();
                vec[i] = messages_limb_i[j].value;
            }
            self.msg_per_id.push(vec);
        }
    }
}
