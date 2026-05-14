use serde::{Deserialize, Serialize};
use std::time::{SystemTime, UNIX_EPOCH};

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Default)]
pub enum View {
    #[default]
    Table,
    Graph,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Default)]
pub enum BalanceMode {
    #[default]
    All,
    External,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct Facility {
    pub uid: String,
    pub recipe_id: String,
    pub count: u32,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub x: Option<f32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub y: Option<f32>,
}

#[derive(Serialize, Deserialize, Clone, Debug, Default)]
pub struct PersistState {
    pub facilities: Vec<Facility>,
    pub view: View,
    pub balance_mode: BalanceMode,
}

pub fn new_uid() -> String {
    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .subsec_nanos();
    // Mix in a pseudo-random component to avoid collisions on fast calls
    static COUNTER: std::sync::atomic::AtomicU32 = std::sync::atomic::AtomicU32::new(0);
    let seq = COUNTER.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
    format!("f{}_{}", nanos, seq)
}

pub fn example_facilities() -> Vec<Facility> {
    vec![
        Facility { uid: "ex1".into(), recipe_id: "fbr_1x_1".into(), count: 4, x: None, y: None },
        Facility { uid: "ex2".into(), recipe_id: "reprocess_cf".into(), count: 1, x: None, y: None },
        Facility { uid: "ex3".into(), recipe_id: "enr_ebf_cf".into(), count: 1, x: None, y: None },
        Facility { uid: "ex4".into(), recipe_id: "chem_blanket_from_yc".into(), count: 1, x: None, y: None },
        Facility { uid: "ex5".into(), recipe_id: "turb_sp".into(), count: 16, x: None, y: None },
        Facility { uid: "ex6".into(), recipe_id: "cool_sp".into(), count: 2, x: None, y: None },
    ]
}
