use super::icons::{MachineIcon, ResIcon};
use super::table_view::CountStepper;
use crate::balance::fmt_qty;
use crate::data::GameData;
use crate::state::Facility;
use dioxus::prelude::*;
use std::sync::Arc;

const NODE_W: f32 = 240.0;
const NODE_H_HEADER: f32 = 44.0;
const NODE_H_BODY_BASE: f32 = 26.0; // per port row
const NODE_H_FOOT: f32 = 40.0;
const COL_COUNT: usize = 3;
const GAP_X: f32 = 60.0;
const GAP_Y: f32 = 60.0;
const ORIGIN: f32 = 40.0;

fn auto_pos(idx: usize) -> (f32, f32) {
    let col = idx % COL_COUNT;
    let row = idx / COL_COUNT;
    (
        ORIGIN + col as f32 * (NODE_W + GAP_X),
        ORIGIN + row as f32 * (200.0 + GAP_Y),
    )
}

fn node_height(port_rows: usize) -> f32 {
    NODE_H_HEADER + port_rows.max(1) as f32 * NODE_H_BODY_BASE + NODE_H_FOOT + 32.0
}

#[derive(Clone, Debug)]
struct NodeDrag {
    uid: String,
    start_mouse_x: f64,
    start_mouse_y: f64,
    start_x: f32,
    start_y: f32,
    live_x: f32,
    live_y: f32,
}

#[derive(Clone, Debug)]
struct Edge {
    from: String,
    to: String,
    resource_id: String,
    rate: f64,
}

#[component]
pub fn GraphView(facilities: Signal<Vec<Facility>>) -> Element {
    let game_data = use_context::<Arc<GameData>>();
    let mut node_drag: Signal<Option<NodeDrag>> = use_signal(|| None);

    let facs = facilities.read().clone();

    // Compute positions (use stored or auto-layout)
    let positions: Vec<(String, f32, f32)> = facs
        .iter()
        .enumerate()
        .map(|(i, f)| {
            if let (Some(x), Some(y)) = (f.x, f.y) {
                (f.uid.clone(), x, y)
            } else {
                let (x, y) = auto_pos(i);
                (f.uid.clone(), x, y)
            }
        })
        .collect();

    // Override with live drag position
    let positions: Vec<(String, f32, f32)> = positions
        .into_iter()
        .map(|(uid, x, y)| {
            if let Some(drag) = node_drag.read().as_ref() {
                if drag.uid == uid {
                    return (uid, drag.live_x, drag.live_y);
                }
            }
            (uid, x, y)
        })
        .collect();

    // Compute node heights
    let heights: Vec<(String, f32)> = facs
        .iter()
        .map(|f| {
            let rows = game_data
                .recipes_by_id
                .get(&f.recipe_id)
                .map(|r| r.inputs.len().max(r.outputs.len()))
                .unwrap_or(1);
            (f.uid.clone(), node_height(rows))
        })
        .collect();

    // Compute edges
    let mut edges: Vec<Edge> = Vec::new();
    for prod in &facs {
        let Some(pr) = game_data.recipes_by_id.get(&prod.recipe_id) else {
            continue;
        };
        let prod_mul = (60.0 / pr.duration as f64) * prod.count as f64;
        for output in &pr.outputs {
            for cons in &facs {
                if cons.uid == prod.uid {
                    continue;
                }
                let Some(cr) = game_data.recipes_by_id.get(&cons.recipe_id) else {
                    continue;
                };
                if cr.inputs.iter().any(|i| i.resource == output.resource) {
                    edges.push(Edge {
                        from: prod.uid.clone(),
                        to: cons.uid.clone(),
                        resource_id: output.resource.clone(),
                        rate: output.qty * prod_mul,
                    });
                }
            }
        }
    }

    // Compute canvas extents
    let max_x = positions
        .iter()
        .map(|(_, x, _)| x + NODE_W + 100.0)
        .fold(800.0f32, f32::max);
    let max_y = positions
        .iter()
        .zip(heights.iter())
        .map(|((_, _, y), (_, h))| y + h + 100.0)
        .fold(600.0f32, f32::max);
    let canvas_w = max_x as u32;
    let canvas_h = max_y as u32;

    // Build edge paths
    struct EdgePath {
        path: String,
        mid_x: f32,
        mid_y: f32,
        resource_id: String,
        rate: f64,
    }

    let edge_paths: Vec<EdgePath> = edges
        .iter()
        .filter_map(|e| {
            let (_, x1_base, y1_base) = positions.iter().find(|(uid, _, _)| uid == &e.from)?;
            let (_, x2_base, y2_base) = positions.iter().find(|(uid, _, _)| uid == &e.to)?;
            let h1 = heights
                .iter()
                .find(|(uid, _)| uid == &e.from)
                .map(|(_, h)| *h)
                .unwrap_or(200.0);
            let h2 = heights
                .iter()
                .find(|(uid, _)| uid == &e.to)
                .map(|(_, h)| *h)
                .unwrap_or(200.0);

            let x1 = x1_base + NODE_W;
            let y1 = y1_base + h1 / 2.0;
            let x2 = *x2_base;
            let y2 = y2_base + h2 / 2.0;
            let dx = ((x2 - x1).abs() * 0.5).max(60.0);
            let c1x = x1 + dx;
            let c2x = x2 - dx;

            let path = format!(
                "M {:.1} {:.1} C {:.1} {:.1}, {:.1} {:.1}, {:.1} {:.1}",
                x1, y1, c1x, y1, c2x, y2, x2, y2
            );
            let mid_x = (x1 + x2) / 2.0;
            let mid_y = 0.125 * y1 + 0.375 * y1 + 0.375 * y2 + 0.125 * y2;

            Some(EdgePath {
                path,
                mid_x,
                mid_y,
                resource_id: e.resource_id.clone(),
                rate: e.rate,
            })
        })
        .collect();

    // Pointer move handler (for dragging)
    let handle_ptr_move = move |e: Event<PointerData>| {
        let mut drag = node_drag.write();
        if let Some(d) = drag.as_mut() {
            let mx = e.client_coordinates().x;
            let my = e.client_coordinates().y;
            let nx = (d.start_x + (mx as f32 - d.start_mouse_x as f32)).max(0.0);
            let ny = (d.start_y + (my as f32 - d.start_mouse_y as f32)).max(0.0);
            d.live_x = nx;
            d.live_y = ny;
        }
    };

    let handle_ptr_up = move |_: Event<PointerData>| {
        let drag_data = node_drag.read().clone();
        if let Some(d) = drag_data {
            let mut facs = facilities.write();
            if let Some(f) = facs.iter_mut().find(|f| f.uid == d.uid) {
                f.x = Some(d.live_x.round());
                f.y = Some(d.live_y.round());
            }
        }
        node_drag.set(None);
    };

    rsx! {
        div {
            class: "g-scroll",
            onpointermove: handle_ptr_move,
            onpointerup: handle_ptr_up,
            onpointerleave: move |_| {
                // Commit position if pointer leaves canvas
                let drag_data = node_drag.read().clone();
                if let Some(d) = drag_data {
                    let mut facs = facilities.write();
                    if let Some(f) = facs.iter_mut().find(|f| f.uid == d.uid) {
                        f.x = Some(d.live_x.round());
                        f.y = Some(d.live_y.round());
                    }
                }
                node_drag.set(None);
            },

            div {
                class: "g-canvas",
                style: "width:{canvas_w}px;height:{canvas_h}px;",

                // SVG edges layer
                svg {
                    class: "g-edges",
                    width: "{canvas_w}",
                    height: "{canvas_h}",
                    defs {
                        for (res_id, res) in game_data.resources_by_id.iter() {
                            marker {
                                id: "arrow-{res_id}",
                                "viewBox": "0 0 10 10",
                                "refX": "9",
                                "refY": "5",
                                "markerWidth": "6",
                                "markerHeight": "6",
                                orient: "auto",
                                path {
                                    d: "M0,0 L10,5 L0,10 Z",
                                    fill: "{res.color}",
                                }
                            }
                        }
                    }
                    for ep in edge_paths.iter() {
                        {
                            let color = game_data.resources_by_id.get(&ep.resource_id)
                                .map(|r| r.color.as_str())
                                .unwrap_or("#666");
                            let rid = &ep.resource_id;
                            rsx! {
                                path {
                                    d: "{ep.path}",
                                    stroke: "{color}",
                                    "stroke-width": "2",
                                    fill: "none",
                                    "stroke-opacity": "0.55",
                                    "marker-end": "url(#arrow-{rid})",
                                }
                            }
                        }
                    }
                }

                // Edge labels
                for ep in edge_paths.iter() {
                    {
                        let res = game_data.resources_by_id.get(&ep.resource_id).cloned();
                        let short = res.as_ref().map(|r| r.short.as_str()).unwrap_or("");
                        let rate_str = fmt_qty(ep.rate);
                        let mx = ep.mid_x as i32;
                        let my = ep.mid_y as i32;
                        let rid = ep.resource_id.clone();
                        rsx! {
                            div {
                                class: "g-edge-lbl",
                                style: "left:{mx}px;top:{my}px;",
                                ResIcon { resource_id: rid, size: 12 }
                                span { class: "g-edge-qty", "{rate_str}" }
                                span { class: "g-edge-res", "{short}/m" }
                            }
                        }
                    }
                }

                // Graph nodes
                for (facility, (_, pos_x, pos_y)) in facs.iter().zip(positions.iter()) {
                    {
                        let f = facility.clone();
                        let uid = f.uid.clone();
                        let px = *pos_x as i32;
                        let py = *pos_y as i32;
                        let uid_drag = uid.clone();
                        let start_x = *pos_x;
                        let start_y = *pos_y;

                        rsx! {
                            GraphNode {
                                facility: f,
                                pos_x: px,
                                pos_y: py,
                                facilities: facilities,
                                on_header_pointer_down: move |e: Event<PointerData>| {
                                    node_drag.set(Some(NodeDrag {
                                        uid: uid_drag.clone(),
                                        start_mouse_x: e.client_coordinates().x,
                                        start_mouse_y: e.client_coordinates().y,
                                        start_x,
                                        start_y,
                                        live_x: start_x,
                                        live_y: start_y,
                                    }));
                                },
                            }
                        }
                    }
                }
            }
        }
    }
}

#[component]
fn GraphNode(
    facility: Facility,
    pos_x: i32,
    pos_y: i32,
    facilities: Signal<Vec<Facility>>,
    on_header_pointer_down: EventHandler<Event<PointerData>>,
) -> Element {
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
        div {
            class: "g-node",
            style: "left:{pos_x}px;top:{pos_y}px;width:{NODE_W as i32}px;--cat-color:{cat_color};",

            div {
                class: "g-node-header",
                onpointerdown: move |e| {
                    e.prevent_default();
                    on_header_pointer_down.call(e);
                },
                MachineIcon { building: recipe.building.clone(), size: 26 }
                div { class: "g-node-header-text",
                    div { class: "g-node-building", "{recipe.building}" }
                    div { class: "g-node-name", "{recipe.name}" }
                }
                button {
                    class: "fac-close",
                    onpointerdown: move |e| e.stop_propagation(),
                    onclick: move |_| {
                        facilities.write().retain(|f| f.uid != uid);
                    },
                    "×"
                }
            }

            div { class: "g-node-body",
                div { class: "g-node-ports",
                    div { class: "g-node-port-col in",
                        if recipe.inputs.is_empty() {
                            div { class: "g-port-empty", "—" }
                        }
                        for io in &recipe.inputs {
                            {
                                let res = game_data.resources_by_id.get(&io.resource).cloned();
                                let short = res.as_ref().map(|r| r.short.as_str()).unwrap_or("");
                                let qty_str = fmt_qty(io.qty * mul);
                                let rid = io.resource.clone();
                                let rid2 = rid.clone();
                                let rname = res.as_ref().map(|r| r.name.as_str()).unwrap_or("");
                                rsx! {
                                    div { class: "g-port", "data-resource": "{rid2}", "data-dir": "in", title: "{rname}",
                                        ResIcon { resource_id: rid, size: 14 }
                                        span { class: "g-port-qty", "{qty_str}" }
                                        span { class: "g-port-lbl", "{short}" }
                                    }
                                }
                            }
                        }
                    }
                    div { class: "g-node-port-col out",
                        if recipe.outputs.is_empty() {
                            div { class: "g-port-empty", "—" }
                        }
                        for io in &recipe.outputs {
                            {
                                let res = game_data.resources_by_id.get(&io.resource).cloned();
                                let short = res.as_ref().map(|r| r.short.as_str()).unwrap_or("");
                                let qty_str = fmt_qty(io.qty * mul);
                                let rid = io.resource.clone();
                                let rid2 = rid.clone();
                                let rname = res.as_ref().map(|r| r.name.as_str()).unwrap_or("");
                                rsx! {
                                    div { class: "g-port", "data-resource": "{rid2}", "data-dir": "out", title: "{rname}",
                                        span { class: "g-port-lbl", "{short}" }
                                        span { class: "g-port-qty", "{qty_str}" }
                                        ResIcon { resource_id: rid, size: 14 }
                                    }
                                }
                            }
                        }
                    }
                }
                div { class: "g-node-foot",
                    CountStepper { count: count, uid: uid2.clone(), facilities: facilities }
                    span { class: "g-node-unit-lbl", "× units" }
                }
            }
        }
    }
}
