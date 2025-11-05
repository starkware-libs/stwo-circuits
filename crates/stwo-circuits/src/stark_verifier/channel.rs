use stwo::core::circle::CirclePoint;

use crate::circuits::blake::{HashValue, blake};
use crate::circuits::context::{Context, Var};
use crate::circuits::ivalue::{IValue, qm31_from_u32s};
use crate::circuits::ops::div;
use crate::eval;

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

    #[cfg(test)]
    pub fn from_digest(
        context: &mut crate::circuits::context::TraceContext,
        init_digest: [stwo::core::fields::qm31::QM31; 2],
    ) -> Self {
        Self {
            digest: HashValue(context.constant(init_digest[0]), context.constant(init_digest[1])),
            n_draws: 0,
        }
    }

    /// Mixes the given root into the channel's digest.
    pub fn mix_commitment(&mut self, context: &mut Context<impl IValue>, root: HashValue<Var>) {
        self.update_digest(blake(context, &[self.digest.0, self.digest.1, root.0, root.1], 16 * 4));
    }

    /// Draws one [QM31] random value from the channel.
    pub fn draw_qm31(&mut self, context: &mut Context<impl IValue>) -> Var {
        self.draw_two_qm31s(context)[0]
    }

    /// Draws two [QM31] random values from the channel.
    pub fn draw_two_qm31s(&mut self, context: &mut Context<impl IValue>) -> [Var; 2] {
        let n_draws_var =
            context.constant(qm31_from_u32s(self.n_draws.try_into().unwrap(), 0, 0, 0));
        // Note that we add a zero byte for domain separation between generating randomness and
        // mixing a single u32.
        let res = blake(context, &[self.digest.0, self.digest.1, n_draws_var], 16 + 16 + 4 + 1);
        self.n_draws += 1;
        [res.0, res.1]
    }

    /// Draws a random point on the ([QM31]) circle from the channel.
    pub fn draw_point(&mut self, context: &mut Context<impl IValue>) -> CirclePoint<Var> {
        let t = self.draw_qm31(context);
        let t2 = eval!(context, (t) * (t));

        let denom = eval!(context, (t2) + (1));
        let denom_inv = div(context, context.one(), denom);
        let x = eval!(context, ((1) - (t2)) * (denom_inv));
        let y = eval!(context, ((2) * (t)) * (denom_inv));
        CirclePoint { x, y }
    }
}
