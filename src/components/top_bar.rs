use crate::data::GameData;
use crate::state::{example_facilities, Facility, PersistState, View};
use dioxus::prelude::*;
use std::sync::Arc;

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
    view: Signal<View>,
) -> Element {
    let game_data = use_context::<Arc<GameData>>();

    rsx! {
        div { class: "topbar",
            div {
                div { class: "title",
                    span { class: "accent", "◆" }
                    " FBR Planner "
                    span { style: "color:var(--text-dim2);font-weight:400;", "· Captain of Industry" }
                }
            }
            div { class: "subtitle", "{fac_count} facilities · {total_units} units" }
            div { class: "spacer" }

            // ── Save ────────────────────────────────────────────────────────
            // Serializes the current state and triggers a browser download.
            button {
                title: "Export current setup as JSON",
                onclick: move |_| {
                    let state = PersistState {
                        facilities: facilities.read().clone(),
                        view: view.read().clone(),
                    };
                    let Ok(json) = serde_json::to_string_pretty(&state) else { return };
                    spawn(async move {
                        let eval = document::eval(
                            "const json = await dioxus.recv();
                                         const blob = new Blob([json], { type: 'application/json' });
                                         const url  = URL.createObjectURL(blob);
                                         const a    = document.createElement('a');
                                         a.href     = url;
                                         a.download = 'fbr-planner.json';
                                         document.body.appendChild(a);
                                         a.click();
                                         document.body.removeChild(a);
                                         setTimeout(() => URL.revokeObjectURL(url), 1000);",
                        );
                        let _ = eval.send(serde_json::Value::String(json));
                    });
                },
                "Save"
            }

            // ── Load ────────────────────────────────────────────────────────
            // Opens a file picker; reads the JSON and replaces current state.
            button {
                title: "Import a setup from a JSON file",
                onclick: move |_| {
                    let gd = game_data.clone();
                    spawn(async move {
                        let mut eval = document::eval(
                            "let resolved = false;
                                         const input  = document.createElement('input');
                                         input.type   = 'file';
                                         input.accept = '.json,application/json';
                                         input.addEventListener('change', async () => {
                                             if (resolved) return;
                                             resolved = true;
                                             const file = input.files[0];
                                             dioxus.send(file ? await file.text() : null);
                                         });
                                         window.addEventListener('focus', function h() {
                                             window.removeEventListener('focus', h);
                                             setTimeout(() => {
                                                 if (!resolved) { resolved = true; dioxus.send(null); }
                                             }, 300);
                                         }, { once: true });
                                         input.click();",
                        );
                        let Ok(val) = eval.recv::<serde_json::Value>().await else { return };
                        let Some(text) = val.as_str() else { return };
                        let Ok(loaded) = serde_json::from_str::<PersistState>(text) else { return };
                        let valid: Vec<Facility> = loaded
                            .facilities
                            .into_iter()
                            .filter(|f| gd.recipes_by_id.contains_key(&f.recipe_id))
                            .collect();
                        facilities.set(valid);
                        view.set(loaded.view);
                    });
                },
                "Load"
            }

            button {
                onclick: move |_| {
                    if !facilities.read().is_empty() {
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
                    button { onclick: move |_| confirm_action.set(None), "Cancel" }
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
