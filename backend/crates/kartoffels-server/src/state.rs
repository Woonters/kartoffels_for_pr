use crate::error::{AppError, AppResult};
use kartoffels::prelude::{Handle, WorldId, WorldName};
use std::collections::HashMap;
use std::path::PathBuf;

#[derive(Debug)]
pub struct AppState {
    pub data: Option<PathBuf>,
    pub worlds: HashMap<WorldId, Handle>,
    pub secret: Option<String>,
}

impl AppState {
    pub fn world(&self, id: WorldId) -> AppResult<Handle> {
        self.worlds.get(&id).ok_or(AppError::WorldNotFound).cloned()
    }

    pub fn has_world_named(&self, name: &WorldName) -> bool {
        self.worlds.values().any(|world| world.name() == name)
    }
}
