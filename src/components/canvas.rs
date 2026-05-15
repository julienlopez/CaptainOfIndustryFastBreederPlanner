use super::graph_view::GraphView;
use super::table_view::TableView;
use crate::data::GameData;
use crate::state::{new_uid, Facility, View};
use dioxus::prelude::*;
use std::sync::Arc;

#[component]
pub fn Canvas(
    facilities: Signal<Vec<Facility>>,
    view: Signal<View>,
    dragging_recipe: Signal<Option<String>>,
) -> Element {
    let game_data = use_context::<Arc<GameData>>();
    let mut drop_active = use_signal(|| false);
    let mut drag_counter = use_signal(|| 0i32);

    let current_view = view.read().clone();
    let is_empty = facilities.read().is_empty();

    let canvas_class = {
        let base = match current_view {
            View::Table => "canvas table",
            View::Graph => "canvas graph",
        };
        if *drop_active.read() {
            format!("{} drop-active", base)
        } else {
            base.to_string()
        }
    };

    let gd = game_data.clone();
    let gd2 = game_data.clone();

    rsx! {
        div { class: "canvas-wrap",
            // Toolbar
            div { class: "view-toolbar",
                div { class: "view-switch",
                    button {
                        class: if current_view == View::Table { "active" } else { "" },
                        onclick: move |_| view.set(View::Table),
                        "▦ Table"
                    }
                    button {
                        class: if current_view == View::Graph { "active" } else { "" },
                        onclick: move |_| view.set(View::Graph),
                        "◌ Graph"
                    }
                }
                if current_view == View::Graph && !is_empty {
                    button {
                        class: "view-action",
                        title: "Re-layout nodes",
                        onclick: move |_| {
                            let mut facs = facilities.write();
                            for f in facs.iter_mut() {
                                f.x = None;
                                f.y = None;
                            }
                        },
                        "↻ Auto-layout"
                    }
                }
            }

            // Canvas drop zone
            div {
                class: "{canvas_class}",
                ondragenter: move |e| {
                    e.prevent_default();
                    *drag_counter.write() += 1;
                    drop_active.set(true);
                },
                ondragleave: move |_| {
                    *drag_counter.write() -= 1;
                    if *drag_counter.read() <= 0 {
                        drag_counter.set(0);
                        drop_active.set(false);
                    }
                },
                ondragover: move |e| {
                    e.prevent_default();
                },
                ondrop: move |e| {
                    e.prevent_default();
                    drag_counter.set(0);
                    drop_active.set(false);
                    let recipe_id = dragging_recipe.read().clone();
                    if let Some(recipe_id) = recipe_id {
                        if gd.recipes_by_id.contains_key(&recipe_id) {
                            let uid = new_uid();
                            facilities.write().push(Facility {
                                uid,
                                recipe_id,
                                count: 1,
                                x: None,
                                y: None,
                            });
                        }
                        dragging_recipe.set(None);
                    }
                },

                if is_empty {
                    div { class: "canvas-empty",
                        div { class: "empty-icon", "⬚" }
                        div { class: "big", "Drop recipes here" }
                        div { "Drag any recipe from the left." br {} "Set unit counts, watch the balance." }
                    }
                } else {
                    match current_view {
                        View::Table => rsx! {
                            TableView { facilities: facilities }
                        },
                        View::Graph => rsx! {
                            GraphView { facilities: facilities }
                        },
                    }
                }

                div { class: "drop-hint", "Release to add facility" }
            }
        }
    }
}
