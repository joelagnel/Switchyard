// SPDX-FileCopyrightText: Copyright (c) 2026 NVIDIA CORPORATION & AFFILIATES. All rights reserved.
// SPDX-License-Identifier: Apache-2.0

//! Model capabilities advertised on `GET /v1/models`. A route id is an operator
//! alias (e.g. `random`) and the model behind it can change, so Switchyard never
//! guesses a capability from the id or the target model name. The operator
//! declares each target's context window and tool-calling support in the server
//! config, and a route advertises those declared values aggregated across the
//! targets it can answer from. A capability no target declares is reported as
//! `null` rather than an invented number.

/// Capabilities advertised on `GET /v1/models`. `None` means the value is
/// unknown (the operator declared nothing) and the endpoint reports `null`. A
/// route's capabilities are combined from its serving targets': a field is known
/// only when every serving tier declares it.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct ModelCapabilities {
    /// Context window in tokens, or `None` when undeclared.
    pub context_window: Option<u32>,
    /// Tool-calling support, or `None` when undeclared.
    pub tool_calling: Option<bool>,
}

impl ModelCapabilities {
    /// Combine the declared capabilities across the targets a route can answer
    /// from. A field is known only when every serving tier declares it: the context
    /// window is the smallest declared value (a route is only as capable as its
    /// most limited tier) and tool calling holds only when every tier supports
    /// it. A single undeclared tier — or a route with no serving targets —
    /// reports `None` for that field rather than guessing.
    pub(crate) fn for_targets(per_target: impl IntoIterator<Item = ModelCapabilities>) -> Self {
        per_target
            .into_iter()
            .reduce(|left, right| ModelCapabilities {
                context_window: merge(left.context_window, right.context_window, u32::min),
                tool_calling: merge(left.tool_calling, right.tool_calling, |a, b| a && b),
            })
            .unwrap_or_default()
    }
}

/// Combine two optional hints, staying known only when both sides are known.
fn merge<T>(left: Option<T>, right: Option<T>, combine: impl FnOnce(T, T) -> T) -> Option<T> {
    match (left, right) {
        (Some(left), Some(right)) => Some(combine(left, right)),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// One target's declared hints.
    fn target(context_window: Option<u32>, tool_calling: Option<bool>) -> ModelCapabilities {
        ModelCapabilities {
            context_window,
            tool_calling,
        }
    }

    #[test]
    fn aggregates_the_smallest_window_and_tool_calling_across_targets() {
        // Every tier declares both, so the route reports the smallest window and
        // tool calling (true only because every tier supports it).
        let caps = ModelCapabilities::for_targets([
            target(Some(1_000_000), Some(true)),
            target(Some(200_000), Some(true)),
        ]);
        assert_eq!(caps.context_window, Some(200_000));
        assert_eq!(caps.tool_calling, Some(true));
    }

    #[test]
    fn one_tier_without_tool_calling_makes_the_route_report_false() {
        // A route can land on the tier that cannot call tools, so the route as a
        // whole cannot claim tool calling.
        let caps = ModelCapabilities::for_targets([
            target(Some(200_000), Some(true)),
            target(Some(200_000), Some(false)),
        ]);
        assert_eq!(caps.tool_calling, Some(false));
    }

    #[test]
    fn a_missing_field_reports_null_while_the_other_field_still_aggregates() {
        // One tier declares a window, the other does not: the window is unknown
        // for the route (null), but tool calling is known for every tier.
        let caps = ModelCapabilities::for_targets([
            target(Some(1_000_000), Some(true)),
            target(None, Some(true)),
        ]);
        assert_eq!(caps.context_window, None);
        assert_eq!(caps.tool_calling, Some(true));
    }

    #[test]
    fn a_route_with_no_serving_targets_reports_null() {
        let caps = ModelCapabilities::for_targets(std::iter::empty());
        assert_eq!(caps.context_window, None);
        assert_eq!(caps.tool_calling, None);
    }
}
