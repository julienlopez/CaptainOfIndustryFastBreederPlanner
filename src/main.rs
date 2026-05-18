mod balance;
mod components;
mod data;
mod persistence;
mod state;

use components::balance_panel::BalancePanel;
use components::canvas::Canvas;
use components::recipe_palette::RecipePalette;
use components::top_bar::{ConfirmAction, ConfirmDialog, TopBar};
use data::GameData;
use dioxus::prelude::*;
use state::{example_facilities, Facility, PersistState, TabData, View};
use std::sync::Arc;

const CSS: &str = include_str!("../assets/main.css");

fn main() {
    dioxus::launch(App);
}

#[component]
fn App() -> Element {
    let game_data: Arc<GameData> = Arc::new(GameData::load());
    provide_context(game_data.clone());

    // Load and migrate persisted state
    let (init_tabs, init_active) = match persistence::load_state() {
        Some(saved) if !saved.tabs.is_empty() => {
            let mut tabs = saved.tabs;
            for tab in &mut tabs {
                tab.facilities
                    .retain(|f| game_data.recipes_by_id.contains_key(&f.recipe_id));
            }
            let idx = saved.active_tab.min(tabs.len().saturating_sub(1));
            (tabs, idx)
        }
        Some(saved) if !saved.facilities.is_empty() => {
            // Legacy single-tab save
            let valid: Vec<Facility> = saved
                .facilities
                .into_iter()
                .filter(|f| game_data.recipes_by_id.contains_key(&f.recipe_id))
                .collect();
            (
                vec![TabData {
                    name: "Tab 1".to_string(),
                    facilities: valid,
                    view: saved.view,
                }],
                0,
            )
        }
        _ => (
            vec![TabData {
                name: "Tab 1".to_string(),
                facilities: example_facilities(),
                view: View::Table,
            }],
            0,
        ),
    };

    let mut tabs: Signal<Vec<TabData>> = use_signal(|| init_tabs.clone());
    let mut active_tab: Signal<usize> = use_signal(|| init_active);

    // Live signals for the active tab — all child components use these
    let mut facilities: Signal<Vec<Facility>> =
        use_signal(|| init_tabs[init_active].facilities.clone());
    let mut view: Signal<View> = use_signal(|| init_tabs[init_active].view.clone());

    let dragging_recipe: Signal<Option<String>> = use_signal(|| None);
    let confirm_action: Signal<Option<ConfirmAction>> = use_signal(|| None);

    // Auto-save: sync active tab data and persist everything
    use_effect(move || {
        let cur_facs = facilities.read().clone();
        let cur_view = view.read().clone();
        let idx = *active_tab.read();
        let mut all_tabs = tabs.read().clone();
        if let Some(tab) = all_tabs.get_mut(idx) {
            tab.facilities = cur_facs;
            tab.view = cur_view;
        }
        persistence::save_state(&PersistState {
            tabs: all_tabs,
            active_tab: idx,
            ..Default::default()
        });
    });

    let fac_count = facilities.read().len();
    let total_units: u32 = facilities.read().iter().map(|f| f.count).sum();
    let maybe_action = confirm_action.read().clone();

    // Build the tab bar items from a snapshot to avoid holding the read guard
    let tab_names: Vec<String> = tabs.read().iter().map(|t| t.name.clone()).collect();
    let active = *active_tab.read();
    let num_tabs = tab_names.len();

    rsx! {
        document::Style { "{CSS}" }

        div { class: "app",
            TopBar {
                fac_count,
                total_units,
                confirm_action,
                facilities,
                view,
                tabs,
                active_tab,
            }

            div { class: "main",
                RecipePalette {
                    facilities,
                    dragging_recipe,
                }

                div { class: "tab-workspace",
                    // ── Tab bar ──────────────────────────────────────────────
                    div { class: "tab-bar",
                        for (i , name) in tab_names.into_iter().enumerate() {
                            div {
                                key: "{i}",
                                class: if i == active { "tab-item active" } else { "tab-item" },
                                onclick: move |_| {
                                    if i == *active_tab.read() {
                                        return;
                                    }
                                    // Snapshot current tab
                                    {
                                        let idx = *active_tab.read();
                                        let mut tw = tabs.write();
                                        if let Some(t) = tw.get_mut(idx) {
                                            t.facilities = facilities.read().clone();
                                            t.view = view.read().clone();
                                        }
                                    }
                                    let (nf, nv) = {
                                        let tr = tabs.read();
                                        let t = &tr[i];
                                        (t.facilities.clone(), t.view.clone())
                                    };
                                    facilities.set(nf);
                                    view.set(nv);
                                    active_tab.set(i);
                                },
                                span { class: "tab-name", "{name}" }
                                if num_tabs > 1 {
                                    button {
                                        class: "tab-close",
                                        onclick: move |e| {
                                            e.stop_propagation();
                                            let old_active = *active_tab.read();
                                            if old_active == i {
                                                // Closing the active tab — switch to neighbour first
                                                let neighbour = if i > 0 { i - 1 } else { 1 };
                                                let (nf, nv) = {
                                                    let tr = tabs.read();
                                                    (
                                                        tr[neighbour].facilities.clone(),
                                                        tr[neighbour].view.clone(),
                                                    )
                                                };
                                                facilities.set(nf);
                                                view.set(nv);
                                                tabs.write().remove(i);
                                                active_tab.set(if i > 0 { i - 1 } else { 0 });
                                            } else {
                                                tabs.write().remove(i);
                                                if old_active > i {
                                                    active_tab.set(old_active - 1);
                                                }
                                            }
                                        },
                                        "×"
                                    }
                                }
                            }
                        }

                        button {
                            class: "tab-add",
                            title: "New tab",
                            onclick: move |_| {
                                // Snapshot current tab
                                {
                                    let idx = *active_tab.read();
                                    let mut tw = tabs.write();
                                    if let Some(t) = tw.get_mut(idx) {
                                        t.facilities = facilities.read().clone();
                                        t.view = view.read().clone();
                                    }
                                }
                                let n = {
                                    let mut tw = tabs.write();
                                    let n = tw.len() + 1;
                                    tw.push(TabData {
                                        name: format!("Tab {}", n),
                                        facilities: vec![],
                                        view: View::Table,
                                    });
                                    n
                                };
                                facilities.set(vec![]);
                                view.set(View::Table);
                                active_tab.set(n - 1);
                            },
                            "+"
                        }
                    }

                    // ── Tab content ──────────────────────────────────────────
                    div { class: "tab-content",
                        Canvas { facilities, view, dragging_recipe }
                        BalancePanel { facilities }
                    }
                }
            }
        }

        if let Some(action) = maybe_action {
            ConfirmDialog {
                action,
                confirm_action,
                facilities,
            }
        }
    }
}
