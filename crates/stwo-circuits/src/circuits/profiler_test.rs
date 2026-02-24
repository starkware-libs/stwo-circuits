use expect_test::expect;

use crate::circuits::context::TraceContext;
use crate::circuits::ops::{add, cond_flip, conj, div, eq, guess, pointwise_mul, sub};
use crate::eval;
use crate::circuits::stats::ProfilerVerbosity;

fn build_profiler_report(verbosity: ProfilerVerbosity) -> String {
    let mut context = TraceContext::default();
    context.stats.profiler.stacks.clear();

    let a = context.new_var(3.into());
    let b = context.new_var(4.into());

    add(&mut context, a, b);
    add(&mut context, a, b);

    context.stats.profiler.format_with(verbosity)
}

fn build_elaborate_circuit() -> TraceContext {
    let mut context = TraceContext::default();

    let a = guess(&mut context, 3.into());
    let b = guess(&mut context, 5.into());
    let c = eval!(&mut context, ((a) + (b)) * ((a) - (b)));
    let d = div(&mut context, c, b);
    let e = pointwise_mul(&mut context, d, a);
    let one = context.one();
    let (f, g) = cond_flip(&mut context, one, e, c);

    // Call a helper from two different call sites.
    let h = helper_block(&mut context, f, g);
    let h2 = helper_block(&mut context, g, f);

    // Loop with a function call inside to exercise repeated call stacks.
    let mut acc = h;
    for _ in 0..3 {
        acc = loop_step(&mut context, acc, a);
    }

    eq(&mut context, f, c);
    eq(&mut context, g, e);
    eq(&mut context, h, h2);
    let acc2 = loop_step(&mut context, h, a);
    eq(&mut context, acc, acc2);

    context
}

fn helper_block(context: &mut TraceContext, x: crate::circuits::context::Var, y: crate::circuits::context::Var) -> crate::circuits::context::Var {
    let z = add(context, x, y);
    let w = sub(context, z, x);
    conj(context, w)
}

fn loop_step(context: &mut TraceContext, x: crate::circuits::context::Var, y: crate::circuits::context::Var) -> crate::circuits::context::Var {
    let t = pointwise_mul(context, x, y);
    add(context, t, y)
}

fn build_elaborate_report(verbosity: ProfilerVerbosity) -> String {
    let mut context = build_elaborate_circuit();
    context.stats.profiler.stacks.clear();
    context = build_elaborate_circuit();
    context.stats.profiler.format_with(verbosity)
}

#[test]
fn test_profiler_format_full() {
    let report = build_profiler_report(ProfilerVerbosity::Full);
    expect![[r#"
        stack=
          ./src/circuits/stats.rs:99 stwo_circuits::circuits::stats::Profiler::record
          ./src/circuits/stats.rs:257 stwo_circuits::circuits::stats::Stats::register
          ./src/circuits/ops.rs:69 stwo_circuits::circuits::ops::add
          ./src/circuits/profiler_test.rs:16 stwo_circuits::circuits::stats::profiler_test::build_profiler_report
          ./src/circuits/profiler_test.rs:71 stwo_circuits::circuits::stats::profiler_test::test_profiler_format_full
          ./src/circuits/profiler_test.rs:70 stwo_circuits::circuits::stats::profiler_test::test_profiler_format_full::{{closure}}
          /home/gil/.rustup/toolchains/nightly-2025-07-14-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/core/src/ops/function.rs:253 core::ops::function::FnOnce::call_once
          /rustc/e9182f195b8505c87c4bd055b9f6e114ccda0981/library/core/src/ops/function.rs:253 core::ops::function::FnOnce::call_once
          /rustc/e9182f195b8505c87c4bd055b9f6e114ccda0981/library/test/src/lib.rs:648 test::__rust_begin_short_backtrace
          /rustc/e9182f195b8505c87c4bd055b9f6e114ccda0981/library/test/src/lib.rs:671 test::run_test_in_process::{{closure}}
          /rustc/e9182f195b8505c87c4bd055b9f6e114ccda0981/library/core/src/panic/unwind_safe.rs:272 <core::panic::unwind_safe::AssertUnwindSafe<F> as core::ops::function::FnOnce<()>>::call_once
          /rustc/e9182f195b8505c87c4bd055b9f6e114ccda0981/library/std/src/panicking.rs:589 std::panicking::catch_unwind::do_call
          /rustc/e9182f195b8505c87c4bd055b9f6e114ccda0981/library/std/src/panicking.rs:552 std::panicking::catch_unwind
          /rustc/e9182f195b8505c87c4bd055b9f6e114ccda0981/library/std/src/panic.rs:359 std::panic::catch_unwind
          /rustc/e9182f195b8505c87c4bd055b9f6e114ccda0981/library/test/src/lib.rs:671 test::run_test_in_process
          /rustc/e9182f195b8505c87c4bd055b9f6e114ccda0981/library/test/src/lib.rs:592 test::run_test::{{closure}}
          /rustc/e9182f195b8505c87c4bd055b9f6e114ccda0981/library/test/src/lib.rs:622 test::run_test::{{closure}}
          /rustc/e9182f195b8505c87c4bd055b9f6e114ccda0981/library/std/src/sys/backtrace.rs:152 std::sys::backtrace::__rust_begin_short_backtrace
          /rustc/e9182f195b8505c87c4bd055b9f6e114ccda0981/library/std/src/thread/mod.rs:559 std::thread::Builder::spawn_unchecked_::{{closure}}::{{closure}}
          /rustc/e9182f195b8505c87c4bd055b9f6e114ccda0981/library/core/src/panic/unwind_safe.rs:272 <core::panic::unwind_safe::AssertUnwindSafe<F> as core::ops::function::FnOnce<()>>::call_once
          /rustc/e9182f195b8505c87c4bd055b9f6e114ccda0981/library/std/src/panicking.rs:589 std::panicking::catch_unwind::do_call
          /rustc/e9182f195b8505c87c4bd055b9f6e114ccda0981/library/std/src/panicking.rs:552 std::panicking::catch_unwind
          /rustc/e9182f195b8505c87c4bd055b9f6e114ccda0981/library/std/src/panic.rs:359 std::panic::catch_unwind
          /rustc/e9182f195b8505c87c4bd055b9f6e114ccda0981/library/std/src/thread/mod.rs:557 std::thread::Builder::spawn_unchecked_::{{closure}}
          /rustc/e9182f195b8505c87c4bd055b9f6e114ccda0981/library/core/src/ops/function.rs:253 core::ops::function::FnOnce::call_once{{vtable.shim}}
          /rustc/e9182f195b8505c87c4bd055b9f6e114ccda0981/library/alloc/src/boxed.rs:1971 <alloc::boxed::Box<F,A> as core::ops::function::FnOnce<Args>>::call_once
          /rustc/e9182f195b8505c87c4bd055b9f6e114ccda0981/library/alloc/src/boxed.rs:1971 <alloc::boxed::Box<F,A> as core::ops::function::FnOnce<Args>>::call_once
          /rustc/e9182f195b8505c87c4bd055b9f6e114ccda0981/library/std/src/sys/pal/unix/thread.rs:97 std::sys::pal::unix::thread::Thread::new::thread_start
          ./nptl/pthread_create.c:447 start_thread
          ./misc/../sysdeps/unix/sysv/linux/x86_64/clone3.S:78 clone3
        Add count=1 sum=0
        stack=
          ./src/circuits/stats.rs:99 stwo_circuits::circuits::stats::Profiler::record
          ./src/circuits/stats.rs:257 stwo_circuits::circuits::stats::Stats::register
          ./src/circuits/ops.rs:69 stwo_circuits::circuits::ops::add
          ./src/circuits/profiler_test.rs:15 stwo_circuits::circuits::stats::profiler_test::build_profiler_report
          ./src/circuits/profiler_test.rs:71 stwo_circuits::circuits::stats::profiler_test::test_profiler_format_full
          ./src/circuits/profiler_test.rs:70 stwo_circuits::circuits::stats::profiler_test::test_profiler_format_full::{{closure}}
          /home/gil/.rustup/toolchains/nightly-2025-07-14-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/core/src/ops/function.rs:253 core::ops::function::FnOnce::call_once
          /rustc/e9182f195b8505c87c4bd055b9f6e114ccda0981/library/core/src/ops/function.rs:253 core::ops::function::FnOnce::call_once
          /rustc/e9182f195b8505c87c4bd055b9f6e114ccda0981/library/test/src/lib.rs:648 test::__rust_begin_short_backtrace
          /rustc/e9182f195b8505c87c4bd055b9f6e114ccda0981/library/test/src/lib.rs:671 test::run_test_in_process::{{closure}}
          /rustc/e9182f195b8505c87c4bd055b9f6e114ccda0981/library/core/src/panic/unwind_safe.rs:272 <core::panic::unwind_safe::AssertUnwindSafe<F> as core::ops::function::FnOnce<()>>::call_once
          /rustc/e9182f195b8505c87c4bd055b9f6e114ccda0981/library/std/src/panicking.rs:589 std::panicking::catch_unwind::do_call
          /rustc/e9182f195b8505c87c4bd055b9f6e114ccda0981/library/std/src/panicking.rs:552 std::panicking::catch_unwind
          /rustc/e9182f195b8505c87c4bd055b9f6e114ccda0981/library/std/src/panic.rs:359 std::panic::catch_unwind
          /rustc/e9182f195b8505c87c4bd055b9f6e114ccda0981/library/test/src/lib.rs:671 test::run_test_in_process
          /rustc/e9182f195b8505c87c4bd055b9f6e114ccda0981/library/test/src/lib.rs:592 test::run_test::{{closure}}
          /rustc/e9182f195b8505c87c4bd055b9f6e114ccda0981/library/test/src/lib.rs:622 test::run_test::{{closure}}
          /rustc/e9182f195b8505c87c4bd055b9f6e114ccda0981/library/std/src/sys/backtrace.rs:152 std::sys::backtrace::__rust_begin_short_backtrace
          /rustc/e9182f195b8505c87c4bd055b9f6e114ccda0981/library/std/src/thread/mod.rs:559 std::thread::Builder::spawn_unchecked_::{{closure}}::{{closure}}
          /rustc/e9182f195b8505c87c4bd055b9f6e114ccda0981/library/core/src/panic/unwind_safe.rs:272 <core::panic::unwind_safe::AssertUnwindSafe<F> as core::ops::function::FnOnce<()>>::call_once
          /rustc/e9182f195b8505c87c4bd055b9f6e114ccda0981/library/std/src/panicking.rs:589 std::panicking::catch_unwind::do_call
          /rustc/e9182f195b8505c87c4bd055b9f6e114ccda0981/library/std/src/panicking.rs:552 std::panicking::catch_unwind
          /rustc/e9182f195b8505c87c4bd055b9f6e114ccda0981/library/std/src/panic.rs:359 std::panic::catch_unwind
          /rustc/e9182f195b8505c87c4bd055b9f6e114ccda0981/library/std/src/thread/mod.rs:557 std::thread::Builder::spawn_unchecked_::{{closure}}
          /rustc/e9182f195b8505c87c4bd055b9f6e114ccda0981/library/core/src/ops/function.rs:253 core::ops::function::FnOnce::call_once{{vtable.shim}}
          /rustc/e9182f195b8505c87c4bd055b9f6e114ccda0981/library/alloc/src/boxed.rs:1971 <alloc::boxed::Box<F,A> as core::ops::function::FnOnce<Args>>::call_once
          /rustc/e9182f195b8505c87c4bd055b9f6e114ccda0981/library/alloc/src/boxed.rs:1971 <alloc::boxed::Box<F,A> as core::ops::function::FnOnce<Args>>::call_once
          /rustc/e9182f195b8505c87c4bd055b9f6e114ccda0981/library/std/src/sys/pal/unix/thread.rs:97 std::sys::pal::unix::thread::Thread::new::thread_start
          ./nptl/pthread_create.c:447 start_thread
          ./misc/../sysdeps/unix/sysv/linux/x86_64/clone3.S:78 clone3
        Add count=1 sum=0
    "#]]
    .assert_eq(&report);
}

#[test]
fn test_profiler_format_filtered() {
    let report = build_profiler_report(ProfilerVerbosity::Filtered);
    expect![[r#"
        stack=
          ./src/circuits/profiler_test.rs:15 stwo_circuits::circuits::stats::profiler_test::build_profiler_report
          ./src/circuits/profiler_test.rs:143 stwo_circuits::circuits::stats::profiler_test::test_profiler_format_filtered
          ./src/circuits/profiler_test.rs:142 stwo_circuits::circuits::stats::profiler_test::test_profiler_format_filtered::{{closure}}
          ./nptl/pthread_create.c:447 start_thread
          ./misc/../sysdeps/unix/sysv/linux/x86_64/clone3.S:78 clone3
        Add count=1 sum=0
        stack=
          ./src/circuits/profiler_test.rs:16 stwo_circuits::circuits::stats::profiler_test::build_profiler_report
          ./src/circuits/profiler_test.rs:143 stwo_circuits::circuits::stats::profiler_test::test_profiler_format_filtered
          ./src/circuits/profiler_test.rs:142 stwo_circuits::circuits::stats::profiler_test::test_profiler_format_filtered::{{closure}}
          ./nptl/pthread_create.c:447 start_thread
          ./misc/../sysdeps/unix/sysv/linux/x86_64/clone3.S:78 clone3
        Add count=1 sum=0
    "#]]
    .assert_eq(&report);
}

#[test]
fn test_profiler_format_top() {
    let report = build_profiler_report(ProfilerVerbosity::Top);
    expect![[r#"
        stack=
          ./src/circuits/profiler_test.rs:15 stwo_circuits::circuits::stats::profiler_test::build_profiler_report
        Add count=1 sum=0
        stack=
          ./src/circuits/profiler_test.rs:16 stwo_circuits::circuits::stats::profiler_test::build_profiler_report
        Add count=1 sum=0
    "#]]
    .assert_eq(&report);
}

#[test]
fn test_profiler_elaborate_circuit_filtered() {
    let report = build_elaborate_report(ProfilerVerbosity::Filtered);
    expect![[r#"
        stack=
          ./src/circuits/ops.rs:28 stwo_circuits::circuits::ops::cond_flip
          ./src/circuits/profiler_test.rs:30 stwo_circuits::circuits::stats::profiler_test::build_elaborate_circuit
          ./src/circuits/profiler_test.rs:65 stwo_circuits::circuits::stats::profiler_test::build_elaborate_report
          ./src/circuits/profiler_test.rs:179 stwo_circuits::circuits::stats::profiler_test::test_profiler_elaborate_circuit_filtered
          ./src/circuits/profiler_test.rs:178 stwo_circuits::circuits::stats::profiler_test::test_profiler_elaborate_circuit_filtered::{{closure}}
          ./nptl/pthread_create.c:447 start_thread
          ./misc/../sysdeps/unix/sysv/linux/x86_64/clone3.S:78 clone3
        Add count=1 sum=0
        stack=
          ./src/circuits/ops.rs:28 stwo_circuits::circuits::stats::profiler_test::build_elaborate_circuit
          ./src/circuits/profiler_test.rs:65 stwo_circuits::circuits::stats::profiler_test::build_elaborate_report
          ./src/circuits/profiler_test.rs:179 stwo_circuits::circuits::stats::profiler_test::test_profiler_elaborate_circuit_filtered
          ./src/circuits/profiler_test.rs:178 stwo_circuits::circuits::stats::profiler_test::test_profiler_elaborate_circuit_filtered::{{closure}}
          ./nptl/pthread_create.c:447 start_thread
          ./misc/../sysdeps/unix/sysv/linux/x86_64/clone3.S:78 clone3
        Add count=1 sum=0
        stack=
          ./src/circuits/profiler_test.rs:52 stwo_circuits::circuits::stats::profiler_test::helper_block
          ./src/circuits/profiler_test.rs:33 stwo_circuits::circuits::stats::profiler_test::build_elaborate_circuit
          ./src/circuits/profiler_test.rs:65 stwo_circuits::circuits::stats::profiler_test::build_elaborate_report
          ./src/circuits/profiler_test.rs:179 stwo_circuits::circuits::stats::profiler_test::test_profiler_elaborate_circuit_filtered
          ./src/circuits/profiler_test.rs:178 stwo_circuits::circuits::stats::profiler_test::test_profiler_elaborate_circuit_filtered::{{closure}}
          ./nptl/pthread_create.c:447 start_thread
          ./misc/../sysdeps/unix/sysv/linux/x86_64/clone3.S:78 clone3
        Add count=1 sum=0
        stack=
          ./src/circuits/profiler_test.rs:52 stwo_circuits::circuits::stats::profiler_test::helper_block
          ./src/circuits/profiler_test.rs:34 stwo_circuits::circuits::stats::profiler_test::build_elaborate_circuit
          ./src/circuits/profiler_test.rs:65 stwo_circuits::circuits::stats::profiler_test::build_elaborate_report
          ./src/circuits/profiler_test.rs:179 stwo_circuits::circuits::stats::profiler_test::test_profiler_elaborate_circuit_filtered
          ./src/circuits/profiler_test.rs:178 stwo_circuits::circuits::stats::profiler_test::test_profiler_elaborate_circuit_filtered::{{closure}}
          ./nptl/pthread_create.c:447 start_thread
          ./misc/../sysdeps/unix/sysv/linux/x86_64/clone3.S:78 clone3
        Add count=1 sum=0
        stack=
          ./src/circuits/profiler_test.rs:59 stwo_circuits::circuits::stats::profiler_test::loop_step
          ./src/circuits/profiler_test.rs:39 stwo_circuits::circuits::stats::profiler_test::build_elaborate_circuit
          ./src/circuits/profiler_test.rs:65 stwo_circuits::circuits::stats::profiler_test::build_elaborate_report
          ./src/circuits/profiler_test.rs:179 stwo_circuits::circuits::stats::profiler_test::test_profiler_elaborate_circuit_filtered
          ./src/circuits/profiler_test.rs:178 stwo_circuits::circuits::stats::profiler_test::test_profiler_elaborate_circuit_filtered::{{closure}}
          ./nptl/pthread_create.c:447 start_thread
          ./misc/../sysdeps/unix/sysv/linux/x86_64/clone3.S:78 clone3
        Add count=3 sum=0
        stack=
          ./src/circuits/profiler_test.rs:59 stwo_circuits::circuits::stats::profiler_test::loop_step
          ./src/circuits/profiler_test.rs:45 stwo_circuits::circuits::stats::profiler_test::build_elaborate_circuit
          ./src/circuits/profiler_test.rs:65 stwo_circuits::circuits::stats::profiler_test::build_elaborate_report
          ./src/circuits/profiler_test.rs:179 stwo_circuits::circuits::stats::profiler_test::test_profiler_elaborate_circuit_filtered
          ./src/circuits/profiler_test.rs:178 stwo_circuits::circuits::stats::profiler_test::test_profiler_elaborate_circuit_filtered::{{closure}}
          ./nptl/pthread_create.c:447 start_thread
          ./misc/../sysdeps/unix/sysv/linux/x86_64/clone3.S:78 clone3
        Add count=1 sum=0
        stack=
          ./src/circuits/profiler_test.rs:27 stwo_circuits::circuits::stats::profiler_test::build_elaborate_circuit
          ./src/circuits/profiler_test.rs:65 stwo_circuits::circuits::stats::profiler_test::build_elaborate_report
          ./src/circuits/profiler_test.rs:179 stwo_circuits::circuits::stats::profiler_test::test_profiler_elaborate_circuit_filtered
          ./src/circuits/profiler_test.rs:178 stwo_circuits::circuits::stats::profiler_test::test_profiler_elaborate_circuit_filtered::{{closure}}
          ./nptl/pthread_create.c:447 start_thread
          ./misc/../sysdeps/unix/sysv/linux/x86_64/clone3.S:78 clone3
        Div count=1 sum=0
        stack=
          ./src/circuits/ops.rs:99 stwo_circuits::circuits::ops::div
          ./src/circuits/profiler_test.rs:27 stwo_circuits::circuits::stats::profiler_test::build_elaborate_circuit
          ./src/circuits/profiler_test.rs:65 stwo_circuits::circuits::stats::profiler_test::build_elaborate_report
          ./src/circuits/profiler_test.rs:179 stwo_circuits::circuits::stats::profiler_test::test_profiler_elaborate_circuit_filtered
          ./src/circuits/profiler_test.rs:178 stwo_circuits::circuits::stats::profiler_test::test_profiler_elaborate_circuit_filtered::{{closure}}
          ./nptl/pthread_create.c:447 start_thread
          ./misc/../sysdeps/unix/sysv/linux/x86_64/clone3.S:78 clone3
        Eq count=1 sum=0
        stack=
          ./src/circuits/profiler_test.rs:42 stwo_circuits::circuits::stats::profiler_test::build_elaborate_circuit
          ./src/circuits/profiler_test.rs:65 stwo_circuits::circuits::stats::profiler_test::build_elaborate_report
          ./src/circuits/profiler_test.rs:179 stwo_circuits::circuits::stats::profiler_test::test_profiler_elaborate_circuit_filtered
          ./src/circuits/profiler_test.rs:178 stwo_circuits::circuits::stats::profiler_test::test_profiler_elaborate_circuit_filtered::{{closure}}
          ./nptl/pthread_create.c:447 start_thread
          ./misc/../sysdeps/unix/sysv/linux/x86_64/clone3.S:78 clone3
        Eq count=1 sum=0
        stack=
          ./src/circuits/profiler_test.rs:43 stwo_circuits::circuits::stats::profiler_test::build_elaborate_circuit
          ./src/circuits/profiler_test.rs:65 stwo_circuits::circuits::stats::profiler_test::build_elaborate_report
          ./src/circuits/profiler_test.rs:179 stwo_circuits::circuits::stats::profiler_test::test_profiler_elaborate_circuit_filtered
          ./src/circuits/profiler_test.rs:178 stwo_circuits::circuits::stats::profiler_test::test_profiler_elaborate_circuit_filtered::{{closure}}
          ./nptl/pthread_create.c:447 start_thread
          ./misc/../sysdeps/unix/sysv/linux/x86_64/clone3.S:78 clone3
        Eq count=1 sum=0
        stack=
          ./src/circuits/profiler_test.rs:44 stwo_circuits::circuits::stats::profiler_test::build_elaborate_circuit
          ./src/circuits/profiler_test.rs:65 stwo_circuits::circuits::stats::profiler_test::build_elaborate_report
          ./src/circuits/profiler_test.rs:179 stwo_circuits::circuits::stats::profiler_test::test_profiler_elaborate_circuit_filtered
          ./src/circuits/profiler_test.rs:178 stwo_circuits::circuits::stats::profiler_test::test_profiler_elaborate_circuit_filtered::{{closure}}
          ./nptl/pthread_create.c:447 start_thread
          ./misc/../sysdeps/unix/sysv/linux/x86_64/clone3.S:78 clone3
        Eq count=1 sum=0
        stack=
          ./src/circuits/profiler_test.rs:46 stwo_circuits::circuits::stats::profiler_test::build_elaborate_circuit
          ./src/circuits/profiler_test.rs:65 stwo_circuits::circuits::stats::profiler_test::build_elaborate_report
          ./src/circuits/profiler_test.rs:179 stwo_circuits::circuits::stats::profiler_test::test_profiler_elaborate_circuit_filtered
          ./src/circuits/profiler_test.rs:178 stwo_circuits::circuits::stats::profiler_test::test_profiler_elaborate_circuit_filtered::{{closure}}
          ./nptl/pthread_create.c:447 start_thread
          ./misc/../sysdeps/unix/sysv/linux/x86_64/clone3.S:78 clone3
        Eq count=1 sum=0
        stack=
          ./src/circuits/context.rs:93 stwo_circuits::circuits::context::Context<Value>::constant
          ./src/circuits/context.rs:162 <stwo_circuits::circuits::context::Context<Value> as core::default::Default>::default
          ./src/circuits/profiler_test.rs:22 stwo_circuits::circuits::stats::profiler_test::build_elaborate_circuit
          ./src/circuits/profiler_test.rs:65 stwo_circuits::circuits::stats::profiler_test::build_elaborate_report
          ./src/circuits/profiler_test.rs:179 stwo_circuits::circuits::stats::profiler_test::test_profiler_elaborate_circuit_filtered
          ./src/circuits/profiler_test.rs:178 stwo_circuits::circuits::stats::profiler_test::test_profiler_elaborate_circuit_filtered::{{closure}}
          ./nptl/pthread_create.c:447 start_thread
          ./misc/../sysdeps/unix/sysv/linux/x86_64/clone3.S:78 clone3
        Guess count=1 sum=0
        stack=
          ./src/circuits/context.rs:93 stwo_circuits::circuits::context::Context<Value>::constant
          ./src/circuits/ops.rs:146 stwo_circuits::circuits::ops::conj
          ./src/circuits/profiler_test.rs:54 stwo_circuits::circuits::stats::profiler_test::helper_block
          ./src/circuits/profiler_test.rs:33 stwo_circuits::circuits::stats::profiler_test::build_elaborate_circuit
          ./src/circuits/profiler_test.rs:65 stwo_circuits::circuits::stats::profiler_test::build_elaborate_report
          ./src/circuits/profiler_test.rs:179 stwo_circuits::circuits::stats::profiler_test::test_profiler_elaborate_circuit_filtered
          ./src/circuits/profiler_test.rs:178 stwo_circuits::circuits::stats::profiler_test::test_profiler_elaborate_circuit_filtered::{{closure}}
          ./nptl/pthread_create.c:447 start_thread
          ./misc/../sysdeps/unix/sysv/linux/x86_64/clone3.S:78 clone3
        Guess count=1 sum=0
        stack=
          ./src/circuits/context.rs:93 stwo_circuits::circuits::context::Context<Value>::constant
          ./src/circuits/context.rs:161 <stwo_circuits::circuits::context::Context<Value> as core::default::Default>::default
          ./src/circuits/profiler_test.rs:22 stwo_circuits::circuits::stats::profiler_test::build_elaborate_circuit
          ./src/circuits/profiler_test.rs:65 stwo_circuits::circuits::stats::profiler_test::build_elaborate_report
          ./src/circuits/profiler_test.rs:179 stwo_circuits::circuits::stats::profiler_test::test_profiler_elaborate_circuit_filtered
          ./src/circuits/profiler_test.rs:178 stwo_circuits::circuits::stats::profiler_test::test_profiler_elaborate_circuit_filtered::{{closure}}
          ./nptl/pthread_create.c:447 start_thread
          ./misc/../sysdeps/unix/sysv/linux/x86_64/clone3.S:78 clone3
        Guess count=1 sum=0
        stack=
          ./src/circuits/ops.rs:97 stwo_circuits::circuits::ops::div
          ./src/circuits/profiler_test.rs:27 stwo_circuits::circuits::stats::profiler_test::build_elaborate_circuit
          ./src/circuits/profiler_test.rs:65 stwo_circuits::circuits::stats::profiler_test::build_elaborate_report
          ./src/circuits/profiler_test.rs:179 stwo_circuits::circuits::stats::profiler_test::test_profiler_elaborate_circuit_filtered
          ./src/circuits/profiler_test.rs:178 stwo_circuits::circuits::stats::profiler_test::test_profiler_elaborate_circuit_filtered::{{closure}}
          ./nptl/pthread_create.c:447 start_thread
          ./misc/../sysdeps/unix/sysv/linux/x86_64/clone3.S:78 clone3
        Guess count=1 sum=0
        stack=
          ./src/circuits/profiler_test.rs:24 stwo_circuits::circuits::stats::profiler_test::build_elaborate_circuit
          ./src/circuits/profiler_test.rs:65 stwo_circuits::circuits::stats::profiler_test::build_elaborate_report
          ./src/circuits/profiler_test.rs:179 stwo_circuits::circuits::stats::profiler_test::test_profiler_elaborate_circuit_filtered
          ./src/circuits/profiler_test.rs:178 stwo_circuits::circuits::stats::profiler_test::test_profiler_elaborate_circuit_filtered::{{closure}}
          ./nptl/pthread_create.c:447 start_thread
          ./misc/../sysdeps/unix/sysv/linux/x86_64/clone3.S:78 clone3
        Guess count=1 sum=0
        stack=
          ./src/circuits/profiler_test.rs:25 stwo_circuits::circuits::stats::profiler_test::build_elaborate_circuit
          ./src/circuits/profiler_test.rs:65 stwo_circuits::circuits::stats::profiler_test::build_elaborate_report
          ./src/circuits/profiler_test.rs:179 stwo_circuits::circuits::stats::profiler_test::test_profiler_elaborate_circuit_filtered
          ./src/circuits/profiler_test.rs:178 stwo_circuits::circuits::stats::profiler_test::test_profiler_elaborate_circuit_filtered::{{closure}}
          ./nptl/pthread_create.c:447 start_thread
          ./misc/../sysdeps/unix/sysv/linux/x86_64/clone3.S:78 clone3
        Guess count=1 sum=0
        stack=
          ./src/circuits/ops.rs:46 stwo_circuits::circuits::ops::cond_flip
          ./src/circuits/profiler_test.rs:30 stwo_circuits::circuits::stats::profiler_test::build_elaborate_circuit
          ./src/circuits/profiler_test.rs:65 stwo_circuits::circuits::stats::profiler_test::build_elaborate_report
          ./src/circuits/profiler_test.rs:179 stwo_circuits::circuits::stats::profiler_test::test_profiler_elaborate_circuit_filtered
          ./src/circuits/profiler_test.rs:178 stwo_circuits::circuits::stats::profiler_test::test_profiler_elaborate_circuit_filtered::{{closure}}
          ./nptl/pthread_create.c:447 start_thread
          ./misc/../sysdeps/unix/sysv/linux/x86_64/clone3.S:78 clone3
        Mul count=1 sum=0
        stack=
          ./src/circuits/ops.rs:46 stwo_circuits::circuits::stats::profiler_test::build_elaborate_circuit
          ./src/circuits/profiler_test.rs:65 stwo_circuits::circuits::stats::profiler_test::build_elaborate_report
          ./src/circuits/profiler_test.rs:179 stwo_circuits::circuits::stats::profiler_test::test_profiler_elaborate_circuit_filtered
          ./src/circuits/profiler_test.rs:178 stwo_circuits::circuits::stats::profiler_test::test_profiler_elaborate_circuit_filtered::{{closure}}
          ./nptl/pthread_create.c:447 start_thread
          ./misc/../sysdeps/unix/sysv/linux/x86_64/clone3.S:78 clone3
        Mul count=1 sum=0
        stack=
          ./src/circuits/ops.rs:98 stwo_circuits::circuits::ops::div
          ./src/circuits/profiler_test.rs:27 stwo_circuits::circuits::stats::profiler_test::build_elaborate_circuit
          ./src/circuits/profiler_test.rs:65 stwo_circuits::circuits::stats::profiler_test::build_elaborate_report
          ./src/circuits/profiler_test.rs:179 stwo_circuits::circuits::stats::profiler_test::test_profiler_elaborate_circuit_filtered
          ./src/circuits/profiler_test.rs:178 stwo_circuits::circuits::stats::profiler_test::test_profiler_elaborate_circuit_filtered::{{closure}}
          ./nptl/pthread_create.c:447 start_thread
          ./misc/../sysdeps/unix/sysv/linux/x86_64/clone3.S:78 clone3
        Mul count=1 sum=0
        stack=
          ./src/circuits/ops.rs:147 stwo_circuits::circuits::ops::conj
          ./src/circuits/profiler_test.rs:54 stwo_circuits::circuits::stats::profiler_test::helper_block
          ./src/circuits/profiler_test.rs:33 stwo_circuits::circuits::stats::profiler_test::build_elaborate_circuit
          ./src/circuits/profiler_test.rs:65 stwo_circuits::circuits::stats::profiler_test::build_elaborate_report
          ./src/circuits/profiler_test.rs:179 stwo_circuits::circuits::stats::profiler_test::test_profiler_elaborate_circuit_filtered
          ./src/circuits/profiler_test.rs:178 stwo_circuits::circuits::stats::profiler_test::test_profiler_elaborate_circuit_filtered::{{closure}}
          ./nptl/pthread_create.c:447 start_thread
          ./misc/../sysdeps/unix/sysv/linux/x86_64/clone3.S:78 clone3
        PointwiseMul count=1 sum=0
        stack=
          ./src/circuits/ops.rs:147 stwo_circuits::circuits::ops::conj
          ./src/circuits/profiler_test.rs:54 stwo_circuits::circuits::stats::profiler_test::helper_block
          ./src/circuits/profiler_test.rs:34 stwo_circuits::circuits::stats::profiler_test::build_elaborate_circuit
          ./src/circuits/profiler_test.rs:65 stwo_circuits::circuits::stats::profiler_test::build_elaborate_report
          ./src/circuits/profiler_test.rs:179 stwo_circuits::circuits::stats::profiler_test::test_profiler_elaborate_circuit_filtered
          ./src/circuits/profiler_test.rs:178 stwo_circuits::circuits::stats::profiler_test::test_profiler_elaborate_circuit_filtered::{{closure}}
          ./nptl/pthread_create.c:447 start_thread
          ./misc/../sysdeps/unix/sysv/linux/x86_64/clone3.S:78 clone3
        PointwiseMul count=1 sum=0
        stack=
          ./src/circuits/profiler_test.rs:28 stwo_circuits::circuits::stats::profiler_test::build_elaborate_circuit
          ./src/circuits/profiler_test.rs:65 stwo_circuits::circuits::stats::profiler_test::build_elaborate_report
          ./src/circuits/profiler_test.rs:179 stwo_circuits::circuits::stats::profiler_test::test_profiler_elaborate_circuit_filtered
          ./src/circuits/profiler_test.rs:178 stwo_circuits::circuits::stats::profiler_test::test_profiler_elaborate_circuit_filtered::{{closure}}
          ./nptl/pthread_create.c:447 start_thread
          ./misc/../sysdeps/unix/sysv/linux/x86_64/clone3.S:78 clone3
        PointwiseMul count=1 sum=0
        stack=
          ./src/circuits/profiler_test.rs:58 stwo_circuits::circuits::stats::profiler_test::loop_step
          ./src/circuits/profiler_test.rs:39 stwo_circuits::circuits::stats::profiler_test::build_elaborate_circuit
          ./src/circuits/profiler_test.rs:65 stwo_circuits::circuits::stats::profiler_test::build_elaborate_report
          ./src/circuits/profiler_test.rs:179 stwo_circuits::circuits::stats::profiler_test::test_profiler_elaborate_circuit_filtered
          ./src/circuits/profiler_test.rs:178 stwo_circuits::circuits::stats::profiler_test::test_profiler_elaborate_circuit_filtered::{{closure}}
          ./nptl/pthread_create.c:447 start_thread
          ./misc/../sysdeps/unix/sysv/linux/x86_64/clone3.S:78 clone3
        PointwiseMul count=3 sum=0
        stack=
          ./src/circuits/profiler_test.rs:58 stwo_circuits::circuits::stats::profiler_test::loop_step
          ./src/circuits/profiler_test.rs:45 stwo_circuits::circuits::stats::profiler_test::build_elaborate_circuit
          ./src/circuits/profiler_test.rs:65 stwo_circuits::circuits::stats::profiler_test::build_elaborate_report
          ./src/circuits/profiler_test.rs:179 stwo_circuits::circuits::stats::profiler_test::test_profiler_elaborate_circuit_filtered
          ./src/circuits/profiler_test.rs:178 stwo_circuits::circuits::stats::profiler_test::test_profiler_elaborate_circuit_filtered::{{closure}}
          ./nptl/pthread_create.c:447 start_thread
          ./misc/../sysdeps/unix/sysv/linux/x86_64/clone3.S:78 clone3
        PointwiseMul count=1 sum=0
        stack=
          ./src/circuits/ops.rs:34 stwo_circuits::circuits::ops::cond_flip
          ./src/circuits/profiler_test.rs:30 stwo_circuits::circuits::stats::profiler_test::build_elaborate_circuit
          ./src/circuits/profiler_test.rs:65 stwo_circuits::circuits::stats::profiler_test::build_elaborate_report
          ./src/circuits/profiler_test.rs:179 stwo_circuits::circuits::stats::profiler_test::test_profiler_elaborate_circuit_filtered
          ./src/circuits/profiler_test.rs:178 stwo_circuits::circuits::stats::profiler_test::test_profiler_elaborate_circuit_filtered::{{closure}}
          ./nptl/pthread_create.c:447 start_thread
          ./misc/../sysdeps/unix/sysv/linux/x86_64/clone3.S:78 clone3
        Sub count=2 sum=0
        stack=
          ./src/circuits/ops.rs:34 stwo_circuits::circuits::stats::profiler_test::build_elaborate_circuit
          ./src/circuits/profiler_test.rs:65 stwo_circuits::circuits::stats::profiler_test::build_elaborate_report
          ./src/circuits/profiler_test.rs:179 stwo_circuits::circuits::stats::profiler_test::test_profiler_elaborate_circuit_filtered
          ./src/circuits/profiler_test.rs:178 stwo_circuits::circuits::stats::profiler_test::test_profiler_elaborate_circuit_filtered::{{closure}}
          ./nptl/pthread_create.c:447 start_thread
          ./misc/../sysdeps/unix/sysv/linux/x86_64/clone3.S:78 clone3
        Sub count=1 sum=0
        stack=
          ./src/circuits/profiler_test.rs:53 stwo_circuits::circuits::stats::profiler_test::helper_block
          ./src/circuits/profiler_test.rs:34 stwo_circuits::circuits::stats::profiler_test::build_elaborate_circuit
          ./src/circuits/profiler_test.rs:65 stwo_circuits::circuits::stats::profiler_test::build_elaborate_report
          ./src/circuits/profiler_test.rs:179 stwo_circuits::circuits::stats::profiler_test::test_profiler_elaborate_circuit_filtered
          ./src/circuits/profiler_test.rs:178 stwo_circuits::circuits::stats::profiler_test::test_profiler_elaborate_circuit_filtered::{{closure}}
          ./nptl/pthread_create.c:447 start_thread
          ./misc/../sysdeps/unix/sysv/linux/x86_64/clone3.S:78 clone3
        Sub count=1 sum=0
        stack=
          ./src/circuits/profiler_test.rs:53 stwo_circuits::circuits::stats::profiler_test::helper_block
          ./src/circuits/profiler_test.rs:33 stwo_circuits::circuits::stats::profiler_test::build_elaborate_circuit
          ./src/circuits/profiler_test.rs:65 stwo_circuits::circuits::stats::profiler_test::build_elaborate_report
          ./src/circuits/profiler_test.rs:179 stwo_circuits::circuits::stats::profiler_test::test_profiler_elaborate_circuit_filtered
          ./src/circuits/profiler_test.rs:178 stwo_circuits::circuits::stats::profiler_test::test_profiler_elaborate_circuit_filtered::{{closure}}
          ./nptl/pthread_create.c:447 start_thread
          ./misc/../sysdeps/unix/sysv/linux/x86_64/clone3.S:78 clone3
        Sub count=1 sum=0
    "#]]
    .assert_eq(&report);
}

#[test]
fn test_profiler_elaborate_circuit_top() {
    let report = build_elaborate_report(ProfilerVerbosity::Top);
    expect![[r#"
        stack=
          ./src/circuits/ops.rs:28 stwo_circuits::circuits::ops::cond_flip
        Add count=1 sum=0
        stack=
          ./src/circuits/ops.rs:28 stwo_circuits::circuits::stats::profiler_test::build_elaborate_circuit
        Add count=1 sum=0
        stack=
          ./src/circuits/profiler_test.rs:52 stwo_circuits::circuits::stats::profiler_test::helper_block
        Add count=1 sum=0
        stack=
          ./src/circuits/profiler_test.rs:52 stwo_circuits::circuits::stats::profiler_test::helper_block
        Add count=1 sum=0
        stack=
          ./src/circuits/profiler_test.rs:59 stwo_circuits::circuits::stats::profiler_test::loop_step
        Add count=3 sum=0
        stack=
          ./src/circuits/profiler_test.rs:59 stwo_circuits::circuits::stats::profiler_test::loop_step
        Add count=1 sum=0
        stack=
          ./src/circuits/profiler_test.rs:27 stwo_circuits::circuits::stats::profiler_test::build_elaborate_circuit
        Div count=1 sum=0
        stack=
          ./src/circuits/ops.rs:99 stwo_circuits::circuits::ops::div
        Eq count=1 sum=0
        stack=
          ./src/circuits/profiler_test.rs:42 stwo_circuits::circuits::stats::profiler_test::build_elaborate_circuit
        Eq count=1 sum=0
        stack=
          ./src/circuits/profiler_test.rs:43 stwo_circuits::circuits::stats::profiler_test::build_elaborate_circuit
        Eq count=1 sum=0
        stack=
          ./src/circuits/profiler_test.rs:44 stwo_circuits::circuits::stats::profiler_test::build_elaborate_circuit
        Eq count=1 sum=0
        stack=
          ./src/circuits/profiler_test.rs:46 stwo_circuits::circuits::stats::profiler_test::build_elaborate_circuit
        Eq count=1 sum=0
        stack=
          ./src/circuits/context.rs:93 stwo_circuits::circuits::context::Context<Value>::constant
        Guess count=1 sum=0
        stack=
          ./src/circuits/context.rs:93 stwo_circuits::circuits::context::Context<Value>::constant
        Guess count=1 sum=0
        stack=
          ./src/circuits/context.rs:93 stwo_circuits::circuits::context::Context<Value>::constant
        Guess count=1 sum=0
        stack=
          ./src/circuits/ops.rs:97 stwo_circuits::circuits::ops::div
        Guess count=1 sum=0
        stack=
          ./src/circuits/profiler_test.rs:24 stwo_circuits::circuits::stats::profiler_test::build_elaborate_circuit
        Guess count=1 sum=0
        stack=
          ./src/circuits/profiler_test.rs:25 stwo_circuits::circuits::stats::profiler_test::build_elaborate_circuit
        Guess count=1 sum=0
        stack=
          ./src/circuits/ops.rs:46 stwo_circuits::circuits::ops::cond_flip
        Mul count=1 sum=0
        stack=
          ./src/circuits/ops.rs:46 stwo_circuits::circuits::stats::profiler_test::build_elaborate_circuit
        Mul count=1 sum=0
        stack=
          ./src/circuits/ops.rs:98 stwo_circuits::circuits::ops::div
        Mul count=1 sum=0
        stack=
          ./src/circuits/ops.rs:147 stwo_circuits::circuits::ops::conj
        PointwiseMul count=1 sum=0
        stack=
          ./src/circuits/ops.rs:147 stwo_circuits::circuits::ops::conj
        PointwiseMul count=1 sum=0
        stack=
          ./src/circuits/profiler_test.rs:28 stwo_circuits::circuits::stats::profiler_test::build_elaborate_circuit
        PointwiseMul count=1 sum=0
        stack=
          ./src/circuits/profiler_test.rs:58 stwo_circuits::circuits::stats::profiler_test::loop_step
        PointwiseMul count=3 sum=0
        stack=
          ./src/circuits/profiler_test.rs:58 stwo_circuits::circuits::stats::profiler_test::loop_step
        PointwiseMul count=1 sum=0
        stack=
          ./src/circuits/ops.rs:34 stwo_circuits::circuits::ops::cond_flip
        Sub count=2 sum=0
        stack=
          ./src/circuits/ops.rs:34 stwo_circuits::circuits::stats::profiler_test::build_elaborate_circuit
        Sub count=1 sum=0
        stack=
          ./src/circuits/profiler_test.rs:53 stwo_circuits::circuits::stats::profiler_test::helper_block
        Sub count=1 sum=0
        stack=
          ./src/circuits/profiler_test.rs:53 stwo_circuits::circuits::stats::profiler_test::helper_block
        Sub count=1 sum=0
    "#]]
    .assert_eq(&report);
}
