#[cfg(feature = "circuit-profiler")]
use std::collections::HashMap;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum StatEntryKind {
    Add,
    Sub,
    Mul,
    PointwiseMul,
    Eq,
    Blake { updates: usize },
    Permutation { len: usize },
    Div,
    Guess,
    Output,
}

#[cfg(feature = "circuit-profiler")]
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct StatCounts {
    pub kind: StatEntryKind,
    pub count: usize,
    pub sum: usize,
}

#[cfg(feature = "circuit-profiler")]
impl StatCounts {
    pub fn new(kind: StatEntryKind) -> Self {
        Self { kind, count: 0, sum: 0 }
    }

    pub fn record(&mut self, kind: StatEntryKind, sum_delta: usize) {
        fn same_variant(a: StatEntryKind, b: StatEntryKind) -> bool {
            match (a, b) {
                (StatEntryKind::Blake { .. }, StatEntryKind::Blake { .. }) => true,
                (StatEntryKind::Permutation { .. }, StatEntryKind::Permutation { .. }) => true,
                _ => a == b,
            }
        }
        debug_assert!(
            self.count == 0 || same_variant(self.kind, kind),
            "Profiler stack trace recorded with multiple kinds: {:?} vs {:?}",
            self.kind,
            kind
        );
        if self.count == 0 {
            self.kind = kind;
        }
        self.count += 1;
        self.sum += sum_delta;
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct Position {
    /// Source file path as recorded in the backtrace entry.
    pub file: String,
    /// 1-based line number in the source file.
    pub line: u32,
    /// Fully qualified function name when available.
    pub function: Option<String>,
}

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct StackKey {
    /// Structured frames extracted from a backtrace.
    pub frames: Vec<Position>,
}

#[cfg(feature = "circuit-profiler")]
#[derive(Default, Debug, PartialEq)]
pub struct Profiler {
    pub stacks: HashMap<StackKey, StatCounts>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ProfilerVerbosity {
    /// Keep all frames from the captured backtrace.
    Full,
    /// Keep only frames with paths starting with "./", then drop the top 3 entries.
    Filtered,
    /// Same as Filtered but keep only the first remaining entry.
    Top,
}

#[cfg(feature = "circuit-profiler")]
impl Profiler {
    /// Records a single stats entry with a structured backtrace key.
    pub fn record(&mut self, kind: StatEntryKind) {
        static BACKTRACE_CHECK: std::sync::OnceLock<()> = std::sync::OnceLock::new();
        BACKTRACE_CHECK.get_or_init(|| {
            let val = std::env::var("RUST_BACKTRACE").unwrap_or_default();
            assert!(
                val == "1" || val == "full",
                "circuit-profiler requires RUST_BACKTRACE=1 (or full)"
            );
        });

        let backtrace = std::backtrace::Backtrace::capture();
        let key = StackKey { frames: backtrace_to_positions(&backtrace) };
        let sum_delta = match kind {
            StatEntryKind::Blake { updates } => updates,
            StatEntryKind::Permutation { len } => len,
            _ => 0,
        };
        self.stacks.entry(key).or_insert_with(|| StatCounts::new(kind)).record(kind, sum_delta);
    }

    pub fn format(&self) -> String {
        self.format_with(ProfilerVerbosity::Full)
    }

    /// Formats profiler output according to the requested verbosity.
    pub fn format_with(&self, verbosity: ProfilerVerbosity) -> String {
        let mut entries: Vec<_> = self.stacks.iter().collect();
        entries.sort_by_key(|(key, counts)| {
            let mut sort_key = format!("{:?}", counts.kind);
            if let Some(first) = stack_positions_for_verbosity(key, verbosity).first() {
                sort_key.push('|');
                sort_key.push_str(&first.file);
                sort_key.push(':');
                sort_key.push_str(&first.line.to_string());
                if let Some(function) = &first.function {
                    sort_key.push(':');
                    sort_key.push_str(function);
                }
            }
            sort_key
        });
        let mut out = String::new();
        for (key, counts) in entries {
            out.push_str("stack=\n");
            for position in stack_positions_for_verbosity(key, verbosity) {
                match &position.function {
                    Some(function) => {
                        out.push_str(&format!(
                            "  {}:{} {}\n",
                            position.file, position.line, function
                        ));
                    }
                    None => {
                        out.push_str(&format!("  {}:{}\n", position.file, position.line));
                    }
                }
            }
            out.push_str(&format!(
                "{:?} count={} sum={}\n",
                counts.kind, counts.count, counts.sum
            ));
        }
        out
    }
}

#[cfg(feature = "circuit-profiler")]
/// Converts a `Backtrace` into structured positions by parsing its debug output.
fn backtrace_to_positions(backtrace: &std::backtrace::Backtrace) -> Vec<Position> {
    let text = format!("{backtrace:?}");
    parse_backtrace_text(&text)
}

#[cfg(feature = "circuit-profiler")]
/// Returns the list of positions to display for a given verbosity level.
fn stack_positions_for_verbosity<'a>(
    key: &'a StackKey,
    verbosity: ProfilerVerbosity,
) -> Vec<&'a Position> {
    match verbosity {
        ProfilerVerbosity::Full => key.frames.iter().collect(),
        ProfilerVerbosity::Filtered => key
            .frames
            .iter()
            .filter(|pos| pos.file.starts_with("./"))
            .skip(3)
            .collect(),
        ProfilerVerbosity::Top => key
            .frames
            .iter()
            .filter(|pos| pos.file.starts_with("./"))
            .skip(3)
            .take(1)
            .collect(),
    }
}

#[cfg(feature = "circuit-profiler")]
/// Parses the `Debug` representation of `Backtrace` into structured positions.
///
/// We use debug text parsing because `std::backtrace::Backtrace` does not expose
/// a stable structured frame API; `Backtrace::frames()` is still unstable on
/// the current toolchain.
fn parse_backtrace_text(text: &str) -> Vec<Position> {
    let mut positions = Vec::new();
    let mut rest = text;

    while let Some(start) = rest.find("{ fn: \"") {
        rest = &rest[start + "{ fn: \"".len()..];
        let Some(fn_end) = rest.find("\", file: \"") else { break };
        let function = rest[..fn_end].to_string();
        rest = &rest[fn_end + "\", file: \"".len()..];

        let Some(file_end) = rest.find("\", line: ") else { break };
        let file = rest[..file_end].to_string();
        rest = &rest[file_end + "\", line: ".len()..];

        let mut line_end = rest.find(" }").or_else(|| rest.find(" },"));
        if line_end.is_none() {
            line_end = rest.find(" }]").or_else(|| rest.find(" }]"));
        }
        let Some(line_end) = line_end else { break };
        let line_str = &rest[..line_end];
        let line = line_str.parse::<u32>().unwrap_or(0);
        rest = &rest[line_end..];

        if line > 0 {
            positions.push(Position { file, line, function: Some(function) });
        }
    }

    positions
}

#[derive(Debug, Default, PartialEq)]
pub struct Stats {
    pub equals: usize,
    pub add: usize,
    pub sub: usize,
    pub mul: usize,
    pub pointwise_mul: usize,
    pub blake_updates: usize,
    pub permutation_len: usize,
    /// The number of divisions. Note that each division also increments [Self::mul],
    /// [Self::guess] and [Self::equals] by 1.
    pub div: usize,
    pub guess: usize,
    pub outputs: usize,
    #[cfg(feature = "circuit-profiler")]
    pub profiler: Profiler,
}

impl Stats {
    pub fn register(&mut self, kind: StatEntryKind) {
        match kind {
            StatEntryKind::Add => self.add += 1,
            StatEntryKind::Sub => self.sub += 1,
            StatEntryKind::Mul => self.mul += 1,
            StatEntryKind::PointwiseMul => self.pointwise_mul += 1,
            StatEntryKind::Eq => self.equals += 1,
            StatEntryKind::Blake { updates } => self.blake_updates += updates,
            StatEntryKind::Permutation { len } => self.permutation_len += len,
            StatEntryKind::Div => self.div += 1,
            StatEntryKind::Guess => self.guess += 1,
            StatEntryKind::Output => self.outputs += 1,
        }
        #[cfg(feature = "circuit-profiler")]
        {
            self.profiler.record(kind);
        }
    }
}

/// Asserts Stats equality while explicitly ignoring profiler data.
pub fn assert_stats_eq(actual: &Stats, expected: &Stats) {
    let Stats {
        equals: a_equals,
        add: a_add,
        sub: a_sub,
        mul: a_mul,
        pointwise_mul: a_pointwise_mul,
        blake_updates: a_blake_updates,
        permutation_len: a_permutation_len,
        div: a_div,
        guess: a_guess,
        outputs: a_outputs,
        #[cfg(feature = "circuit-profiler")]
        profiler: _,
    } = actual;

    let Stats {
        equals: e_equals,
        add: e_add,
        sub: e_sub,
        mul: e_mul,
        pointwise_mul: e_pointwise_mul,
        blake_updates: e_blake_updates,
        permutation_len: e_permutation_len,
        div: e_div,
        guess: e_guess,
        outputs: e_outputs,
        #[cfg(feature = "circuit-profiler")]
        profiler: _,
    } = expected;

    assert_eq!(a_equals, e_equals);
    assert_eq!(a_add, e_add);
    assert_eq!(a_sub, e_sub);
    assert_eq!(a_mul, e_mul);
    assert_eq!(a_pointwise_mul, e_pointwise_mul);
    assert_eq!(a_blake_updates, e_blake_updates);
    assert_eq!(a_permutation_len, e_permutation_len);
    assert_eq!(a_div, e_div);
    assert_eq!(a_guess, e_guess);
    assert_eq!(a_outputs, e_outputs);
}

// #[cfg(test)]
// #[cfg(feature = "circuit-profiler")]
// #[path = "profiler_test.rs"]
// pub mod profiler_test;
