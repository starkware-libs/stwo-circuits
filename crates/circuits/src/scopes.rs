//! Scope recording for the external `circuit-gui` crate (the default-off
//! `gui-scopes` feature).
//!
//! [`Context::push_scope`] / [`Context::pop_scope`] are the *only* additions the
//! GUI makes to existing builder code; everything that records and stores the
//! resulting spans lives here, in this dedicated module. The recording state is
//! thread-local (circuit construction is sequential), so the `Context` struct is
//! not modified at all.
//!
//! The two methods are defined unconditionally so the no-op call sites in the
//! builders compile in normal (feature-off) builds; their bodies — and all the
//! storage below — exist only when `gui-scopes` is enabled.

use crate::context::Context;
use crate::ivalue::IValue;

impl<Value: IValue> Context<Value> {
    /// Opens a named (optionally nested) scope. Gates produced until the
    /// matching [`Self::pop_scope`] are attributed to this scope by the
    /// `circuit-gui` crate. No-op unless the `gui-scopes` feature is enabled.
    pub fn push_scope(&mut self, _name: &str) {
        #[cfg(feature = "gui-scopes")]
        imp::push(_name, self.gate_counts());
    }

    /// Closes the innermost scope opened by [`Self::push_scope`]. No-op unless
    /// the `gui-scopes` feature is enabled.
    pub fn pop_scope(&mut self) {
        #[cfg(feature = "gui-scopes")]
        imp::pop(self.gate_counts());
    }

    #[cfg(feature = "gui-scopes")]
    fn gate_counts(&self) -> imp::Counts {
        let c = &self.circuit;
        [
            c.add.len(),
            c.sub.len(),
            c.mul.len(),
            c.pointwise_mul.len(),
            c.eq.len(),
            c.blake_g_gate.len(),
            c.triple_xor.len(),
            c.m31_to_u32.len(),
            c.permutation.len(),
            c.output.len(),
        ]
    }
}

#[cfg(feature = "gui-scopes")]
pub use imp::{ScopeSpan, reset, take_spans};

#[cfg(feature = "gui-scopes")]
mod imp {
    use std::cell::RefCell;

    /// Per-kind gate counts, in the order the kinds appear in `Circuit`:
    /// add, sub, mul, pointwise_mul, eq, blake_g_gate, triple_xor, m31_to_u32,
    /// permutation, output. Must match the `circuit-gui` exporter's kind order.
    pub type Counts = [usize; 10];

    /// A named, possibly nested scope, capturing the gate-count range produced
    /// between its `push_scope` and `pop_scope`.
    #[derive(Clone, Debug)]
    pub struct ScopeSpan {
        /// Path from outermost to innermost scope (e.g. `["hash", "blake"]`).
        pub path: Vec<String>,
        pub before: Counts,
        pub after: Counts,
    }

    thread_local! {
        static STACK: RefCell<Vec<String>> = const { RefCell::new(Vec::new()) };
        static OPEN: RefCell<Vec<(Vec<String>, Counts)>> = const { RefCell::new(Vec::new()) };
        static SPANS: RefCell<Vec<ScopeSpan>> = const { RefCell::new(Vec::new()) };
    }

    pub(super) fn push(name: &str, counts: Counts) {
        STACK.with(|s| s.borrow_mut().push(name.to_string()));
        let path = STACK.with(|s| s.borrow().clone());
        OPEN.with(|o| o.borrow_mut().push((path, counts)));
    }

    pub(super) fn pop(counts: Counts) {
        let (path, before) =
            OPEN.with(|o| o.borrow_mut().pop().expect("pop_scope without matching push_scope"));
        SPANS.with(|sp| sp.borrow_mut().push(ScopeSpan { path, before, after: counts }));
        STACK.with(|s| {
            s.borrow_mut().pop();
        });
    }

    /// Clears all recorded scope state. Call before building a circuit.
    pub fn reset() {
        STACK.with(|s| s.borrow_mut().clear());
        OPEN.with(|o| o.borrow_mut().clear());
        SPANS.with(|sp| sp.borrow_mut().clear());
    }

    /// Drains and returns the spans recorded since the last [`reset`].
    pub fn take_spans() -> Vec<ScopeSpan> {
        SPANS.with(|sp| std::mem::take(&mut *sp.borrow_mut()))
    }
}
