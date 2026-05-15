use super::icons::{Chip, MachineIcon};
use crate::data::GameData;
use crate::state::Facility;
use dioxus::prelude::*;
use std::sync::Arc;

#[component]
pub fn RecipePalette(
    facilities: Signal<Vec<Facility>>,
    dragging_recipe: Signal<Option<String>>,
) -> Element {
    let game_data = use_context::<Arc<GameData>>();
    let categories = game_data.categories.clone();

    rsx! {
        div { class: "panel",
            div { class: "panel-title", "Recipes — drag to canvas" }
            for cat in categories {
                CategoryGroup {
                    category_id: cat.id.clone(),
                    facilities,
                    dragging_recipe,
                }
            }
        }
    }
}

#[component]
fn CategoryGroup(
    category_id: String,
    facilities: Signal<Vec<Facility>>,
    dragging_recipe: Signal<Option<String>>,
) -> Element {
    let game_data = use_context::<Arc<GameData>>();
    let Some(cat) = game_data.categories_by_id.get(&category_id).cloned() else {
        return rsx! {};
    };
    let recipes = game_data
        .recipes_by_category
        .get(&category_id)
        .cloned()
        .unwrap_or_default();

    if recipes.is_empty() {
        return rsx! {};
    }

    rsx! {
        div { class: "cat-group",
            div { class: "cat-header",
                span { class: "cat-dot", style: "background:{cat.color};" }
                "{cat.name}"
            }
            for recipe in recipes {
                RecipeCard { recipe_id: recipe.id.clone(), dragging_recipe }
            }
        }
    }
}

#[component]
fn RecipeCard(recipe_id: String, dragging_recipe: Signal<Option<String>>) -> Element {
    let game_data = use_context::<Arc<GameData>>();
    let Some(recipe) = game_data.recipes_by_id.get(&recipe_id).cloned() else {
        return rsx! {};
    };
    let cat_color = game_data
        .categories_by_id
        .get(&recipe.category)
        .map(|c| c.color.clone())
        .unwrap_or_else(|| "#f59e0b".to_string());

    let is_dragging = dragging_recipe.read().as_deref() == Some(&recipe_id);
    let card_class = if is_dragging {
        "recipe-card dragging"
    } else {
        "recipe-card"
    };

    let rid = recipe_id.clone();

    rsx! {
        div {
            class: "{card_class}",
            style: "--cat-color:{cat_color};",
            draggable: "true",
            ondragstart: move |e| {
                e.stop_propagation();
                dragging_recipe.set(Some(rid.clone()));
            },
            ondragend: move |_| {
                dragging_recipe.set(None);
            },

            div { class: "rc-top",
                MachineIcon { building: recipe.building.clone(), size: 28 }
                div { class: "rc-top-text",
                    div { class: "rc-building", "{recipe.building}" }
                    div { class: "rc-name",
                        "{recipe.name}"
                        if !recipe.exact {
                            span {
                                class: "rc-approx",
                                title: "Approximate values — verify in-game",
                                "~APX"
                            }
                        }
                    }
                }
            }
            div { class: "rc-flow",
                for io in &recipe.inputs {
                    Chip {
                        resource_id: io.resource.clone(),
                        qty: io.qty,
                        suffix: None,
                    }
                }
                if !recipe.inputs.is_empty() && !recipe.outputs.is_empty() {
                    span { class: "arrow", "→" }
                }
                for io in &recipe.outputs {
                    Chip {
                        resource_id: io.resource.clone(),
                        qty: io.qty,
                        suffix: None,
                    }
                }
            }
        }
    }
}
