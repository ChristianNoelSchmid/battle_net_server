use std::{collections::HashMap, sync::Arc};

use lazy_static::lazy_static;

use crate::{services::{game_service::models::Stats, battle_service::data_layer::{ATTACK_IDX, DEFEND_IDX, IDLE_IDX}}, dice};

pub trait Ai : Send + Sync {
    fn next_act(&self, pl_stats: &Stats, monst_stats: &Stats) -> i64;
}

struct WilfredAi;
impl Ai for WilfredAi {
    fn next_act(&self, _pl_stats: &Stats, monst_stats: &Stats) -> i64 {
        if monst_stats.power < 2 {
            return if dice::single(2) == 1 { DEFEND_IDX } else { IDLE_IDX };
        }
        return if dice::single(3) == 1 { ATTACK_IDX } else { IDLE_IDX };
    }
}

struct NutcrackerAi;
impl Ai for NutcrackerAi {
    fn next_act(&self, _pl_stats: &Stats, _monst_stats: &Stats) -> i64 {
        match dice::single(3) {
            1 => IDLE_IDX,
            2 => ATTACK_IDX,
            _ => DEFEND_IDX 
        }
    }
}

lazy_static! {
    pub static ref AI: HashMap<&'static str, Arc<dyn Ai>> = [
        ("Wilfred, the Esteemed Wizard", Arc::new(WilfredAi) as Arc<dyn Ai>),
        ("Possesed Nutcracker", Arc::new(NutcrackerAi) as Arc<dyn Ai>)
    ].iter().cloned().collect();
}