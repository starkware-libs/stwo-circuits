use crate::circuits::blake::{HashValue, blake};
use crate::circuits::context::{Context, Var};
use crate::circuits::ivalue::IValue;

#[cfg(test)]
#[path = "channel_test.rs"]
pub mod test;

pub struct Channel {
    /// The current digest of the channel.
    digest: HashValue<Var>,
    /// The number of times values were taken from the channel.
    n_draws: usize,
}

impl Channel {
    /// Constructs a new channel, with a zero digest.
    pub fn new(context: &mut Context<impl IValue>) -> Self {
        let zero = context.zero();
        Self { digest: HashValue(zero, zero), n_draws: 0 }
    }

    /// Updates the digest with the given new digest, and resets the number of draws to zero.
    fn update_digest(&mut self, new_digest: HashValue<Var>) {
        self.digest = new_digest;
        self.n_draws = 0;
    }

    #[cfg(test)]
    pub fn digest(&self) -> HashValue<Var> {
        self.digest
    }

    /// Mixes the given root into the channel's digest.
    pub fn mix_root(&mut self, context: &mut Context<impl IValue>, root: HashValue<Var>) {
        self.update_digest(blake(context, &[self.digest.0, self.digest.1, root.0, root.1], 16 * 4));
    }
}
