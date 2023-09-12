use crate::game_executable::GameExecutable;
use serde::Serialize;
use specta::Type;
use std::collections::HashMap;

pub type GameMap = HashMap<u32, Game>;

#[derive(Serialize, Type, Clone)]
#[serde(rename_all = "camelCase")]
pub struct Game {
    pub id: u32,
    pub name: String,
    pub executables: HashMap<String, GameExecutable>,
}
