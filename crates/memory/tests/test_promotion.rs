use cmos_memory::l2l3::{EventStore, MemoryEvent, EventType, Layer};
use cmos_memory::l4::ProjectMemory;
use cmos_memory::promotion::{PromotionEngine, PromotionConfig};

fn make_event(event_type: EventType, importance: f32, access_count: u32) -> MemoryEvent {
    MemoryEvent {
        id: None,
        project_id: "test".into(),
        layer: Layer::L2,
        event_type,
        entity_id: None,
        session_id: Some("s1".into()),
        timestamp: "2026-05-26T10:00:00Z".into(),
        payload: serde_json::json!({"label": "test_item", "description": "a test"}),
        access_count,
        importance,
    }
}

#[test]
fn l2_to_l3_promotes_qualifying_events() {
    let store = EventStore::open_in_memory().unwrap();
    let engine = PromotionEngine::new(PromotionConfig::default());

    let high = make_event(EventType::Decision, 0.8, 5);
    store.append(&high).unwrap();

    let low = make_event(EventType::Turn, 0.2, 0);
    store.append(&low).unwrap();

    let promoted = engine.run_l2_to_l3(&store, "test").unwrap();
    assert_eq!(promoted.len(), 1);

    let l3 = store.query_by_layer("test", Layer::L3).unwrap();
    assert_eq!(l3.len(), 1);
    assert_eq!(l3[0].event_type, EventType::Decision);
}

#[test]
fn l2_to_l3_skips_low_importance() {
    let store = EventStore::open_in_memory().unwrap();
    let engine = PromotionEngine::new(PromotionConfig::default());

    let event = make_event(EventType::Decision, 0.3, 10);
    store.append(&event).unwrap();

    let promoted = engine.run_l2_to_l3(&store, "test").unwrap();
    assert!(promoted.is_empty());
}

#[test]
fn l3_to_l4_promotes_qualifying_events() {
    let store = EventStore::open_in_memory().unwrap();
    let project_mem = ProjectMemory::open_in_memory().unwrap();
    let engine = PromotionEngine::new(PromotionConfig::default());

    let mut event = make_event(EventType::Decision, 0.9, 6);
    event.layer = Layer::L3;
    store.append(&event).unwrap();

    let promoted = engine.run_l3_to_l4(&store, &project_mem, "test").unwrap();
    assert_eq!(promoted.len(), 1);

    let facts = project_mem.query_by_kind("test", "decision").unwrap();
    assert_eq!(facts.len(), 1);
    assert_eq!(facts[0].label, "test_item");
}

#[test]
fn l3_to_l4_skips_wrong_event_types() {
    let store = EventStore::open_in_memory().unwrap();
    let project_mem = ProjectMemory::open_in_memory().unwrap();
    let engine = PromotionEngine::new(PromotionConfig::default());

    let mut event = make_event(EventType::Turn, 0.9, 10);
    event.layer = Layer::L3;
    store.append(&event).unwrap();

    let promoted = engine.run_l3_to_l4(&store, &project_mem, "test").unwrap();
    assert!(promoted.is_empty());
}

#[test]
fn full_promotion_pipeline() {
    let store = EventStore::open_in_memory().unwrap();
    let project_mem = ProjectMemory::open_in_memory().unwrap();
    let engine = PromotionEngine::new(PromotionConfig {
        l2_to_l3_min_access: 2,
        l2_to_l3_min_importance: 0.5,
        l3_to_l4_min_access: 2,
        l3_to_l4_min_importance: 0.5,
        l3_to_l4_event_types: vec![EventType::Decision, EventType::Lesson],
    });

    // Start in L2 with qualifying stats
    let event = make_event(EventType::Lesson, 0.7, 3);
    store.append(&event).unwrap();

    // Promote L2 → L3
    let promoted_l3 = engine.run_l2_to_l3(&store, "test").unwrap();
    assert_eq!(promoted_l3.len(), 1);

    // Now promote L3 → L4
    let promoted_l4 = engine.run_l3_to_l4(&store, &project_mem, "test").unwrap();
    assert_eq!(promoted_l4.len(), 1);

    let facts = project_mem.query_by_kind("test", "lesson").unwrap();
    assert_eq!(facts.len(), 1);
    assert_eq!(facts[0].label, "test_item");
}
