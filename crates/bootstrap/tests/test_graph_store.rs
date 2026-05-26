use cmos_bootstrap::graph_store::{GraphStore, Node, Edge};

fn test_store() -> GraphStore {
    let store = GraphStore::open_in_memory().unwrap();
    store.ensure_project("test-project", "Test Project", "/tmp/test").unwrap();
    store
}

#[test]
fn insert_and_query_node() {
    let store = test_store();

    let id = store.insert_node(&Node {
        id: None,
        project_id: "test-project".into(),
        kind: "function".into(),
        label: "get_user".into(),
        file_path: Some("src/users.py".into()),
        line_start: Some(10),
        line_end: Some(20),
        properties_json: "{}".into(),
        phase_id: 1,
    }).unwrap();

    assert!(id > 0);

    let nodes = store.query_nodes_by_kind("test-project", "function").unwrap();
    assert_eq!(nodes.len(), 1);
    assert_eq!(nodes[0].label, "get_user");
    assert_eq!(nodes[0].file_path.as_deref(), Some("src/users.py"));
}

#[test]
fn insert_batch_nodes() {
    let store = test_store();

    let nodes: Vec<Node> = (0..100).map(|i| Node {
        id: None,
        project_id: "test-project".into(),
        kind: "function".into(),
        label: format!("func_{}", i),
        file_path: Some("src/main.py".into()),
        line_start: Some(i * 10),
        line_end: Some(i * 10 + 5),
        properties_json: "{}".into(),
        phase_id: 1,
    }).collect();

    let ids = store.insert_nodes_batch(&nodes).unwrap();
    assert_eq!(ids.len(), 100);

    let result = store.query_nodes_by_kind("test-project", "function").unwrap();
    assert_eq!(result.len(), 100);
}

#[test]
fn query_by_file() {
    let store = test_store();

    store.insert_node(&Node {
        id: None,
        project_id: "test-project".into(),
        kind: "function".into(),
        label: "foo".into(),
        file_path: Some("src/a.py".into()),
        line_start: Some(1),
        line_end: Some(5),
        properties_json: "{}".into(),
        phase_id: 1,
    }).unwrap();

    store.insert_node(&Node {
        id: None,
        project_id: "test-project".into(),
        kind: "function".into(),
        label: "bar".into(),
        file_path: Some("src/b.py".into()),
        line_start: Some(1),
        line_end: Some(5),
        properties_json: "{}".into(),
        phase_id: 1,
    }).unwrap();

    let a_nodes = store.query_nodes_by_file("test-project", "src/a.py").unwrap();
    assert_eq!(a_nodes.len(), 1);
    assert_eq!(a_nodes[0].label, "foo");
}

#[test]
fn insert_and_query_edges() {
    let store = test_store();

    let src_id = store.insert_node(&Node {
        id: None,
        project_id: "test-project".into(),
        kind: "django_model".into(),
        label: "Event".into(),
        file_path: Some("models.py".into()),
        line_start: Some(10),
        line_end: Some(20),
        properties_json: "{}".into(),
        phase_id: 1,
    }).unwrap();

    let tgt_id = store.insert_node(&Node {
        id: None,
        project_id: "test-project".into(),
        kind: "django_model".into(),
        label: "Artist".into(),
        file_path: Some("models.py".into()),
        line_start: Some(1),
        line_end: Some(8),
        properties_json: "{}".into(),
        phase_id: 1,
    }).unwrap();

    let edge_id = store.insert_edge(&Edge {
        id: None,
        project_id: "test-project".into(),
        source_id: src_id,
        target_id: tgt_id,
        kind: "fk_to".into(),
        properties_json: r#"{"field":"artist"}"#.into(),
        phase_id: 2,
    }).unwrap();

    assert!(edge_id > 0);

    let edge_counts = store.count_edges_by_kind("test-project").unwrap();
    assert_eq!(edge_counts.len(), 1);
    assert_eq!(edge_counts[0], ("fk_to".to_string(), 1));
}

#[test]
fn count_nodes_by_kind() {
    let store = test_store();

    for i in 0..5 {
        store.insert_node(&Node {
            id: None,
            project_id: "test-project".into(),
            kind: "function".into(),
            label: format!("fn_{}", i),
            file_path: None,
            line_start: None,
            line_end: None,
            properties_json: "{}".into(),
            phase_id: 1,
        }).unwrap();
    }

    for i in 0..3 {
        store.insert_node(&Node {
            id: None,
            project_id: "test-project".into(),
            kind: "class".into(),
            label: format!("cls_{}", i),
            file_path: None,
            line_start: None,
            line_end: None,
            properties_json: "{}".into(),
            phase_id: 1,
        }).unwrap();
    }

    let counts = store.count_nodes_by_kind("test-project").unwrap();
    let fn_count = counts.iter().find(|(k, _)| k == "function").unwrap().1;
    let cls_count = counts.iter().find(|(k, _)| k == "class").unwrap().1;
    assert_eq!(fn_count, 5);
    assert_eq!(cls_count, 3);
}

#[test]
fn checkpoint_save_and_query() {
    let store = test_store();

    store.save_checkpoint("test-project", 1, "completed", "2026-01-01T00:00:00", Some("2026-01-01T00:01:00"), None).unwrap();
    store.save_checkpoint("test-project", 2, "completed", "2026-01-01T00:01:00", Some("2026-01-01T00:02:00"), None).unwrap();
    store.save_checkpoint("test-project", 3, "failed", "2026-01-01T00:02:00", None, None).unwrap();

    let last = store.get_last_completed_phase("test-project").unwrap();
    assert_eq!(last, Some(2));
}

#[test]
fn find_node_by_label() {
    let store = test_store();

    store.insert_node(&Node {
        id: None,
        project_id: "test-project".into(),
        kind: "django_model".into(),
        label: "Artist".into(),
        file_path: Some("models.py".into()),
        line_start: Some(1),
        line_end: Some(10),
        properties_json: "{}".into(),
        phase_id: 1,
    }).unwrap();

    let found = store.find_node_id_by_label("test-project", "django_model", "Artist").unwrap();
    assert!(found.is_some());

    let not_found = store.find_node_id_by_label("test-project", "django_model", "NonExistent").unwrap();
    assert!(not_found.is_none());
}

#[test]
fn update_node_properties() {
    let store = test_store();

    let id = store.insert_node(&Node {
        id: None,
        project_id: "test-project".into(),
        kind: "convention".into(),
        label: "snake_case".into(),
        file_path: None,
        line_start: None,
        line_end: None,
        properties_json: r#"{"confidence":0.5}"#.into(),
        phase_id: 4,
    }).unwrap();

    store.update_node_properties(id, r#"{"confidence":0.9,"verified":true}"#).unwrap();

    let nodes = store.query_nodes_by_kind("test-project", "convention").unwrap();
    assert_eq!(nodes.len(), 1);
    assert!(nodes[0].properties_json.contains("0.9"));
    assert!(nodes[0].properties_json.contains("verified"));
}

#[test]
fn project_isolation() {
    let store = GraphStore::open_in_memory().unwrap();
    store.ensure_project("project-a", "Project A", "/a").unwrap();
    store.ensure_project("project-b", "Project B", "/b").unwrap();

    store.insert_node(&Node {
        id: None,
        project_id: "project-a".into(),
        kind: "function".into(),
        label: "only_in_a".into(),
        file_path: None,
        line_start: None,
        line_end: None,
        properties_json: "{}".into(),
        phase_id: 1,
    }).unwrap();

    store.insert_node(&Node {
        id: None,
        project_id: "project-b".into(),
        kind: "function".into(),
        label: "only_in_b".into(),
        file_path: None,
        line_start: None,
        line_end: None,
        properties_json: "{}".into(),
        phase_id: 1,
    }).unwrap();

    let a_nodes = store.query_nodes_by_kind("project-a", "function").unwrap();
    let b_nodes = store.query_nodes_by_kind("project-b", "function").unwrap();

    assert_eq!(a_nodes.len(), 1);
    assert_eq!(a_nodes[0].label, "only_in_a");
    assert_eq!(b_nodes.len(), 1);
    assert_eq!(b_nodes[0].label, "only_in_b");
}
