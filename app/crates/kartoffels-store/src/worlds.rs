use ahash::AHashMap;
use anyhow::{anyhow, Context, Result};
use arc_swap::ArcSwap;
use kartoffels_utils::Id;
use kartoffels_world::prelude::{Config as WorldConfig, Handle as WorldHandle};
use std::collections::hash_map;
use std::path::Path;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{Arc, Mutex};
use tokio::fs;
use tracing::info;

#[derive(Debug)]
pub struct Worlds {
    public: ArcSwap<Vec<WorldHandle>>,
    private: Arc<Mutex<AHashMap<Id, WorldHandle>>>,
    testing_id: AtomicU64,
}

impl Worlds {
    pub const MAX_PRIVATE_WORLDS: usize = 128;

    pub async fn new(dir: Option<&Path>) -> Result<Self> {
        let public = if let Some(dir) = dir {
            Self::load(dir).await?
        } else {
            Default::default()
        };

        Ok(Self {
            public: ArcSwap::new(Arc::new(public)),
            private: Default::default(),
            testing_id: AtomicU64::new(1),
        })
    }

    async fn load(dir: &Path) -> Result<Vec<WorldHandle>> {
        let mut worlds = Vec::new();
        let mut entries = fs::read_dir(dir).await?;

        while let Some(entry) = entries.next_entry().await? {
            let path = entry.path();

            let Some(entry_stem) =
                path.file_stem().and_then(|stem| stem.to_str())
            else {
                continue;
            };

            let Some("world") = path.extension().and_then(|ext| ext.to_str())
            else {
                continue;
            };

            info!("loading: {}", path.display());

            let result: Result<()> = try {
                let id = entry_stem
                    .parse()
                    .context("couldn't extract world id from path")?;

                let world = kartoffels_world::resume(id, &path)?;

                worlds.push(world);
            };

            result.with_context(|| {
                format!("couldn't load world: {}", path.display())
            })?;
        }

        worlds.sort_by_key(|world| world.name().to_owned());

        Ok(worlds)
    }

    pub fn set(&self, public: impl IntoIterator<Item = WorldHandle>) {
        self.public.swap(Arc::new(public.into_iter().collect()));
    }

    pub fn create_public(&self, config: WorldConfig) -> Result<WorldHandle> {
        assert!(config.id.is_none());
        assert!(config.path.is_none());

        todo!();
    }

    pub fn public(&self) -> Arc<Vec<WorldHandle>> {
        self.public.load_full()
    }

    pub fn create_private(
        &self,
        testing: bool,
        config: WorldConfig,
    ) -> Result<WorldHandle> {
        assert!(config.id.is_none());
        assert!(config.path.is_none());

        let mut private = self.private.lock().unwrap();

        if private.len() >= Self::MAX_PRIVATE_WORLDS {
            return Err(anyhow!("ouch, the server is currently overloaded"));
        }

        let (id, handle) = loop {
            let id = if testing {
                Id::new(self.testing_id.fetch_add(1, Ordering::Relaxed))
            } else {
                rand::random()
            };

            if let hash_map::Entry::Vacant(entry) = private.entry(id) {
                info!(?id, "private world created");

                let handle = kartoffels_world::create(WorldConfig {
                    id: Some(id),
                    ..config
                });

                entry.insert(handle.clone());

                break (id, handle);
            }
        };

        let handle = handle.on_last_drop({
            let private = self.private.clone();

            move || {
                info!(?id, "private world destroyed");
                private.lock().unwrap().remove(&id);
            }
        });

        Ok(handle)
    }

    pub fn first_private(&self) -> Option<WorldHandle> {
        self.private.lock().unwrap().values().next().cloned()
    }

    pub async fn shutdown(&self) -> Result<()> {
        for world in self.public.load().iter() {
            world.shutdown().await?;
        }

        Ok(())
    }
}
