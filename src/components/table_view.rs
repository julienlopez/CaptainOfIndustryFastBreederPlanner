use super::icons::{Chip, MachineIcon};
use crate::balance::fmt_qty;
use crate::data::GameData;
use crate::state::Facility;
use dioxus::prelude::*;
use std::sync::Arc;

#[component]
pub fn TableView(facilities: Signal<Vec<Facility>>) -> Element {
    let game_data = use_context::<Arc<GameData>>();
    let fac_list = facilities.read().clone();

    rsx! {
        div { class: "facility-grid",
            for facility in fac_list {
                FacilityCard {
                    facility: facility.clone(),
                    facilities: facilities,
                }
            }
        }
    }
}

#[component]
fn FacilityCard(facility: Facility, facilities: Signal<Vec<Facility>>) -> Element {
    let game_data = use_context::<Arc<GameData>>();
    let uid = facility.uid.clone();
    let uid2 = uid.clone();

    let Some(recipe) = game_data.recipes_by_id.get(&facility.recipe_id).cloned() else {
        return rsx! {};
    };
    let cat_color = game_data
        .categories_by_id
        .get(&recipe.category)
        .map(|c| c.color.clone())
        .unwrap_or_else(|| "#f59e0b".to_string());

    let mul = (60.0 / recipe.duration as f64) * facility.count as f64;
    let count = facility.count;

    rsx! {
        div { class: "facility", style: "--cat-color:{cat_color};",
            div { class: "fac-header",
                MachineIcon { building: recipe.building.clone(), size: 32 }
                div { class: "fac-header-text",
                    div { class: "fac-building", "{recipe.building}" }
                    div { class: "fac-name",
                        "{recipe.name}"
                        if !recipe.exact {
                            span { class: "rc-approx", style: "margin-left:6px;", "~APX" }
                        }
                    }
                }
                button {
                    class: "fac-close",
                    title: "Remove",
                    onclick: move |_| {
                        facilities.write().retain(|f| f.uid != uid);
                    },
                    "×"
                }
            }
            div { class: "fac-body",
                div { class: "fac-io",
                    div { class: "fac-io-row",
                        span { class: "fac-io-label in", "IN" }
                        if recipe.inputs.is_empty() {
                            span { style: "color:var(--text-dim2);font-size:11px;font-family:var(--mono);", "—" }
                        }
                        for io in &recipe.inputs {
                            Chip { resource_id: io.resource.clone(), qty: io.qty * mul, suffix: Some("/m".into()) }
                        }
                    }
                    div { class: "fac-io-row",
                        span { class: "fac-io-label out", "OUT" }
                        if recipe.outputs.is_empty() {
                            span { style: "color:var(--text-dim2);font-size:11px;font-family:var(--mono);", "—" }
                        }
                        for io in &recipe.outputs {
                            Chip { resource_id: io.resource.clone(), qty: io.qty * mul, suffix: Some("/m".into()) }
                        }
                    }
                }
                div { class: "fac-count",
                    span { class: "fac-count-label", "Units" }
                    CountStepper {
                        count: count,
                        uid: uid2.clone(),
                        facilities: facilities,
                    }
                }
            }
        }
    }
}

#[component]
pub fn CountStepper(count: u32, uid: String, facilities: Signal<Vec<Facility>>) -> Element {
    let uid_dec = uid.clone();
    let uid_inc = uid.clone();
    let uid_inp = uid.clone();

    rsx! {
        div { class: "fac-count-controls",
            button {
                onclick: move |_| {
                    let mut facs = facilities.write();
                    if let Some(f) = facs.iter_mut().find(|f| f.uid == uid_dec) {
                        f.count = f.count.saturating_sub(1);
                    }
                },
                "−"
            }
            input {
                r#type: "number",
                min: "0",
                value: "{count}",
                oninput: move |e| {
                    let val = e.value().parse::<u32>().unwrap_or(0);
                    let mut facs = facilities.write();
                    if let Some(f) = facs.iter_mut().find(|f| f.uid == uid_inp) {
                        f.count = val;
                    }
                }
            }
            button {
                onclick: move |_| {
                    let mut facs = facilities.write();
                    if let Some(f) = facs.iter_mut().find(|f| f.uid == uid_inc) {
                        f.count += 1;
                    }
                },
                "+"
            }
        }
    }
}
