use cmos_memory::l1::{WorkingMemory, WorkingMemoryConfig, SlotPriority};

#[test]
fn insert_and_retrieve() {
    let wm = WorkingMemory::new(WorkingMemoryConfig::default());
    wm.insert("ctx-1", "Hello world", SlotPriority::Context);

    let slot = wm.get("ctx-1").unwrap();
    assert_eq!(slot.content, "Hello world");
    assert_eq!(slot.access_count, 1);
}

#[test]
fn insert_replaces_existing() {
    let wm = WorkingMemory::new(WorkingMemoryConfig::default());
    wm.insert("ctx-1", "version 1", SlotPriority::Context);
    wm.insert("ctx-1", "version 2", SlotPriority::Context);

    let slot = wm.get("ctx-1").unwrap();
    assert_eq!(slot.content, "version 2");
    assert_eq!(wm.slot_count(), 1);
}

#[test]
fn remove_slot() {
    let wm = WorkingMemory::new(WorkingMemoryConfig::default());
    wm.insert("ctx-1", "data", SlotPriority::Context);
    let removed = wm.remove("ctx-1");
    assert!(removed.is_some());
    assert!(wm.get("ctx-1").is_none());
    assert_eq!(wm.total_tokens(), 0);
}

#[test]
fn eviction_respects_priority() {
    let config = WorkingMemoryConfig {
        max_tokens: 20,
        eviction_headroom: 5,
    };
    let wm = WorkingMemory::new(config);

    wm.insert("system", "sys", SlotPriority::System);
    wm.insert("scratch", "scr", SlotPriority::Scratch);

    // Insert something that forces eviction
    let evicted = wm.insert("big", "a]".repeat(10).as_str(), SlotPriority::Context);

    // Scratch should be evicted before system
    if let Some(evicted_slots) = evicted {
        assert!(evicted_slots.iter().any(|s| s.id == "scratch"));
        assert!(!evicted_slots.iter().any(|s| s.id == "system"));
    }
}

#[test]
fn assemble_orders_by_priority() {
    let wm = WorkingMemory::new(WorkingMemoryConfig::default());
    wm.insert("scratch", "scratch-data", SlotPriority::Scratch);
    wm.insert("system", "system-data", SlotPriority::System);
    wm.insert("context", "context-data", SlotPriority::Context);

    let assembled = wm.assemble();
    let sys_pos = assembled.find("system-data").unwrap();
    let ctx_pos = assembled.find("context-data").unwrap();
    let scr_pos = assembled.find("scratch-data").unwrap();

    assert!(sys_pos < ctx_pos);
    assert!(ctx_pos < scr_pos);
}

#[test]
fn assemble_within_budget_respects_limit() {
    let wm = WorkingMemory::new(WorkingMemoryConfig::default());
    wm.insert("a", "short", SlotPriority::System);
    wm.insert("b", "x".repeat(1000), SlotPriority::Scratch);

    let assembled = wm.assemble_within_budget(10);
    assert!(assembled.contains("short"));
    assert!(!assembled.contains(&"x".repeat(1000)));
}

#[test]
fn total_tokens_tracks_correctly() {
    let wm = WorkingMemory::new(WorkingMemoryConfig::default());
    assert_eq!(wm.total_tokens(), 0);

    wm.insert("a", "hello world", SlotPriority::Context); // ~3 tokens
    let t1 = wm.total_tokens();
    assert!(t1 > 0);

    wm.insert("b", "more data", SlotPriority::Context);
    assert!(wm.total_tokens() > t1);

    wm.remove("a");
    assert_eq!(wm.total_tokens(), wm.get("b").map(|s| s.token_estimate).unwrap_or(0));
}

#[test]
fn clear_resets_state() {
    let wm = WorkingMemory::new(WorkingMemoryConfig::default());
    wm.insert("a", "data", SlotPriority::Context);
    wm.insert("b", "more", SlotPriority::System);

    wm.clear();
    assert_eq!(wm.slot_count(), 0);
    assert_eq!(wm.total_tokens(), 0);
}

#[test]
fn concurrent_access() {
    let wm = WorkingMemory::new(WorkingMemoryConfig::default());
    let wm2 = wm.clone();

    std::thread::spawn(move || {
        for i in 0..100 {
            wm2.insert(format!("slot-{}", i), format!("data-{}", i), SlotPriority::Context);
        }
    }).join().unwrap();

    assert_eq!(wm.slot_count(), 100);
}
