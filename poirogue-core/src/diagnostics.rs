use std::any::type_name;
use shipyard::{Storage, View, World};
use serde::{Serialize, Deserialize};
use crate::core_systems::IsCharacter;
use crate::entity::HasPosition;
use crate::game::{MemoryUsageLog, Store};
use crate::game_systems::{BumpIntent, CollectIntent, Handle, InvestigateIntent, IsItem, IsLocked, MoveDirective, UnlockDirective, UnlockIntent};

pub fn create_memory_usage_log(usage_log: &mut MemoryUsageLog) {
    fn create_memory_usage_log_for<T>(usage_log: &mut MemoryUsageLog) {
        let name = String::from(type_name::<T>());
        let name = name
            .replace("poirogue_core::game_systems::intents::Handle", "Handle")
            .replace("poirogue_core::game_systems::", "")
            .replace("poirogue_core::core_systems::", "");

        usage_log.0.lcreate(format!("{}", name).as_str()).unwrap();
    }

    create_memory_usage_log_for::<Handle<BumpIntent>>(usage_log);
    create_memory_usage_log_for::<Handle<UnlockIntent>>(usage_log);
    create_memory_usage_log_for::<Handle<CollectIntent>>(usage_log);
    create_memory_usage_log_for::<Handle<InvestigateIntent>>(usage_log);
    create_memory_usage_log_for::<MoveDirective>(usage_log);
    create_memory_usage_log_for::<UnlockDirective>(usage_log);
    create_memory_usage_log_for::<IsItem>(usage_log);
    create_memory_usage_log_for::<IsLocked>(usage_log);
    create_memory_usage_log_for::<HasPosition>(usage_log);
    create_memory_usage_log_for::<IsCharacter>(usage_log);
}

#[derive(Serialize, Deserialize)]
struct MemoryUsage {
    frame: u64,
    allocated_bytes: usize,
    memory_used_bytes: usize,
    component_count: usize,
}

pub fn log_overall_memory_usage(frame: u64, world: &World, usage_log: &mut MemoryUsageLog) {
    fn log_memory_usage<T: Send + Sync + 'static>(frame: u64, world: &World, usage_log: &mut MemoryUsageLog) {
        if let Ok(view) = world.borrow::<View<T>>() {
            let usage = view.memory_usage().unwrap();
            let name = String::from(type_name::<T>());
            let name = name
                .replace("poirogue_core::game_systems::intents::Handle", "Handle")
                .replace("poirogue_core::game_systems::", "")
                .replace("poirogue_core::core_systems::", "");

            let mem = MemoryUsage {
                frame,
                allocated_bytes: usage.allocated_memory_bytes,
                memory_used_bytes: usage.used_memory_bytes,
                component_count: usage.component_count,
            };

            usage_log.0.ladd(format!("{}", name.as_str()).as_str(),  &mem);
        }
    }

    log_memory_usage::<Handle<BumpIntent>>(frame, world, usage_log);
    log_memory_usage::<Handle<UnlockIntent>>(frame, world, usage_log);
    log_memory_usage::<Handle<CollectIntent>>(frame, world, usage_log);
    log_memory_usage::<Handle<InvestigateIntent>>(frame, world, usage_log);
    log_memory_usage::<MoveDirective>(frame, world, usage_log);
    log_memory_usage::<UnlockDirective>(frame, world, usage_log);
    log_memory_usage::<IsItem>(frame, world, usage_log);
    log_memory_usage::<IsLocked>(frame, world, usage_log);
    log_memory_usage::<HasPosition>(frame, world, usage_log);
    log_memory_usage::<IsCharacter>(frame, world, usage_log);

    usage_log.0.dump().unwrap();
}
