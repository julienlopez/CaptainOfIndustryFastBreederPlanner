use crate::state::PersistState;
use std::path::PathBuf;

fn state_path() -> Option<PathBuf> {
    dirs::config_dir().map(|d| d.join("coi_fbr_planner").join("state.json"))
}

pub fn load_state() -> Option<PersistState> {
    let path = state_path()?;
    let data = std::fs::read_to_string(path).ok()?;
    serde_json::from_str(&data).ok()
}

pub fn save_state(state: &PersistState) {
    let Some(path) = state_path() else { return };
    if let Some(parent) = path.parent() {
        let _ = std::fs::create_dir_all(parent);
    }
    if let Ok(json) = serde_json::to_string_pretty(state) {
        let _ = std::fs::write(path, json);
    }
}
