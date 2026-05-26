use std::path::Path;
use cmos_bootstrap::extractors::python::PythonExtractor;

fn fixture_path(name: &str) -> std::path::PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("tests")
        .join("fixtures")
        .join(name)
}

fn parse_fixture(name: &str) -> Vec<cmos_bootstrap::extractors::RawNode> {
    let path = fixture_path(name);
    let source = std::fs::read(&path).unwrap();
    let mut extractor = PythonExtractor::new().unwrap();
    extractor.parse_file(&source, &path).unwrap()
}

#[test]
fn extracts_functions() {
    let nodes = parse_fixture("views.py");
    let functions: Vec<_> = nodes.iter().filter(|n| n.kind == "function").collect();

    assert!(functions.iter().any(|f| f.label == "health_check"));
    assert!(functions.iter().any(|f| f.label == "search_artists"));
    assert!(functions.iter().any(|f| f.label == "plain_helper_function"));
    assert!(functions.iter().any(|f| f.label == "follow"));
}

#[test]
fn extracts_classes() {
    let nodes = parse_fixture("views.py");
    let classes: Vec<_> = nodes.iter().filter(|n| n.kind == "class").collect();

    assert!(classes.iter().any(|c| c.label == "ArtistListView"));
    assert!(classes.iter().any(|c| c.label == "ArtistViewSet"));
    assert!(classes.iter().any(|c| c.label == "EventAPIView"));
}

#[test]
fn class_has_bases() {
    let nodes = parse_fixture("views.py");
    let viewset = nodes.iter().find(|n| n.label == "ArtistViewSet").unwrap();
    let bases = viewset.properties.get("bases").unwrap().as_array().unwrap();
    let base_strs: Vec<&str> = bases.iter().filter_map(|v| v.as_str()).collect();
    assert!(base_strs.contains(&"ModelViewSet"));
}

#[test]
fn extracts_imports() {
    let nodes = parse_fixture("views.py");
    let imports: Vec<_> = nodes.iter().filter(|n| n.kind == "import").collect();

    assert!(imports.iter().any(|i| i.label.contains("ListView")));
    assert!(imports.iter().any(|i| i.label.contains("api_view")));
    assert!(!imports.is_empty());
}

#[test]
fn decorated_function_has_decorators() {
    let nodes = parse_fixture("views.py");
    let health = nodes.iter().find(|n| n.label == "health_check").unwrap();
    let decorators = health.properties.get("decorators").unwrap().as_array().unwrap();
    assert!(decorators.iter().any(|d| d.as_str().unwrap().contains("api_view")));
}

#[test]
fn line_numbers_are_correct() {
    let nodes = parse_fixture("models.py");
    let artist = nodes.iter().find(|n| n.label == "Artist" && n.kind == "class").unwrap();
    assert!(artist.line_start >= 1);
    assert!(artist.line_end > artist.line_start);
}

#[test]
fn extracts_models_with_fields() {
    let nodes = parse_fixture("models.py");
    let classes: Vec<_> = nodes.iter().filter(|n| n.kind == "class").collect();

    assert!(classes.iter().any(|c| c.label == "Artist"));
    assert!(classes.iter().any(|c| c.label == "Event"));
    assert!(classes.iter().any(|c| c.label == "Venue"));
    assert!(classes.iter().any(|c| c.label == "Tag"));
    assert!(classes.iter().any(|c| c.label == "CustomUser"));
}

#[test]
fn extracts_signal_handlers() {
    let nodes = parse_fixture("signals.py");
    let functions: Vec<_> = nodes.iter().filter(|n| n.kind == "function").collect();

    assert!(functions.iter().any(|f| f.label == "notify_on_artist_create"));
    assert!(functions.iter().any(|f| f.label == "cleanup_event_resources"));
    assert!(functions.iter().any(|f| f.label == "helper_not_a_signal"));

    let signal_fn = functions.iter().find(|f| f.label == "notify_on_artist_create").unwrap();
    let decorators = signal_fn.properties.get("decorators").unwrap().as_array().unwrap();
    assert!(decorators.iter().any(|d| d.as_str().unwrap().contains("receiver")));
}

#[test]
fn extracts_management_command() {
    let nodes = parse_fixture("management_command.py");
    let classes: Vec<_> = nodes.iter().filter(|n| n.kind == "class").collect();
    assert!(classes.iter().any(|c| c.label == "Command"));

    let cmd = classes.iter().find(|c| c.label == "Command").unwrap();
    let bases = cmd.properties.get("bases").unwrap().as_array().unwrap();
    let base_strs: Vec<&str> = bases.iter().filter_map(|v| v.as_str()).collect();
    assert!(base_strs.contains(&"BaseCommand"));
}
