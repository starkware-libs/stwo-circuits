pub mod qm31_ops;
use stwo::core::channel::Channel;

pub struct CircuitClaim {
    pub qm31_ops: qm31_ops::Claim,
}
impl CircuitClaim {
    pub fn mix_into(&self, channel: &mut impl Channel) {
        let Self { qm31_ops } = self;
        qm31_ops.mix_into(channel);
    }
}
