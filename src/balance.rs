use crate::data::Recipe;
use crate::state::Facility;
use std::collections::HashMap;

#[derive(Debug, Default, Clone)]
pub struct NetBalance {
    pub in_per_min: f64,
    pub out_per_min: f64,
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::data::{Recipe, RecipeIO};
    use crate::state::Facility;

    fn make_recipe(
        id: &str,
        duration: u32,
        inputs: Vec<(&str, f64)>,
        outputs: Vec<(&str, f64)>,
    ) -> Recipe {
        Recipe {
            id: id.into(),
            building: "b".into(),
            name: id.into(),
            category: "cat".into(),
            duration,
            inputs: inputs
                .into_iter()
                .map(|(r, q)| RecipeIO {
                    resource: r.into(),
                    qty: q,
                })
                .collect(),
            outputs: outputs
                .into_iter()
                .map(|(r, q)| RecipeIO {
                    resource: r.into(),
                    qty: q,
                })
                .collect(),
        }
    }

    fn make_facility(recipe_id: &str, count: u32) -> Facility {
        Facility {
            uid: "u".into(),
            recipe_id: recipe_id.into(),
            count,
            x: None,
            y: None,
        }
    }

    // --- fmt_qty ---

    #[test]
    fn fmt_qty_zero() {
        assert_eq!(fmt_qty(0.0), "0");
    }

    #[test]
    fn fmt_qty_small_whole() {
        assert_eq!(fmt_qty(5.0), "5");
    }

    #[test]
    fn fmt_qty_small_decimal() {
        assert_eq!(fmt_qty(5.5), "5.5");
        assert_eq!(fmt_qty(5.55), "5.55");
    }

    #[test]
    fn fmt_qty_tens_whole() {
        assert_eq!(fmt_qty(15.0), "15");
    }

    #[test]
    fn fmt_qty_tens_decimal() {
        assert_eq!(fmt_qty(15.5), "15.5");
    }

    #[test]
    fn fmt_qty_hundreds() {
        assert_eq!(fmt_qty(150.0), "150");
        assert_eq!(fmt_qty(999.0), "999");
    }

    #[test]
    fn fmt_qty_thousands_whole() {
        assert_eq!(fmt_qty(1000.0), "1k");
        assert_eq!(fmt_qty(2000.0), "2k");
    }

    #[test]
    fn fmt_qty_thousands_decimal() {
        assert_eq!(fmt_qty(1500.0), "1.5k");
        assert_eq!(fmt_qty(1234.0), "1.23k");
    }

    #[test]
    fn fmt_qty_negative() {
        assert_eq!(fmt_qty(-5.5), "-5.5");
        assert_eq!(fmt_qty(-1500.0), "-1.5k");
    }

    // --- compute_balance ---

    #[test]
    fn balance_empty_facilities() {
        let result = compute_balance(&[], &HashMap::new());
        assert!(result.is_empty());
    }

    #[test]
    fn balance_skips_zero_count() {
        let mut recipes = HashMap::new();
        recipes.insert(
            "r1".into(),
            make_recipe("r1", 60, vec![("iron", 1.0)], vec![]),
        );
        let result = compute_balance(&[make_facility("r1", 0)], &recipes);
        assert!(result.is_empty());
    }

    #[test]
    fn balance_skips_unknown_recipe() {
        let result = compute_balance(&[make_facility("unknown", 1)], &HashMap::new());
        assert!(result.is_empty());
    }

    #[test]
    fn balance_single_facility_60s_recipe() {
        // duration=60, count=1 → mul=1.0
        let mut recipes = HashMap::new();
        recipes.insert(
            "r1".into(),
            make_recipe("r1", 60, vec![("iron", 2.0)], vec![("steel", 1.0)]),
        );
        let result = compute_balance(&[make_facility("r1", 1)], &recipes);
        assert_eq!(result["iron"].in_per_min, 2.0);
        assert_eq!(result["steel"].out_per_min, 1.0);
    }

    #[test]
    fn balance_faster_recipe_multiplies_throughput() {
        // duration=30, count=1 → mul=2.0
        let mut recipes = HashMap::new();
        recipes.insert(
            "r1".into(),
            make_recipe("r1", 30, vec![("coal", 3.0)], vec![("coke", 1.0)]),
        );
        let result = compute_balance(&[make_facility("r1", 1)], &recipes);
        assert_eq!(result["coal"].in_per_min, 6.0);
        assert_eq!(result["coke"].out_per_min, 2.0);
    }

    #[test]
    fn balance_count_multiplies_throughput() {
        // duration=60, count=4 → mul=4.0
        let mut recipes = HashMap::new();
        recipes.insert(
            "r1".into(),
            make_recipe("r1", 60, vec![("ore", 1.0)], vec![("metal", 2.0)]),
        );
        let result = compute_balance(&[make_facility("r1", 4)], &recipes);
        assert_eq!(result["ore"].in_per_min, 4.0);
        assert_eq!(result["metal"].out_per_min, 8.0);
    }

    #[test]
    fn balance_accumulates_across_facilities() {
        let mut recipes = HashMap::new();
        recipes.insert(
            "r1".into(),
            make_recipe("r1", 60, vec![("iron", 1.0)], vec![]),
        );
        recipes.insert(
            "r2".into(),
            make_recipe("r2", 60, vec![("iron", 3.0)], vec![]),
        );
        let facilities = [make_facility("r1", 1), make_facility("r2", 1)];
        let result = compute_balance(&facilities, &recipes);
        assert_eq!(result["iron"].in_per_min, 4.0);
    }

    #[test]
    fn balance_resource_can_be_both_input_and_output() {
        // Recipe consumes water and outputs steam; steam is also consumed by another recipe
        let mut recipes = HashMap::new();
        recipes.insert(
            "r1".into(),
            make_recipe("r1", 60, vec![("water", 1.0)], vec![("steam", 1.0)]),
        );
        recipes.insert(
            "r2".into(),
            make_recipe("r2", 60, vec![("steam", 0.5)], vec![]),
        );
        let facilities = [make_facility("r1", 1), make_facility("r2", 1)];
        let result = compute_balance(&facilities, &recipes);
        assert_eq!(result["steam"].out_per_min, 1.0);
        assert_eq!(result["steam"].in_per_min, 0.5);
    }
}
