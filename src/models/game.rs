use crate::resources::game_resources::BaseStats;
use serde::Serialize;

#[derive(Serialize, Clone, Copy)]
pub struct Stats {
    pub health: i64,
    pub magicka: i64,
    pub armor: i64,
    pub wisdom: i64,
    pub reflex: i64,
    pub miss_turn: bool,
}

impl Stats {
    pub fn from_base_stats(b_stats: BaseStats) -> Self {
        Self {
            health: b_stats.health,
            magicka: b_stats.magicka,
            armor: b_stats.armor,
            wisdom: b_stats.wisdom,
            reflex: b_stats.reflex,
            miss_turn: false,
        }
    }
}