pub mod components;
pub mod relations;

use crate::circuit_air::components::{eq, qm31_ops};
use num_traits::Zero;
use stwo::core::channel::Channel;
use stwo::core::fields::qm31::SecureField;
use stwo::core::pcs::TreeVec;

pub struct CircuitClaim {
    pub qm31_ops_log_size: u32,
    pub eq_log_size: u32,
    // ...
}
impl CircuitClaim {
    pub fn mix_into(&self, channel: &mut impl Channel) {
        channel.mix_u64(self.qm31_ops_log_size as u64);
        channel.mix_u64(self.eq_log_size as u64);
    }

    /// Returns the log sizes of the components.
    /// Does not include the preprocessed trace log sizes.
    pub fn log_sizes(&self) -> TreeVec<Vec<u32>> {
        let log_sizes_list =
            vec![qm31_ops::log_sizes(self.qm31_ops_log_size), eq::log_sizes(self.eq_log_size)];

        TreeVec::concat_cols(log_sizes_list.into_iter())
    }
}

pub struct CircuitInteractionElements {
    pub gate: relations::Gate,
}
impl CircuitInteractionElements {
    pub fn draw(channel: &mut impl Channel) -> CircuitInteractionElements {
        CircuitInteractionElements { gate: relations::Gate::draw(channel) }
    }
}

pub struct CircuitInteractionClaim {
    pub qm31_ops_claimed_sum: SecureField,
    pub eq_claimed_sum: SecureField,
    // ...
}
impl CircuitInteractionClaim {
    pub fn mix_into(&self, channel: &mut impl Channel) {
        channel.mix_felts(&[self.qm31_ops_claimed_sum]);
        channel.mix_felts(&[self.eq_claimed_sum]);
    }
}

pub fn lookup_sum(interaction_claim: &CircuitInteractionClaim) -> SecureField {
    let mut sum = SecureField::zero();

    sum += interaction_claim.qm31_ops_claimed_sum;
    sum += interaction_claim.eq_claimed_sum;

    sum
}
