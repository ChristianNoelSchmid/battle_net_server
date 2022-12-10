use serde::{Deserialize, Serialize};

#[derive(Default)]
pub struct ResourceLoader {
    pub evd_card_cats: Vec<EvidenceCardCategories>,
    pub avatars: Vec<Avatar>,
    pub monsters: Vec<Monster>,
    pub riddles: Vec<Riddle>,
}

impl ResourceLoader {
    pub fn load(folder_path: String) -> Self {
        Self::default()
    }
}

pub struct Resources<'a> {
    pub evd_card_cats: &'a Vec<EvidenceCardCategories>,
    pub avatars: &'a Vec<Avatar>,
    pub monsters: &'a Vec<Monster>,
    pub riddles: &'a Vec<Riddle>,
}

impl<'a> Resources<'a> {
    pub fn from_loader(res_loader: &'a ResourceLoader) -> Self {
        Self {
            evd_card_cats: &res_loader.evd_card_cats,
            avatars: &res_loader.avatars,
            monsters: &res_loader.monsters,
            riddles: &res_loader.riddles,
        }
    }
}

#[derive(Serialize, Clone)]
pub struct Avatar {
    pub card_idx: i64,
    pub user_name: String,
    pub user_img_path: Option<String>,
}

#[derive(Serialize, Deserialize)]
pub struct EvidenceCardCategories {
    pub name: String,
    pub tag: String,
    pub cards: Vec<EvidenceCard>,
}

#[derive(Serialize, Deserialize)]
pub struct EvidenceCard {
    pub name: String,
    pub img_path: Option<String>,
}

#[derive(Deserialize, Clone)]
pub struct SabotageCard {
    pub name: String,
    pub effect_tags: Vec<String>,
}

#[derive(Serialize, Deserialize)]
pub struct Monster {
    tag: String,
    name: String,
    base_health: i32,
    base_damage: i32,
    base_magicka: i32,
    item_tags: Vec<String>,
    spell_tags: Vec<String>,
    spell_count: i32,
}

#[derive(Serialize, Clone)]
pub struct Riddle {
    pub text: String,
    pub answers: Vec<String>,
}

pub struct Spell {
    tag: String,
    name: String,
    flavor_text: String,
    magicka_cost: i32,
}
