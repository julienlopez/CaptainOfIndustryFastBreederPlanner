use crate::state::{example_facilities, BalanceMode, Facility, View};
use dioxus::prelude::*;

#[derive(Clone, PartialEq)]
pub enum ConfirmAction {
    ClearAll,
    LoadExample,
}

#[component]
pub fn TopBar(
    fac_count: usize,
    total_units: u32,
    confirm_action: Signal<Option<ConfirmAction>>,
    facilities: Signal<Vec<Facility>>,
) -> Element {
    rsx! {
        div { class: "topbar",
            div {
                div { class: "title",
                    span { class: "accent", "◆" }
                    " FBR Planner "
                    span { style: "color:var(--text-dim2);font-weight:400;", "· Captain of Industry" }
                }
            }
            div { class: "subtitle",
                "{fac_count} facilities · {total_units} units"
            }
            div { class: "spacer" }
            button {
                onclick: move |_| {
                    let has_facilities = !facilities.read().is_empty();
                    if has_facilities {
                        confirm_action.set(Some(ConfirmAction::LoadExample));
                    } else {
                        facilities.set(example_facilities());
                    }
                },
                "Example Loop"
            }
            button {
                class: "danger",
                onclick: move |_| {
                    if !facilities.read().is_empty() {
                        confirm_action.set(Some(ConfirmAction::ClearAll));
                    }
                },
                "Clear All"
            }
        }
    }
}

#[component]
pub fn ConfirmDialog(
    action: ConfirmAction,
    confirm_action: Signal<Option<ConfirmAction>>,
    facilities: Signal<Vec<Facility>>,
) -> Element {
    let (title, msg) = match action {
        ConfirmAction::ClearAll => ("Clear All", "Remove all facilities from the canvas?"),
        ConfirmAction::LoadExample => (
            "Load Example Loop",
            "Replace the current layout with the example loop?",
        ),
    };
    let action_clone = action.clone();
    rsx! {
        div { class: "confirm-overlay",
            div { class: "confirm-box",
                div { class: "confirm-title", "{title}" }
                div { class: "confirm-msg", "{msg}" }
                div { class: "confirm-btns",
                    button {
                        onclick: move |_| confirm_action.set(None),
                        "Cancel"
                    }
                    button {
                        class: "confirm-yes",
                        onclick: move |_| {
                            match action_clone {
                                ConfirmAction::ClearAll => facilities.set(vec![]),
                                ConfirmAction::LoadExample => facilities.set(example_facilities()),
                            }
                            confirm_action.set(None);
                        },
                        "Confirm"
                    }
                }
            }
        }
    }
}
