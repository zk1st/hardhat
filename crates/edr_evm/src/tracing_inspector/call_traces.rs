// Based on Reth
// https://github.com/paradigmxyz/reth/blob/6ad221fd282f32d4bf699c92f22d4e2478372b5a/crates/revm/revm-inspectors/src/tracing/arena.rs
// https://github.com/paradigmxyz/reth/blob/6ad221fd282f32d4bf699c92f22d4e2478372b5a/LICENSE-MIT

use crate::tracing_inspector::types::{CallTrace, CallTraceNode, LogCallOrder};

/// A collection of recorded traces.
///
/// This type will be populated via the
/// [TracingInspector](crate::tracing_inspector::TracingInspector).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CallTraces {
    /// The recorded trace nodes
    pub(crate) traces: Vec<CallTraceNode>,
}

impl CallTraces {
    /// Pushes a new trace into the collection, returning the trace ID
    ///
    /// This appends a new trace to the collection, and also inserts a new entry
    /// in the node's parent node children set if `attach_to_parent` is
    /// `true`. E.g. if calls to precompiles should not be included in the
    /// call graph this should be called with [PushTraceKind::PushOnly].
    pub(crate) fn push_trace(
        &mut self,
        mut entry: usize,
        kind: PushTraceKind,
        new_trace: CallTrace,
    ) -> usize {
        loop {
            match new_trace.depth {
                // The entry node, just update it
                0 => {
                    self.traces[0].trace = new_trace;
                    return 0;
                }
                // We found the parent node, add the new trace as a child
                _ if self.traces[entry].trace.depth == new_trace.depth - 1 => {
                    let id = self.traces.len();
                    let node = CallTraceNode {
                        parent: Some(entry),
                        trace: new_trace,
                        idx: id,
                        ..Default::default()
                    };
                    self.traces.push(node);

                    // also track the child in the parent node
                    if kind.is_attach_to_parent() {
                        let parent = &mut self.traces[entry];
                        let trace_location = parent.children.len();
                        parent.ordering.push(LogCallOrder::Call(trace_location));
                        parent.children.push(id);
                    }

                    return id;
                }
                _ => {
                    // We haven't found the parent node, go deeper
                    entry = *self.traces[entry]
                        .children
                        .last()
                        .expect("Disconnected trace");
                }
            }
        }
    }

    /// Returns the nodes in the collection
    pub fn nodes(&self) -> &[CallTraceNode] {
        &self.traces
    }

    /// Consumes the collection and returns the nodes
    pub fn into_nodes(self) -> Vec<CallTraceNode> {
        self.traces
    }
}

impl Default for CallTraces {
    fn default() -> Self {
        // The first node is the root node
        CallTraces {
            traces: vec![Default::default()],
        }
    }
}

/// How to push a trace into the collection
pub(crate) enum PushTraceKind {
    /// This will _only_ push the trace into the collection.
    PushOnly,
    /// This will push the trace into the collection, and also insert a new
    /// entry in the node's parent node children set.
    PushAndAttachToParent,
}

impl PushTraceKind {
    #[inline]
    fn is_attach_to_parent(&self) -> bool {
        matches!(self, PushTraceKind::PushAndAttachToParent)
    }
}
