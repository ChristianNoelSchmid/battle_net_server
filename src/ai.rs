use std::{collections::HashMap, sync::Arc};

use lazy_static::lazy_static;

use crate::services::{game_service::models::Stats, battle_service::models::BattleAction};

pub trait Ai : Send + Sync {
    fn next_act(&self, pl_stats: &Stats, monst_stats: &Stats) -> BattleAction;
}

struct WilfredAi;
impl Ai for WilfredAi {
    fn next_act(&self, _pl_stats: &Stats, _monst_stats: &Stats) -> BattleAction {
        BattleAction::Idle
    }
}

struct NutcrackerAi;
impl Ai for NutcrackerAi {
    fn next_act(&self, _pl_stats: &Stats, _monst_stats: &Stats) -> BattleAction {
        BattleAction::Idle
    }
}

lazy_static! {
    pub static ref AI: HashMap<&'static str, Arc<dyn Ai>> = [
        ("Wilfred, the Esteemed Wizard", Arc::new(WilfredAi) as Arc<dyn Ai>),
        ("Possessed Nutcracker", Arc::new(NutcrackerAi) as Arc<dyn Ai>)
    ].iter().cloned().collect();
}