use cmos_memory::l4::{ProjectMemory, Fact, FactSource};

fn test_memory() -> ProjectMemory {
    ProjectMemory::open_in_memory().unwrap()
}

#[test]
fn insert_and_query_by_kind() {
    let mem = test_memory();

    mem.insert_fact(&Fact {
        id: None,
        project_id: "test".into(),
        kind: "convention".into(),
        label: "snake_case".into(),
        description: "Use snake_case for functions".into(),
        source: FactSource::Bootstrap,
        confidence: 0.9,
        access_count: 0,
    }).unwrap();

    let facts = mem.query_by_kind("test", "convention").unwrap();
    assert_eq!(facts.len(), 1);
    assert_eq!(facts[0].label, "snake_case");
    assert_eq!(facts[0].confidence, 0.9);
}

#[test]
fn query_by_label_partial_match() {
    let mem = test_memory();

    mem.insert_fact(&Fact {
        id: None,
        project_id: "test".into(),
        kind: "decision".into(),
        label: "use_django_rest_framework".into(),
        description: "DRF for all API endpoints".into(),
        source: FactSource::UserDeclared,
        confidence: 1.0,
        access_count: 0,
    }).unwrap();

    let results = mem.query_by_label("test", "django").unwrap();
    assert_eq!(results.len(), 1);

    let no_results = mem.query_by_label("test", "flask").unwrap();
    assert!(no_results.is_empty());
}

#[test]
fn increment_access() {
    let mem = test_memory();

    let id = mem.insert_fact(&Fact {
        id: None,
        project_id: "test".into(),
        kind: "fact".into(),
        label: "test_fact".into(),
        description: "".into(),
        source: FactSource::Inferred,
        confidence: 0.5,
        access_count: 0,
    }).unwrap();

    mem.increment_access(id).unwrap();
    mem.increment_access(id).unwrap();

    let facts = mem.query_by_kind("test", "fact").unwrap();
    assert_eq!(facts[0].access_count, 2);
}

#[test]
fn tombstone_hides_fact() {
    let mem = test_memory();

    let id = mem.insert_fact(&Fact {
        id: None,
        project_id: "test".into(),
        kind: "lesson".into(),
        label: "old_lesson".into(),
        description: "superseded".into(),
        source: FactSource::Inferred,
        confidence: 0.5,
        access_count: 0,
    }).unwrap();

    mem.tombstone(id).unwrap();

    let facts = mem.query_by_kind("test", "lesson").unwrap();
    assert!(facts.is_empty());
}

#[test]
fn promotion_source_tracks_event_id() {
    let mem = test_memory();

    mem.insert_fact(&Fact {
        id: None,
        project_id: "test".into(),
        kind: "decision".into(),
        label: "promoted_decision".into(),
        description: "came from L3".into(),
        source: FactSource::Promotion { from_event_id: 42 },
        confidence: 0.85,
        access_count: 5,
    }).unwrap();

    let facts = mem.query_by_kind("test", "decision").unwrap();
    assert_eq!(facts.len(), 1);
    match &facts[0].source {
        FactSource::Promotion { from_event_id } => assert_eq!(*from_event_id, 42),
        _ => panic!("expected Promotion source"),
    }
}

#[test]
fn count_by_kind() {
    let mem = test_memory();

    for i in 0..3 {
        mem.insert_fact(&Fact {
            id: None,
            project_id: "test".into(),
            kind: "convention".into(),
            label: format!("conv_{}", i),
            description: "".into(),
            source: FactSource::Bootstrap,
            confidence: 0.8,
            access_count: 0,
        }).unwrap();
    }

    mem.insert_fact(&Fact {
        id: None,
        project_id: "test".into(),
        kind: "decision".into(),
        label: "dec_1".into(),
        description: "".into(),
        source: FactSource::UserDeclared,
        confidence: 1.0,
        access_count: 0,
    }).unwrap();

    let counts = mem.count_by_kind("test").unwrap();
    let conv = counts.iter().find(|(k, _)| k == "convention").unwrap().1;
    let dec = counts.iter().find(|(k, _)| k == "decision").unwrap().1;
    assert_eq!(conv, 3);
    assert_eq!(dec, 1);
}
