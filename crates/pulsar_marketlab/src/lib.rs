//! **Market Lab OTL** embedded in **Pulsar-Native** via path dependencies to the parent repo (`../../../../crates/...`).
//!
//! - Call [`otl_eval_dispatcher`] once the **`marketlab`** feature is enabled (see `pulsar_engine --features marketlab`) so GPUI / blueprint code shares **one** FIFO worker thread.
//! - Map **Plugin_Blueprints** / DAG **output** and **AOV** sockets by **ordinal** with [`otl_param_index_for_blueprint_output_socket`] and
//!   [`otl_param_index_for_blueprint_aov_socket`] — those indices match [`CompiledSignal::output_param_indices`] /
//!   [`CompiledSignal::aov_param_indices`] (slots / [`RunArgs`] still key **inputs** by OTL **parameter name**).

pub use marketlab_pulsar_bridge::{OtlEvalDispatcher, OtlEvalJob};
pub use marketlab_otl::{
    compile_and_run_eval, compile_str, run_eval, CompiledSignal, OtlError, RunArgs, SignalEvalResult,
    Value,
};

use std::sync::OnceLock;

static OTL_EVAL_DISPATCHER: OnceLock<OtlEvalDispatcher> = OnceLock::new();

/// Process-wide OTL compile / eval worker ([`OtlEvalDispatcher`]): **one** background thread, **FIFO** jobs.
///
/// Lazily initialized on first use. **Safe from the GPUI thread** — only spawns the worker; enqueue with
/// [`OtlEvalDispatcher::submit_compile_and_run`] / [`OtlEvalDispatcher::submit_run_eval`] and read results on the
/// `mpsc::Receiver` you pass in (poll from a background task, or `recv` on a dedicated thread — never block the UI
/// waiting on eval).
pub fn otl_eval_dispatcher() -> &'static OtlEvalDispatcher {
    OTL_EVAL_DISPATCHER.get_or_init(OtlEvalDispatcher::spawn)
}

/// Ordinal of an **`output`** port on a blueprint node, in **source order** (0 = primary when one output exists).
pub type BlueprintOutputSocketOrdinal = usize;

/// Ordinal of an **`aov`** port on a blueprint node, in **source order** (matches OTL `aov` parameter order).
pub type BlueprintAovSocketOrdinal = usize;

/// Resolved index into [`CompiledSignal::params`].
pub type OtlParamIndex = usize;

/// Maps a blueprint **output** socket ordinal to an OTL **parameter index** (`params` row).
///
/// Contract: blueprint tools should list `output` sockets in the same order as OTL `output` parameters in the compiled signal.
#[inline]
pub fn otl_param_index_for_blueprint_output_socket(
    compiled: &CompiledSignal,
    socket: BlueprintOutputSocketOrdinal,
) -> Option<OtlParamIndex> {
    compiled.output_param_indices.get(socket).copied()
}

/// Maps a blueprint **AOV** socket ordinal to an OTL **parameter index**.
#[inline]
pub fn otl_param_index_for_blueprint_aov_socket(
    compiled: &CompiledSignal,
    socket: BlueprintAovSocketOrdinal,
) -> Option<OtlParamIndex> {
    compiled.aov_param_indices.get(socket).copied()
}

/// Primary (first `output`) OTL parameter index, if any.
#[inline]
pub fn otl_param_index_for_primary_output(compiled: &CompiledSignal) -> Option<OtlParamIndex> {
    compiled.output_param_indices.first().copied()
}

#[cfg(test)]
mod tests {
    use super::*;

    const MULTI_OUT: &str = r#"
signal demo(
    input float x = 0.0,
    output float y = 0.0,
    aov float a = 0.0,
    aov float b = 0.0
) {
    y = x;
    a = x + 1.0;
    b = x + 2.0;
}
"#;

    #[test]
    fn global_dispatcher_is_singleton() {
        let a = std::ptr::from_ref(otl_eval_dispatcher());
        let b = std::ptr::from_ref(otl_eval_dispatcher());
        assert_eq!(a, b);
    }

    #[test]
    fn blueprint_ordinals_track_output_and_aov_indices() {
        let c = compile_str(MULTI_OUT).unwrap();
        assert_eq!(otl_param_index_for_blueprint_output_socket(&c, 0), c.output_param_indices.first().copied());
        assert_eq!(otl_param_index_for_blueprint_aov_socket(&c, 0), c.aov_param_indices.first().copied());
        assert_eq!(otl_param_index_for_blueprint_aov_socket(&c, 1), c.aov_param_indices.get(1).copied());
    }
}
