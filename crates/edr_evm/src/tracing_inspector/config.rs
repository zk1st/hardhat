// Based on Reth
// https://github.com/paradigmxyz/reth/blob/6ad221fd282f32d4bf699c92f22d4e2478372b5a/crates/revm/revm-inspectors/src/tracing/config.rs
// https://github.com/paradigmxyz/reth/blob/6ad221fd282f32d4bf699c92f22d4e2478372b5a/LICENSE-MIT

/// Gives guidance to the [TracingInspector](crate::tracing::TracingInspector).
///
/// Use [TracingInspectorConfig::default_parity] or
/// [TracingInspectorConfig::default_geth] to get the default configs for
/// specific styles of traces.
#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub struct TracingInspectorConfig {
    /// Whether to record every individual opcode level step.
    pub record_steps: bool,
    // /// Whether to record individual memory snapshots.
    // pub record_memory_snapshots: bool,
    /// Whether to record individual stack snapshots.
    pub record_stack_snapshots: StackSnapshotType,
    /// Whether to record state diffs.
    pub record_state_diff: bool,
    /// Whether to ignore precompile calls.
    pub exclude_precompile_calls: bool,
    // /// Whether to record logs
    // pub record_logs: bool,
}

impl TracingInspectorConfig {
    /// Returns a config with everything enabled.
    pub const fn all() -> Self {
        Self {
            record_steps: true,
            // record_memory_snapshots: true,
            record_stack_snapshots: StackSnapshotType::Full,
            record_state_diff: true,
            exclude_precompile_calls: false,
            // record_logs: true,
        }
    }

    /// Returns a config with nothing enabled.
    pub const fn none() -> Self {
        Self {
            record_steps: false,
            // record_memory_snapshots: true,
            record_stack_snapshots: StackSnapshotType::None,
            record_state_diff: false,
            exclude_precompile_calls: false,
            // record_logs: true,
        }
    }
}

/// How much of the stack to record. Nothing, just the items pushed, or the full
/// stack
#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub enum StackSnapshotType {
    /// Don't record stack snapshots
    None,
    /// Record only the items pushed to the stack
    Pushes,
    /// Record the full stack
    Full,
}

impl StackSnapshotType {
    /// Returns true if this is the [StackSnapshotType::Full] variant
    #[inline]
    pub fn is_full(self) -> bool {
        matches!(self, Self::Full)
    }

    /// Returns true if this is the [StackSnapshotType::Pushes] variant
    #[inline]
    pub fn is_pushes(self) -> bool {
        matches!(self, Self::Pushes)
    }
}
