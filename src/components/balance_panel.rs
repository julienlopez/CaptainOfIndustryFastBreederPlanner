use super::icons::ResIcon;
use crate::balance::{compute_balance, fmt_qty};
use crate::data::GameData;
use crate::state::Facility;
use dioxus::prelude::*;
use std::sync::Arc;

#[component]
pub fn BalancePanel(facilities: Signal<Vec<Facility>>) -> Element {
    let game_data = use_context::<Arc<GameData>>();
    let facs = facilities.read().clone();

    let balance = compute_balance(&facs, &game_data.recipes_by_id);

    // External I/O: resources with a non-zero net flow (unbalanced).
    // Internally recycled resources (in == out) are hidden.
    let mut shown: Vec<(String, f64, f64)> = balance
        .iter()
        .filter(|(_, b)| (b.out_per_min - b.in_per_min).abs() > 0.001)
        .map(|(id, b)| (id.clone(), b.in_per_min, b.out_per_min))
        .collect();

    shown.sort_by(|(_, in_a, out_a), (_, in_b, out_b)| {
        let net_a = (out_a - in_a).abs();
        let net_b = (out_b - in_b).abs();
        net_b.partial_cmp(&net_a).unwrap_or(std::cmp::Ordering::Equal)
    });

    let has_any_active = balance
        .values()
        .any(|b| b.in_per_min > 0.0 || b.out_per_min > 0.0);

    rsx! {
        div { class: "panel",
            div { class: "panel-title", "Net Balance — per minute" }

            if shown.is_empty() {
                div { class: "empty-balance",
                    if !has_any_active {
                        "No active resources yet."
                        br {}
                        "Add some facilities."
                    } else {
                        "Everything is internally balanced —"
                        br {}
                        "no external inputs or outputs."
                    }
                }
            } else {
                div { class: "balance-list",
                    for (resource_id, in_amt, out_amt) in shown {
                        BalanceRow { resource_id, in_amt, out_amt }
                    }
                }
            }

            div { class: "note-bar",
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
    let net_class = if net > 0.0 { "bal-net pos" } else { "bal-net neg" };
    let sign = if net > 0.0 { "+" } else { "" };
    let net_str = fmt_qty(net);

    rsx! {
        div { class: "bal-row",
            ResIcon { resource_id: resource_id.clone(), size: 18 }
            span { class: "bal-name", "{res.name}" }
            span { class: "{net_class}", "{sign}{net_str}/m" }
        }
    }
}
