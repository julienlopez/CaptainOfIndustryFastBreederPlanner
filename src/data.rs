use serde::Deserialize;
use std::collections::HashMap;

#[derive(Deserialize, Clone, Debug)]
pub struct Resource {
    pub id: String,
    pub name: String,
    pub short: String,
    pub color: String,
    pub group: String,
    pub icon: Vec<String>,
}

#[derive(Deserialize, Clone, Debug)]
pub struct Machine {
    pub name: String,
    pub tier: u8,
    pub category: String,
    pub icon: Vec<String>,
}

#[derive(Deserialize, Clone, Debug)]
pub struct RecipeIO {
    pub resource: String,
    pub qty: f64,
}

#[derive(Deserialize, Clone, Debug)]
pub struct Recipe {
    pub id: String,
    pub building: String,
    pub name: String,
    pub category: String,
    pub duration: u32,
    pub inputs: Vec<RecipeIO>,
    pub outputs: Vec<RecipeIO>,
}

#[derive(Deserialize, Clone, Debug)]
pub struct Category {
    pub id: String,
    pub name: String,
    pub color: String,
}

pub struct GameData {
    pub resources: Vec<Resource>,
    pub resources_by_id: HashMap<String, Resource>,
    pub machines_by_name: HashMap<String, Machine>,
    pub recipes: Vec<Recipe>,
    pub recipes_by_id: HashMap<String, Recipe>,
    pub categories: Vec<Category>,
    pub categories_by_id: HashMap<String, Category>,
    pub recipes_by_category: HashMap<String, Vec<Recipe>>,
}

impl GameData {
    pub fn load() -> Self {
        let resources: Vec<Resource> = serde_json::from_str(include_str!("../data/resources.json"))
            .expect("parse resources.json");

        let machines: Vec<Machine> = serde_json::from_str(include_str!("../data/machines.json"))
            .expect("parse machines.json");

        let recipes: Vec<Recipe> =
            serde_json::from_str(include_str!("../data/recipes.json")).expect("parse recipes.json");

        let categories: Vec<Category> =
            serde_json::from_str(include_str!("../data/categories.json"))
                .expect("parse categories.json");

        let resources_by_id: HashMap<String, Resource> = resources
            .iter()
            .map(|r| (r.id.clone(), r.clone()))
            .collect();

        let machines_by_name: HashMap<String, Machine> = machines
            .iter()
            .map(|m| (m.name.clone(), m.clone()))
            .collect();

        let recipes_by_id: HashMap<String, Recipe> =
            recipes.iter().map(|r| (r.id.clone(), r.clone())).collect();

        let categories_by_id: HashMap<String, Category> = categories
            .iter()
            .map(|c| (c.id.clone(), c.clone()))
            .collect();

        let mut recipes_by_category: HashMap<String, Vec<Recipe>> = HashMap::new();
        for cat in &categories {
            recipes_by_category.insert(cat.id.clone(), Vec::new());
        }
        for recipe in &recipes {
            recipes_by_category
                .entry(recipe.category.clone())
                .or_default()
                .push(recipe.clone());
        }

        Self {
            resources,
            resources_by_id,
            machines_by_name,
            recipes,
            recipes_by_id,
            categories,
            categories_by_id,
            recipes_by_category,
        }
    }
}
