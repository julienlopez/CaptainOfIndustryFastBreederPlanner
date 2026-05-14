use dioxus::prelude::*;
use std::sync::Arc;
use crate::data::GameData;
use crate::state::{Facility, BalanceMode};
use crate::balance::{compute_balance, fmt_qty};
use super::icons::ResIcon;

#[component]
pub fn BalancePanel(
    facilities: Signal<Vec<Facility>>,
    balance_mode: Signal<BalanceMode>,
) -> Element {
    let game_data = use_context::<Arc<GameData>>();
    let facs = facilities.read().clone();
    let mode = balance_mode.read().clone();

    // Compute balance
    let balance = compute_balance(&facs, &game_data.recipes_by_id);

    // Collect active resources (any non-zero flow)
    let mut active: Vec<(String, f64, f64)> = balance
        .iter()
        .filter(|(_, b)| b.in_per_min > 0.0 || b.out_per_min > 0.0)
        .map(|(id, b)| (id.clone(), b.in_per_min, b.out_per_min))
        .collect();

    // Sort by |net| descending
    active.sort_by(|(_, in_a, out_a), (_, in_b, out_b)| {
        let net_a = (out_a - in_a).abs();
        let net_b = (out_b - in_b).abs();
        net_b.partial_cmp(&net_a).unwrap_or(std::cmp::Ordering::Equal)
    });

    // Filter for external mode
    let shown: Vec<(String, f64, f64)> = match mode {
        BalanceMode::All => active.clone(),
        BalanceMode::External => active
            .iter()
            .filter(|(_, in_v, out_v)| (out_v - in_v).abs() > 0.001)
            .cloned()
            .collect(),
    };

    let cur_mode = balance_mode.read().clone();

    rsx! {
        div { class: "panel",
            div { class: "bal-header",
                div { class: "panel-title", "Net Balance — per minute" }
                div { class: "bal-switch",
                    button {
                        class: if cur_mode == BalanceMode::All { "active" } else { "" },
                        title: "Show every resource in play",
                        onclick: move |_| balance_mode.set(BalanceMode::All),
                        "All"
                    }
                    button {
                        class: if cur_mode == BalanceMode::External { "active" } else { "" },
                        title: "Show only unbalanced resources",
                        onclick: move |_| balance_mode.set(BalanceMode::External),
                        "External I/O"
                    }
                }
            }

            if shown.is_empty() {
                div { class: "empty-balance",
                    if active.is_empty() {
                        "No active resources yet."
                        br {}
                        "Add some facilities."
                    } else {
                        "No external inputs or outputs —"
                        br {}
                        "everything is internally balanced."
                    }
                }
            } else {
                div { class: "balance-list",
                    for (resource_id, in_amt, out_amt) in shown {
                        BalanceRow {
                            resource_id: resource_id,
                            in_amt: in_amt,
                            out_amt: out_amt,
                        }
                    }
                }
            }

            div { class: "note-bar",
                span { class: "warn", "~APX" }
                " = approximate; verify in-game. "
                "Reactor recipes (orange) are exact from the wiki. "
                "All rates shown "
                strong { "per minute" }
                "."
            }
        }
    }
}

#[component]
fn BalanceRow(resource_id: String, in_amt: f64, out_amt: f64) -> Element {
    let game_data = use_context::<Arc<GameData>>();
    let Some(res) = game_data.resources_by_id.get(&resource_id).cloned() else {
        return rsx! {};
    };

    let net = out_amt - in_amt;
    let net_class = if net.abs() < 0.001 {
        "bal-net zero"
    } else if net > 0.0 {
        "bal-net pos"
    } else {
        "bal-net neg"
    };
    let sign = if net > 0.001 { "+" } else { "" };
    let net_str = fmt_qty(net);

    rsx! {
        div { class: "bal-row",
            ResIcon { resource_id: resource_id.clone(), size: 18 }
            span { class: "bal-name", "{res.name}" }
            span { class: "{net_class}", "{sign}{net_str}/m" }
        }
    }
}
