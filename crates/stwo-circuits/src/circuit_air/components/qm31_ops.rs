use stwo::core::channel::Channel;

pub const N_TRACE_COLUMNS: usize = 12;
pub struct Claim {
    pub log_size: u32,
}
impl Claim {
    pub fn mix_into(&self, channel: &mut impl Channel) {
        channel.mix_u64(self.log_size as u64);
    }
}
