pub mod components;
pub mod relations;

use crate::circuit_air::components::{ComponentList, N_COMPONENTS, eq, qm31_ops};
use num_traits::Zero;
use stwo::core::channel::Channel;
use stwo::core::fields::qm31::SecureField;
use stwo::core::pcs::TreeVec;

pub type ComponentLogSize = u32;
pub type ClaimedSum = SecureField;

pub struct CircuitClaim {
    pub log_sizes: [ComponentLogSize; N_COMPONENTS],
}
impl CircuitClaim {
    pub fn mix_into(&self, channel: &mut impl Channel) {
        for log_size in self.log_sizes {
            channel.mix_u64(log_size as u64);
        }
    }

    /// Returns the log sizes of the components.
    /// Does not include the preprocessed trace log sizes.
    pub fn log_sizes(&self) -> TreeVec<Vec<u32>> {
        let log_sizes_list = vec![
            qm31_ops::log_sizes(self.log_sizes[ComponentList::Qm31Ops as usize]),
            eq::log_sizes(self.log_sizes[ComponentList::Eq as usize]),
        ];

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
    pub claimed_sums: [ClaimedSum; N_COMPONENTS],
}
impl CircuitInteractionClaim {
    pub fn mix_into(&self, channel: &mut impl Channel) {
        for claimed_sum in self.claimed_sums {
            channel.mix_felts(&[claimed_sum]);
        }
    }
}

pub fn lookup_sum(interaction_claim: &CircuitInteractionClaim) -> SecureField {
    let mut sum = SecureField::zero();
    for claimed_sum in interaction_claim.claimed_sums {
        sum += claimed_sum;
    }
    sum
}
