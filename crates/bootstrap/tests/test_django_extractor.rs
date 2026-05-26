use std::path::Path;
use cmos_bootstrap::extractors::python::PythonExtractor;
use cmos_bootstrap::extractors::django::DjangoExtractor;
use cmos_bootstrap::extractors::RawNode;

fn fixture_path(name: &str) -> std::path::PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("tests")
        .join("fixtures")
        .join(name)
}

fn parse_and_classify(name: &str) -> Vec<RawNode> {
    let path = fixture_path(name);
    let source = std::fs::read(&path).unwrap();
    let mut extractor = PythonExtractor::new().unwrap();
    let raw_nodes = extractor.parse_file(&source, &path).unwrap();

    let django = DjangoExtractor::new();
    let mut result = Vec::new();
    for node in &raw_nodes {
        if let Some(classified) = django.classify_node(node) {
            result.push(classified);
        } else {
            result.push(node.clone());
        }
    }
    result
}

#[test]
fn classifies_views_from_cbv() {
    let nodes = parse_and_classify("views.py");
    let views: Vec<_> = nodes.iter().filter(|n| n.kind == "django_view").collect();

    assert!(views.iter().any(|v| v.label == "ArtistListView"));
    assert!(views.iter().any(|v| v.label == "ArtistDetailView"));
    assert!(views.iter().any(|v| v.label == "EventCreateView"));
    assert!(views.iter().any(|v| v.label == "ArtistViewSet"));
    assert!(views.iter().any(|v| v.label == "EventAPIView"));
}

#[test]
fn classifies_views_from_decorator() {
    let nodes = parse_and_classify("views.py");
    let views: Vec<_> = nodes.iter().filter(|n| n.kind == "django_view").collect();

    assert!(views.iter().any(|v| v.label == "health_check"));
    assert!(views.iter().any(|v| v.label == "search_artists"));
    assert!(views.iter().any(|v| v.label == "follow"));
}

#[test]
fn plain_function_not_classified_as_view() {
    let nodes = parse_and_classify("views.py");
    let helper = nodes.iter().find(|n| n.label == "plain_helper_function").unwrap();
    assert_eq!(helper.kind, "function");
}

#[test]
fn classifies_admins() {
    let nodes = parse_and_classify("admin.py");
    let admins: Vec<_> = nodes.iter().filter(|n| n.kind == "django_admin").collect();

    assert!(admins.iter().any(|a| a.label == "ArtistAdmin"));
    assert!(admins.iter().any(|a| a.label == "EventAdmin"));
    assert!(admins.iter().any(|a| a.label == "VenueInline"));
}

#[test]
fn admin_not_classified_as_model() {
    let nodes = parse_and_classify("admin.py");
    let models: Vec<_> = nodes.iter().filter(|n| n.kind == "django_model").collect();
    assert!(models.is_empty(), "Admin classes should not be classified as models");
}

#[test]
fn classifies_serializers() {
    let nodes = parse_and_classify("serializers.py");
    let serializers: Vec<_> = nodes.iter().filter(|n| n.kind == "serializer").collect();

    assert!(serializers.iter().any(|s| s.label == "ArtistSerializer"));
    assert!(serializers.iter().any(|s| s.label == "EventSerializer"));
    assert!(serializers.iter().any(|s| s.label == "TagSerializer"));
}

#[test]
fn classifies_models() {
    let nodes = parse_and_classify("models.py");
    let models: Vec<_> = nodes.iter().filter(|n| n.kind == "django_model").collect();

    assert!(models.iter().any(|m| m.label == "Artist"));
    assert!(models.iter().any(|m| m.label == "Event"));
    assert!(models.iter().any(|m| m.label == "Venue"));
    assert!(models.iter().any(|m| m.label == "Tag"));
    assert!(models.iter().any(|m| m.label == "CustomUser"));
}

#[test]
fn classifies_signal_handlers() {
    let nodes = parse_and_classify("signals.py");
    let signals: Vec<_> = nodes.iter().filter(|n| n.kind == "signal_handler").collect();

    assert!(signals.iter().any(|s| s.label == "notify_on_artist_create"));
    assert!(signals.iter().any(|s| s.label == "cleanup_event_resources"));
    assert_eq!(signals.len(), 2);
}

#[test]
fn helper_in_signals_not_classified() {
    let nodes = parse_and_classify("signals.py");
    let helper = nodes.iter().find(|n| n.label == "helper_not_a_signal").unwrap();
    assert_eq!(helper.kind, "function");
}

#[test]
fn classifies_management_commands() {
    let nodes = parse_and_classify("management_command.py");
    let commands: Vec<_> = nodes.iter().filter(|n| n.kind == "management_command").collect();
    assert_eq!(commands.len(), 1);
    assert_eq!(commands[0].label, "Command");
}

#[test]
fn classifies_middleware() {
    let nodes = parse_and_classify("middleware.py");
    let mw: Vec<_> = nodes.iter().filter(|n| n.kind == "middleware").collect();

    assert!(mw.iter().any(|m| m.label == "RequestTimingMiddleware"));
    assert!(mw.iter().any(|m| m.label == "TenantMiddleware"));
}

#[test]
fn classifies_consumers() {
    let nodes = parse_and_classify("consumers.py");
    let consumers: Vec<_> = nodes.iter().filter(|n| n.kind == "websocket_consumer").collect();

    assert!(consumers.iter().any(|c| c.label == "ChatConsumer"));
    assert!(consumers.iter().any(|c| c.label == "NotificationConsumer"));
}

#[test]
fn extracts_url_patterns() {
    let path = fixture_path("urls.py");
    let source = std::fs::read(&path).unwrap();
    let django = DjangoExtractor::new();
    let urls = django.extract_url_patterns(&source, &path).unwrap();

    assert!(urls.len() >= 5);
    assert!(urls.iter().any(|u| u.label.contains("artists/")));
    assert!(urls.iter().any(|u| u.label.contains("api/health/")));

    for url in &urls {
        assert_eq!(url.kind, "django_url");
        assert!(url.line_start > 0);
    }
}

#[test]
fn extracts_model_fields_with_relations() {
    let path = fixture_path("models.py");
    let source = std::fs::read(&path).unwrap();
    let django = DjangoExtractor::new();
    let fields = django.extract_model_fields(&source, &path).unwrap();

    let fk_fields: Vec<_> = fields.iter().filter(|f| f.relation_target.is_some()).collect();
    assert!(fk_fields.iter().any(|f| f.model_name == "Event" && f.field_name == "artist"));
    assert!(fk_fields.iter().any(|f| f.model_name == "Event" && f.field_name == "venue"));
    assert!(fk_fields.iter().any(|f| f.model_name == "Event" && f.field_name == "tags"));

    let artist_fk = fk_fields.iter().find(|f| f.field_name == "artist").unwrap();
    assert_eq!(artist_fk.relation_target.as_deref(), Some("Artist"));
}

#[test]
fn extracts_settings_middleware() {
    let path = fixture_path("settings.py");
    let source = std::fs::read(&path).unwrap();
    let django = DjangoExtractor::new();
    let middleware = django.extract_settings_middleware(&source);

    assert_eq!(middleware.len(), 7);
    assert!(middleware.contains(&"django.middleware.security.SecurityMiddleware".to_string()));
    assert!(middleware.contains(&"core.middleware.RequestTimingMiddleware".to_string()));
    assert!(middleware.contains(&"core.middleware.TenantMiddleware".to_string()));
}
