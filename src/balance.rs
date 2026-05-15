use crate::data::Recipe;
use crate::state::Facility;
use std::collections::HashMap;

#[derive(Debug, Default, Clone)]
pub struct NetBalance {
    pub in_per_min: f64,
    pub out_per_min: f64,
}

impl NetBalance {
    pub fn net(&self) -> f64 {
        self.out_per_min - self.in_per_min
    }
}

pub fn compute_balance(
    facilities: &[Facility],
    recipes_by_id: &HashMap<String, Recipe>,
) -> HashMap<String, NetBalance> {
    let mut totals: HashMap<String, NetBalance> = HashMap::new();
    for f in facilities {
        if f.count == 0 {
            continue;
        }
        let Some(recipe) = recipes_by_id.get(&f.recipe_id) else {
            continue;
        };
        let mul = (60.0 / recipe.duration as f64) * f.count as f64;
        for input in &recipe.inputs {
            totals.entry(input.resource.clone()).or_default().in_per_min += input.qty * mul;
        }
        for output in &recipe.outputs {
            totals
                .entry(output.resource.clone())
                .or_default()
                .out_per_min += output.qty * mul;
        }
    }
    totals
}

pub fn fmt_qty(n: f64) -> String {
    let abs = n.abs();
    if abs == 0.0 {
        return "0".into();
    }
    if abs >= 1000.0 {
        let s = format!("{:.2}", n / 1000.0);
        return s.trim_end_matches('0').trim_end_matches('.').to_string() + "k";
    }
    if abs >= 100.0 {
        return format!("{:.0}", n);
    }
    if abs >= 10.0 {
        let s = format!("{:.1}", n);
        return s.trim_end_matches('0').trim_end_matches('.').to_string();
    }
    let s = format!("{:.2}", n);
    s.trim_end_matches('0').trim_end_matches('.').to_string()
}
