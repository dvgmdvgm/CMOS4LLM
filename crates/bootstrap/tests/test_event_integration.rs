use cmos_memory::{EventStore, Layer, EventType};
use cmos_bootstrap::PipelineRunner;
use tempfile::TempDir;

fn setup_minimal_project(dir: &std::path::Path) {
    let cmos_dir = dir.join(".cmos");
    std::fs::create_dir_all(&cmos_dir).unwrap();

    let src_dir = dir.join("src");
    std::fs::create_dir_all(&src_dir).unwrap();
    std::fs::write(
        src_dir.join("main.py"),
        "class UserService:\n    def get_user(self, user_id: int):\n        pass\n",
    ).unwrap();
}

#[test]
fn bootstrap_emits_l2_events() {
    let tmp = TempDir::new().unwrap();
    let root = tmp.path().to_path_buf();
    setup_minimal_project(&root);

    let mut runner = PipelineRunner::new("test-project", root.clone());
    runner.no_interactive = true;
    runner.skip_phases = vec![4, 5, 6, 7, 8];

    runner.run().unwrap();

    let events_path = root.join(".cmos").join("events.db");
    assert!(events_path.exists(), "events.db should be created");

    let es = EventStore::open(&events_path).unwrap();
    let events = es.query_by_layer("test-project", Layer::L2).unwrap();

    assert!(!events.is_empty(), "should have L2 events after bootstrap");

    for event in &events {
        assert_eq!(event.event_type, EventType::Extraction);
        assert_eq!(event.project_id, "test-project");
        assert!(event.session_id.as_deref() == Some("bootstrap"));
        assert!(event.entity_id.as_ref().unwrap().starts_with("bootstrap:"));

        let summary = event.payload.get("summary").and_then(|v| v.as_str()).unwrap();
        assert!(summary.contains("Bootstrap phase"));
    }
}

#[test]
fn bootstrap_events_contain_phase_stats() {
    let tmp = TempDir::new().unwrap();
    let root = tmp.path().to_path_buf();
    setup_minimal_project(&root);

    let mut runner = PipelineRunner::new("stats-project", root.clone());
    runner.no_interactive = true;
    runner.skip_phases = vec![2, 3, 4, 5, 6, 7, 8];

    runner.run().unwrap();

    let events_path = root.join(".cmos").join("events.db");
    let es = EventStore::open(&events_path).unwrap();
    let events = es.query_by_layer("stats-project", Layer::L2).unwrap();

    assert_eq!(events.len(), 1, "one phase ran, one event expected");

    let event = &events[0];
    assert!(event.payload.get("nodes_created").and_then(|v| v.as_u64()).is_some());
    assert!(event.payload.get("edges_created").and_then(|v| v.as_u64()).is_some());

    let phase = event.payload.get("phase").and_then(|v| v.as_str()).unwrap();
    assert_eq!(phase, "Static AST Sweep");
}

#[test]
fn skipped_phases_do_not_emit_events() {
    let tmp = TempDir::new().unwrap();
    let root = tmp.path().to_path_buf();
    setup_minimal_project(&root);

    let mut runner = PipelineRunner::new("skip-project", root.clone());
    runner.no_interactive = true;
    runner.skip_phases = vec![1, 2, 3, 4, 5, 6, 7, 8];

    runner.run().unwrap();

    let events_path = root.join(".cmos").join("events.db");
    let es = EventStore::open(&events_path).unwrap();
    let events = es.query_by_layer("skip-project", Layer::L2).unwrap();

    assert!(events.is_empty(), "skipped phases should not emit events");
}
