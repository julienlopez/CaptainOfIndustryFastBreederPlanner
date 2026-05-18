use crate::data::GameData;
use crate::state::{example_facilities, Facility, PersistState, TabData, View};
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
    tabs: Signal<Vec<TabData>>,
    active_tab: Signal<usize>,
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

            // ── Save current tab ────────────────────────────────────────────
            button {
                title: "Export current tab as JSON",
                onclick: move |_| {
                    let idx = *active_tab.read();
                    let tab_name = tabs.read().get(idx).map(|t| t.name.clone()).unwrap_or_else(|| "tab".to_string());
                    let state = PersistState {
                        tabs: vec![TabData {
                            name: tab_name.clone(),
                            facilities: facilities.read().clone(),
                            view: view.read().clone(),
                        }],
                        active_tab: 0,
                        ..Default::default()
                    };
                    let Ok(json) = serde_json::to_string_pretty(&state) else { return };
                    let filename = format!("fbr-{}.json", tab_name.to_lowercase().replace(' ', "-"));
                    spawn(async move {
                        let eval = document::eval(
                            "const [json, name] = await dioxus.recv();
                             const blob = new Blob([json], { type: 'application/json' });
                             const url  = URL.createObjectURL(blob);
                             const a    = document.createElement('a');
                             a.href     = url;
                             a.download = name;
                             document.body.appendChild(a);
                             a.click();
                             document.body.removeChild(a);
                             setTimeout(() => URL.revokeObjectURL(url), 1000);",
                        );
                        let _ = eval.send(serde_json::json!([json, filename]));
                    });
                },
                "Save Tab"
            }

            // ── Save all tabs ────────────────────────────────────────────────
            button {
                title: "Export all tabs as JSON",
                onclick: move |_| {
                    // Sync active tab into the snapshot before exporting
                    let idx = *active_tab.read();
                    let mut all_tabs = tabs.read().clone();
                    if let Some(tab) = all_tabs.get_mut(idx) {
                        tab.facilities = facilities.read().clone();
                        tab.view = view.read().clone();
                    }
                    let state = PersistState {
                        tabs: all_tabs,
                        active_tab: idx,
                        ..Default::default()
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
                "Save All Tabs"
            }

            // ── Load ────────────────────────────────────────────────────────
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
                        let Ok(loaded) = serde_json::from_str::<PersistState>(text) else {
                            return;
                        };

                        if !loaded.tabs.is_empty() {
                            // New multi-tab format
                            let mut new_tabs: Vec<TabData> = loaded
                                .tabs
                                .into_iter()
                                .map(|mut tab| {
                                    tab.facilities
                                        .retain(|f| gd.recipes_by_id.contains_key(&f.recipe_id));
                                    tab
                                })
                                .collect();
                            if new_tabs.is_empty() {
                                new_tabs.push(TabData {
                                    name: "Tab 1".to_string(),
                                    facilities: vec![],
                                    view: View::Table,
                                });
                            }
                            let new_active = loaded.active_tab.min(new_tabs.len() - 1);
                            let (nf, nv) = (
                                new_tabs[new_active].facilities.clone(),
                                new_tabs[new_active].view.clone(),
                            );
                            tabs.set(new_tabs);
                            facilities.set(nf);
                            view.set(nv);
                            active_tab.set(new_active);
                        } else {
                            // Legacy single-tab format
                            let valid: Vec<Facility> = loaded
                                .facilities
                                .into_iter()
                                .filter(|f| gd.recipes_by_id.contains_key(&f.recipe_id))
                                .collect();
                            let new_view = loaded.view;
                            tabs.set(vec![TabData {
                                name: "Tab 1".to_string(),
                                facilities: valid.clone(),
                                view: new_view.clone(),
                            }]);
                            facilities.set(valid);
                            view.set(new_view);
                            active_tab.set(0);
                        }
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
