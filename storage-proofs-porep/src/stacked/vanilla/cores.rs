use filecoin_core_affinity::*;
use lazy_static::lazy_static;
use anyhow::Result;
use serde::{Serialize, Deserialize};


#[derive(Serialize, Deserialize, Debug)]
pub enum Level {
    L3,
    L2,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct CoreAllocatorSettings {
    #[serde(default)]
    pub cpus: Option<Vec<usize>>,
    pub level: Level,
    pub excluded: Option<Vec<u32>>,
}

impl CoreAllocatorSettings {
    pub fn load() -> Result<Self> {
        let content = std::fs::read_to_string(std::env::var("CORE_ALLOCATOR_SETTINGS_FILE").unwrap_or("core_allocator.yaml".to_owned()))?;
        let this = serde_yaml::from_str(&content)?;
        Ok(this)
    }
    pub fn get_allocator(&self) -> Result<Box<dyn CoreAllocator>> {
        let depth = match self.level {
            Level::L3 => { hwloc2::ObjectType::L3Cache }
            Level::L2 => { hwloc2::ObjectType::L2Cache }
        };
        let mut allocator = HierarchicalAllocator::new_at_depth(depth);
        if let Some(cpus) = self.cpus.clone() {
            allocator = allocator.on_cpu(cpus);
        }

        let mut excluded: Vec<u32> = Vec::new();
        if let Some(cfg) = self.excluded.clone() {
            excluded = cfg;
        }.expect("Please configure cores to filter");
        let finished = allocator.finish(excluded);
        println!("CoreGroups {:?}", finished);
        Ok(Box::new(finished))
    }
}

pub fn checkout_core_group() -> Option<CoreGroup> {
    lazy_static! {
            static ref ALLOCATOR: Box<dyn CoreAllocator> = {
                CoreAllocatorSettings::load().unwrap().get_allocator().unwrap()
            };
        }
    Some(ALLOCATOR.allocate_core().unwrap())
}