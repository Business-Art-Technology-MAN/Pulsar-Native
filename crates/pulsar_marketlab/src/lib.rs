//! **Market Lab OTL** embedded in **Pulsar-Native** via path dependencies to the parent repo (`../../../../crates/...`).
//!
//! - Use [`OtlEvalDispatcher`] for **off–UI-thread** compile / [`run_eval`](marketlab_otl::run_eval) (GPUI must stay non-blocking).
//! - Map **Plugin_Blueprints** / DAG **output** and **AOV** sockets by **ordinal** with [`otl_param_index_for_blueprint_output_socket`] and
//!   [`otl_param_index_for_blueprint_aov_socket`] — those indices match [`marketlab_otl::vm::CompiledSignal::output_param_indices`] /
//!   [`marketlab_otl::vm::CompiledSignal::aov_param_indices`] (slots / [`RunArgs`] still key **inputs** by OTL **parameter name**).

pub use marketlab_pulsar_bridge::{OtlEvalDispatcher, OtlEvalJob};
pub use marketlab_otl::{
    compile_and_run_eval, compile_str, run_eval, CompiledSignal, OtlError, RunArgs, SignalEvalResult,
    Value,
};

/// Ordinal of an **`output`** port on a blueprint node, in **source order** (0 = primary when one output exists).
pub type BlueprintOutputSocketOrdinal = usize;

/// Ordinal of an **`aov`** port on a blueprint node, in **source order** (matches OTL `aov` parameter order).
pub type BlueprintAovSocketOrdinal = usize;

/// Resolved index into [`CompiledSignal::params`](marketlab_otl::vm::CompiledSignal::params).
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
    fn blueprint_ordinals_track_output_and_aov_indices() {
        let c = compile_str(MULTI_OUT).unwrap();
        assert_eq!(otl_param_index_for_blueprint_output_socket(&c, 0), c.output_param_indices.first().copied());
        assert_eq!(otl_param_index_for_blueprint_aov_socket(&c, 0), c.aov_param_indices.first().copied());
        assert_eq!(otl_param_index_for_blueprint_aov_socket(&c, 1), c.aov_param_indices.get(1).copied());
    }
}
