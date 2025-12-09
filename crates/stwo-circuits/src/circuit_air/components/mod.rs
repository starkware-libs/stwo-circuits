pub mod qm31_ops;
use stwo::core::channel::Channel;
use stwo::core::pcs::TreeVec;

pub struct CircuitClaim {
    pub qm31_ops: qm31_ops::Claim,
    // ...
}
impl CircuitClaim {
    pub fn mix_into(&self, channel: &mut impl Channel) {
        self.qm31_ops.mix_into(channel);
    }

    /// Returns the log sizes of the components.
    /// Does not include the preprocessed trace log sizes.
    pub fn log_sizes(&self) -> TreeVec<Vec<u32>> {
        let log_sizes_list = vec![self.qm31_ops.log_sizes()];

        TreeVec::concat_cols(log_sizes_list.into_iter())
    }
}
