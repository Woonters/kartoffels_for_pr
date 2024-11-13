mod systems;

pub use self::systems::*;
use crate::{BotEvent, BotId, Dir, Map};
use ahash::AHashMap;
use glam::IVec2;
use itertools::Itertools;
use prettytable::{row, Table};
use std::cmp::Reverse;
use std::collections::VecDeque;
use std::fmt;
use std::sync::Arc;

#[derive(Debug, Default, PartialEq, Eq)]
pub struct Snapshot {
    raw_map: Map,
    map: Map,
    bots: SnapshotBots,
    version: u64,
}

impl Snapshot {
    pub fn raw_map(&self) -> &Map {
        &self.raw_map
    }

    pub fn map(&self) -> &Map {
        &self.map
    }

    pub fn bots(&self) -> &SnapshotBots {
        &self.bots
    }

    pub fn version(&self) -> u64 {
        self.version
    }
}

impl fmt::Display for Snapshot {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        writeln!(f, "# map")?;
        writeln!(f)?;
        writeln!(f, "```")?;
        writeln!(f, "{}", self.map)?;
        writeln!(f, "```")?;

        if !self.bots.is_empty() {
            writeln!(f)?;
            writeln!(f, "# bots")?;
            writeln!(f)?;
            writeln!(f, "{}", self.bots)?;
        }

        Ok(())
    }
}

#[derive(Debug, Default, PartialEq, Eq)]
pub struct SnapshotBots {
    alive: SnapshotAliveBots,
    dead: SnapshotDeadBots,
    queued: SnapshotQueuedBots,
}

impl SnapshotBots {
    pub fn alive(&self) -> &SnapshotAliveBots {
        &self.alive
    }

    pub fn dead(&self) -> &SnapshotDeadBots {
        &self.dead
    }

    pub fn queued(&self) -> &SnapshotQueuedBots {
        &self.queued
    }

    pub fn has(&self, id: BotId) -> bool {
        self.alive.get(id).is_some()
            || self.dead.get(id).is_some()
            || self.queued.get(id).is_some()
    }

    pub fn is_empty(&self) -> bool {
        self.alive.is_empty() && self.dead.is_empty() && self.queued.is_empty()
    }
}

impl fmt::Display for SnapshotBots {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        if !self.alive.is_empty() {
            writeln!(f, "## alive")?;
            writeln!(f)?;
            writeln!(f, "{}", self.alive)?;
        }

        Ok(())
    }
}

#[derive(Debug, Default, PartialEq, Eq)]
pub struct SnapshotAliveBots {
    entries: Vec<SnapshotAliveBot>,
    id_to_idx: AHashMap<BotId, u8>,
    idx_by_scores: Vec<u8>,
}

impl SnapshotAliveBots {
    pub fn get(&self, id: BotId) -> Option<&SnapshotAliveBot> {
        self.get_by_idx(*self.id_to_idx.get(&id)?)
    }

    pub fn get_by_idx(&self, idx: u8) -> Option<&SnapshotAliveBot> {
        self.entries.get(idx as usize)
    }

    pub fn has(&self, id: BotId) -> bool {
        self.get(id).is_some()
    }

    pub fn has_all_of(&self, ids: &[BotId]) -> bool {
        ids.iter().all(|id| self.has(*id))
    }

    pub fn has_any_of(&self, ids: &[BotId]) -> bool {
        ids.iter().any(|id| self.has(*id))
    }

    pub fn iter(&self) -> impl Iterator<Item = &SnapshotAliveBot> {
        self.entries.iter()
    }

    pub fn iter_sorted_by_birth(
        &self,
    ) -> impl Iterator<Item = &SnapshotAliveBot> {
        self.entries
            .iter()
            .sorted_unstable_by_key(|bot| (Reverse(bot.age), bot.id))
    }

    pub fn iter_sorted_by_scores(
        &self,
    ) -> impl Iterator<Item = &SnapshotAliveBot> + '_ {
        self.idx_by_scores
            .iter()
            .filter_map(|idx| self.get_by_idx(*idx))
    }

    pub fn len(&self) -> usize {
        self.entries.len()
    }

    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }
}

impl fmt::Display for SnapshotAliveBots {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let mut table = Table::init(vec![]);

        table.set_titles(row!["id", "pos", "dir", "age", "score"]);

        for bot in self.iter_sorted_by_scores() {
            table.add_row(row![bot.id, bot.pos, bot.dir, bot.age, bot.score]);
        }

        write!(f, "{table}")
    }
}

#[derive(Debug, PartialEq, Eq)]
pub struct SnapshotAliveBot {
    pub age: u32,
    pub dir: Dir,
    pub events: Arc<VecDeque<Arc<BotEvent>>>,
    pub id: BotId,
    pub pos: IVec2,
    pub score: u32,
    pub serial: Arc<VecDeque<u32>>,
}

#[derive(Debug, Default, PartialEq, Eq)]
pub struct SnapshotDeadBots {
    entries: AHashMap<BotId, SnapshotDeadBot>,
}

impl SnapshotDeadBots {
    pub fn get(&self, id: BotId) -> Option<&SnapshotDeadBot> {
        self.entries.get(&id)
    }

    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }
}

#[derive(Debug, PartialEq, Eq)]
pub struct SnapshotDeadBot {
    pub events: Arc<VecDeque<Arc<BotEvent>>>,
    pub serial: Arc<VecDeque<u32>>,
}

#[derive(Debug, Default, PartialEq, Eq)]
pub struct SnapshotQueuedBots {
    entries: AHashMap<BotId, SnapshotQueuedBot>,
}

impl SnapshotQueuedBots {
    pub fn get(&self, id: BotId) -> Option<&SnapshotQueuedBot> {
        self.entries.get(&id)
    }

    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }
}

#[derive(Debug, PartialEq, Eq)]
pub struct SnapshotQueuedBot {
    pub events: Arc<VecDeque<Arc<BotEvent>>>,
    pub place: u8,
    pub requeued: bool,
    pub serial: Arc<VecDeque<u32>>,
}
