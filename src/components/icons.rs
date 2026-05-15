use crate::data::GameData;
use dioxus::prelude::*;
use std::sync::Arc;

// Resource icon: walks the candidate URL list, falls back to a colored swatch
#[component]
pub fn ResIcon(resource_id: String, size: u32) -> Element {
    let game_data = use_context::<Arc<GameData>>();
    let resource = game_data.resources_by_id.get(&resource_id).cloned();
    let mut url_idx = use_signal(|| 0usize);

    let Some(res) = resource else {
        return rsx! { span { style: "width:{size}px;height:{size}px;display:inline-block;" } };
    };

    let icons = res.icon.clone();
    let color = res.color.clone();
    let name = res.name.clone();
    let idx = *url_idx.read();

    if idx < icons.len() {
        let url = icons[idx].clone();
        rsx! {
            img {
                class: "res-icon",
                src: "{url}",
                alt: "{name}",
                title: "{name}",
                width: "{size}",
                height: "{size}",
                style: "width:{size}px;height:{size}px;",
                onerror: move |_| {
                    *url_idx.write() += 1;
                }
            }
        }
    } else {
        rsx! {
            span {
                class: "res-swatch",
                title: "{name}",
                style: "width:{size}px;height:{size}px;background:{color};border-radius:2px;"
            }
        }
    }
}

// Machine icon: walks the candidate URL list, falls back to a two-letter monogram
#[component]
pub fn MachineIcon(building: String, size: u32) -> Element {
    let game_data = use_context::<Arc<GameData>>();
    let machine = game_data.machines_by_name.get(&building).cloned();
    let mut url_idx = use_signal(|| 0usize);

    let icons = machine.map(|m| m.icon).unwrap_or_default();
    let idx = *url_idx.read();

    if idx < icons.len() {
        let url = icons[idx].clone();
        let bld = building.clone();
        rsx! {
            img {
                class: "machine-icon",
                src: "{url}",
                alt: "{bld}",
                title: "{bld}",
                width: "{size}",
                height: "{size}",
                style: "width:{size}px;height:{size}px;",
                onerror: move |_| {
                    *url_idx.write() += 1;
                }
            }
        }
    } else {
        let letters: String = building
            .split_whitespace()
            .filter_map(|w| w.chars().next())
            .take(2)
            .collect::<String>()
            .to_uppercase();
        let font_size = (size as f32 * 0.42) as u32;
        rsx! {
            span {
                class: "machine-mono",
                title: "{building}",
                style: "width:{size}px;height:{size}px;font-size:{font_size}px;",
                "{letters}"
            }
        }
    }
}

// Resource chip: icon + quantity + short label
#[component]
pub fn Chip(resource_id: String, qty: f64, suffix: Option<String>) -> Element {
    let game_data = use_context::<Arc<GameData>>();
    let Some(res) = game_data.resources_by_id.get(&resource_id).cloned() else {
        return rsx! {};
    };

    let qty_str = crate::balance::fmt_qty(qty);
    let short = res.short.clone();
    let suffix_str = suffix.unwrap_or_default();

    rsx! {
        span { class: "chip", title: "{res.name}",
            ResIcon { resource_id: resource_id.clone(), size: 14 }
            span { class: "qty",
                "{qty_str}"
                if !suffix_str.is_empty() {
                    span { class: "qty-suffix", "{suffix_str}" }
                }
            }
            span { class: "lbl", "{short}" }
        }
    }
}
