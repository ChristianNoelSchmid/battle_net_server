use serde::{Deserialize, Serialize};

use crate::execute;

pub trait Effect {
    fn get_action(&self) -> String;
    fn get_removal_action(&self) -> String;

    fn apply_to_monster(&self, db: &Connection, quest_id: i64, monster_id: i64) {
        let update = format!(
            r"UPDATE stats JOIN monster_state ON id = stats_id SET {} 
              WHERE quest_id = ? AND monster_id = ?",
            self.get_action()
        );
        execute!(
            db,
            update,
            Value::Integer(quest_id),
            Value::Integer(monster_id)
        );
    }
    fn apply_to_user(&self, db: &Connection, user_id: i64) {
        let update = format!(
            r"UPDATE stats JOIN users ON stats.id = users.stats_id 
              SET {} WHERE id = ?",
            self.get_action()
        );
        execute!(db, update, Value::Integer(user_id));
    }
    fn remove_from_user(&self, db: &Connection, user_id: i64) {
        let update = format!(
            r"UPDATE stats JOIN users ON stats.id = users.stats_id
              SET {} WHERE id = ?",
            self.get_removal_action()
        );
        execute!(db, update, Value::Integer(user_id));
    }
}

pub struct DamageHealthEffect(i64);
impl Effect for DamageHealthEffect {
    fn get_action(&self) -> String {
        format!("health = health - {}", self.0)
    }
    fn get_removal_action(&self) -> String {
        format!("health = health + {}", self.0)
    }
}

pub struct BoostHealthEffect(i64);
impl Effect for BoostHealthEffect {
    fn get_action(&self) -> String {
        format!("health = health + {}", self.0)
    }
    fn get_removal_action(&self) -> String {
        format!("health = health - {}", self.0)
    }
}

pub struct DamageMagickaEffect(i64);
impl Effect for DamageMagickaEffect {
    fn get_action(&self) -> String {
        format!("magicka = magicka - {}", self.0)
    }
    fn get_removal_action(&self) -> String {
        format!("magicka = magicka + {}", self.0)
    }
}

pub struct BoostMagickaEffect(i64);
impl Effect for BoostMagickaEffect {
    fn get_action(&self) -> String {
        format!("magicka = magicka + {}", self.0)
    }
    fn get_removal_action(&self) -> String {
        format!("magicka = magicka - {}", self.0)
    }
}

pub struct BoostReflexEffect(i64);
impl Effect for BoostReflexEffect {
    fn get_action(&self) -> String {
        format!("SET reflex = reflex + {}", self.0)
    }
    fn get_removal_action(&self) -> String {
        format!("SET reflex = reflex - {}", self.0)
    }
}

pub struct BoostWisdomEffect(i64);
impl Effect for BoostWisdomEffect {
    fn get_action(&self) -> String {
        format!("SET wisdom = wisdom + {}", self.0)
    }
    fn get_removal_action(&self) -> String {
        format!("SET wisdom = wisdom - {}", self.0)
    }
}

pub struct BoostArmorEffect(i64);
impl Effect for BoostArmorEffect {
    fn get_action(&self) -> String {
        format!("SET armor = armor + {}", self.0)
    }
    fn get_removal_action(&self) -> String {
        format!("SET armor = armor - {}", self.0)
    }
}

#[derive(Serialize, Deserialize)]
pub enum EffectType {
    #[serde(rename = "damage_health")]
    DamageHealth(i64),
    #[serde(rename = "boost_health")]
    BoostHealth(i64),
    #[serde(rename = "damage_magic")]
    DamageMagicka(i64),
    #[serde(rename = "boost_magic")]
    BoostMagicka(i64),
    #[serde(rename = "boost_reflex")]
    BoostReflex(i64),
    #[serde(rename = "boost_wisdom")]
    BoostWisdom(i64),
    #[serde(rename = "boost_armor")]
    BoostArmor(i64),
}

impl EffectType {
    pub fn to_effect(&self) -> Box<dyn Effect> {
        return match self {
            EffectType::BoostHealth(i) => Box::new(BoostHealthEffect(*i)),
            EffectType::DamageHealth(i) => Box::new(DamageHealthEffect(*i)),
            EffectType::DamageMagicka(i) => Box::new(DamageMagickaEffect(*i)),
            EffectType::BoostMagicka(i) => Box::new(BoostMagickaEffect(*i)),
            EffectType::BoostReflex(i) => Box::new(BoostReflexEffect(*i)),
            EffectType::BoostWisdom(i) => Box::new(BoostWisdomEffect(*i)),
            EffectType::BoostArmor(i) => Box::new(BoostArmorEffect(*i)),
        };
    }
}
