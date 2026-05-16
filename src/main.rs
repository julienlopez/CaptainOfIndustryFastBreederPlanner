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
use state::{example_facilities, Facility, PersistState, View};
use std::sync::Arc;

const CSS: &str = include_str!("../assets/main.css");

fn main() {
    dioxus::launch(App);
}

#[component]
fn App() -> Element {
    // Load game data once at startup
    let game_data: Arc<GameData> = Arc::new(GameData::load());
    provide_context(game_data.clone());

    // Load persisted state or use example defaults
    let init = persistence::load_state().unwrap_or_else(|| PersistState {
        facilities: example_facilities(),
        view: View::Table,
    });

    // Validate: drop facilities whose recipe_id no longer exists
    let valid_facilities: Vec<Facility> = init
        .facilities
        .into_iter()
        .filter(|f| game_data.recipes_by_id.contains_key(&f.recipe_id))
        .collect();

    let facilities: Signal<Vec<Facility>> = use_signal(|| valid_facilities);
    let view: Signal<View> = use_signal(|| init.view);
    let dragging_recipe: Signal<Option<String>> = use_signal(|| None);
    let confirm_action: Signal<Option<ConfirmAction>> = use_signal(|| None);

    // Auto-save on any state change
    use_effect(move || {
        let state = PersistState {
            facilities: facilities.read().clone(),
            view: view.read().clone(),
        };
        persistence::save_state(&state);
    });

    let fac_count = facilities.read().len();
    let total_units: u32 = facilities.read().iter().map(|f| f.count).sum();
    let maybe_action = confirm_action.read().clone();

    rsx! {
        document::Style { "{CSS}" }

        div { class: "app",
            TopBar {
                fac_count: fac_count,
                total_units: total_units,
                confirm_action: confirm_action,
                facilities: facilities,
                view: view,
            }

            div { class: "main",
                RecipePalette {
                    facilities: facilities,
                    dragging_recipe: dragging_recipe,
                }
                Canvas {
                    facilities: facilities,
                    view: view,
                    dragging_recipe: dragging_recipe,
                }
                BalancePanel {
                    facilities: facilities,
                }
            }
        }

        // Confirm dialog overlay
        if let Some(action) = maybe_action {
            ConfirmDialog {
                action: action,
                confirm_action: confirm_action,
                facilities: facilities,
            }
        }
    }
}
