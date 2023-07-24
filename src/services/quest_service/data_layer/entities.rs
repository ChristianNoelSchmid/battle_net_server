use serde::Deserialize;

#[derive(Deserialize)]
pub enum QuestType {
    Monster,
    Riddle,
}