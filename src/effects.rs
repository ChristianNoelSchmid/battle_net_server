use sqlite::{Connection, Value};

use crate::execute;

pub trait Effect {
    fn get_action(&self) -> String;
    fn apply_to_monster(&self, db: &Connection, quest_id: i64, monster_id: i64) {
        let update = format!(
            r"UPDATE stats JOIN quest_monsters ON id = stats_id SET {} 
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
}

pub struct DamageEffect(i64);
impl Effect for DamageEffect {
    fn get_action(&self) -> String {
        format!("cur_health = cur_health - {}", self.0)
    }
}

pub struct HealthEffect(i64);
impl Effect for HealthEffect {
    fn get_action(&self) -> String {
        format!("cur_health = cur_health + {}", self.0)
    }
}

pub struct FleeEffect;
impl Effect for FleeEffect {
    fn get_action(&self) -> String {
        format!(
            r"SET cur_flee_from_chances = 
            cur_flee_from_chances + (100 - (cur_flee_from_chances / 2))"
        )
    }
}
