use cmos_memory::l2l3::{EventStore, MemoryEvent, EventType, Layer};

fn test_store() -> EventStore {
    EventStore::open_in_memory().unwrap()
}

fn make_event(event_type: EventType, layer: Layer, importance: f32) -> MemoryEvent {
    MemoryEvent {
        id: None,
        project_id: "test-project".into(),
        layer,
        event_type,
        entity_id: None,
        session_id: Some("session-1".into()),
        timestamp: "2026-05-26T10:00:00Z".into(),
        payload: serde_json::json!({"label": "test", "description": "test event"}),
        access_count: 0,
        importance,
    }
}

#[test]
fn append_and_query_by_session() {
    let store = test_store();
    let event = make_event(EventType::Turn, Layer::L2, 0.5);
    let id = store.append(&event).unwrap();
    assert!(id > 0);

    let events = store.query_by_session("test-project", "session-1").unwrap();
    assert_eq!(events.len(), 1);
    assert_eq!(events[0].event_type, EventType::Turn);
}

#[test]
fn query_by_time_range() {
    let store = test_store();

    let mut e1 = make_event(EventType::Decision, Layer::L2, 0.7);
    e1.timestamp = "2026-05-26T09:00:00Z".into();
    store.append(&e1).unwrap();

    let mut e2 = make_event(EventType::Lesson, Layer::L3, 0.9);
    e2.timestamp = "2026-05-26T11:00:00Z".into();
    store.append(&e2).unwrap();

    let mut e3 = make_event(EventType::Turn, Layer::L2, 0.3);
    e3.timestamp = "2026-05-26T15:00:00Z".into();
    store.append(&e3).unwrap();

    let results = store.query_by_time_range(
        "test-project",
        "2026-05-26T08:00:00Z",
        "2026-05-26T12:00:00Z",
    ).unwrap();
    assert_eq!(results.len(), 2);
}

#[test]
fn query_by_entity() {
    let store = test_store();

    let mut event = make_event(EventType::Extraction, Layer::L2, 0.6);
    event.entity_id = Some("Artist".into());
    store.append(&event).unwrap();

    let mut event2 = make_event(EventType::Turn, Layer::L2, 0.4);
    event2.entity_id = Some("Event".into());
    store.append(&event2).unwrap();

    let results = store.query_by_entity("test-project", "Artist").unwrap();
    assert_eq!(results.len(), 1);
    assert_eq!(results[0].entity_id.as_deref(), Some("Artist"));
}

#[test]
fn query_by_layer() {
    let store = test_store();

    store.append(&make_event(EventType::Turn, Layer::L2, 0.5)).unwrap();
    store.append(&make_event(EventType::Lesson, Layer::L3, 0.8)).unwrap();
    store.append(&make_event(EventType::Decision, Layer::L2, 0.7)).unwrap();

    let l2 = store.query_by_layer("test-project", Layer::L2).unwrap();
    let l3 = store.query_by_layer("test-project", Layer::L3).unwrap();
    assert_eq!(l2.len(), 2);
    assert_eq!(l3.len(), 1);
}

#[test]
fn query_by_type() {
    let store = test_store();

    store.append(&make_event(EventType::Decision, Layer::L2, 0.7)).unwrap();
    store.append(&make_event(EventType::Decision, Layer::L3, 0.8)).unwrap();
    store.append(&make_event(EventType::Turn, Layer::L2, 0.3)).unwrap();

    let decisions = store.query_by_type("test-project", EventType::Decision).unwrap();
    assert_eq!(decisions.len(), 2);
}

#[test]
fn increment_access_count() {
    let store = test_store();
    let id = store.append(&make_event(EventType::Turn, Layer::L2, 0.5)).unwrap();

    store.increment_access(id).unwrap();
    store.increment_access(id).unwrap();

    let events = store.query_by_session("test-project", "session-1").unwrap();
    assert_eq!(events[0].access_count, 2);
}

#[test]
fn promote_to_l3() {
    let store = test_store();
    let id = store.append(&make_event(EventType::Decision, Layer::L2, 0.8)).unwrap();

    store.promote_to_l3(id).unwrap();

    let l2 = store.query_by_layer("test-project", Layer::L2).unwrap();
    let l3 = store.query_by_layer("test-project", Layer::L3).unwrap();
    assert_eq!(l2.len(), 0);
    assert_eq!(l3.len(), 1);
}

#[test]
fn candidates_for_promotion() {
    let store = test_store();

    let mut high = make_event(EventType::Decision, Layer::L2, 0.9);
    high.access_count = 5;
    store.append(&high).unwrap();

    let low = make_event(EventType::Turn, Layer::L2, 0.2);
    store.append(&low).unwrap();

    let candidates = store.candidates_for_promotion("test-project", 3, 0.6).unwrap();
    assert_eq!(candidates.len(), 1);
    assert_eq!(candidates[0].importance, 0.9);
}

#[test]
fn count_by_layer() {
    let store = test_store();

    store.append(&make_event(EventType::Turn, Layer::L2, 0.5)).unwrap();
    store.append(&make_event(EventType::Turn, Layer::L2, 0.5)).unwrap();
    store.append(&make_event(EventType::Lesson, Layer::L3, 0.8)).unwrap();

    let counts = store.count_by_layer("test-project").unwrap();
    let l2_count = counts.iter().find(|(l, _)| l == "L2").map(|(_, c)| *c).unwrap_or(0);
    let l3_count = counts.iter().find(|(l, _)| l == "L3").map(|(_, c)| *c).unwrap_or(0);
    assert_eq!(l2_count, 2);
    assert_eq!(l3_count, 1);
}

#[test]
fn project_isolation() {
    let store = test_store();

    let mut e1 = make_event(EventType::Turn, Layer::L2, 0.5);
    e1.project_id = "project-a".into();
    store.append(&e1).unwrap();

    let mut e2 = make_event(EventType::Turn, Layer::L2, 0.5);
    e2.project_id = "project-b".into();
    store.append(&e2).unwrap();

    let a = store.query_by_layer("project-a", Layer::L2).unwrap();
    let b = store.query_by_layer("project-b", Layer::L2).unwrap();
    assert_eq!(a.len(), 1);
    assert_eq!(b.len(), 1);
}
