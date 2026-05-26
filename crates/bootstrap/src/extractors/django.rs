use std::path::Path;
use tree_sitter::{Node as TsNode, Parser};

use super::{ExtractorError, RawNode};

pub struct DjangoExtractor;

impl Default for DjangoExtractor {
    fn default() -> Self {
        Self
    }
}

impl DjangoExtractor {
    pub fn new() -> Self {
        Self
    }

    pub fn classify_node(&self, node: &RawNode) -> Option<RawNode> {
        match node.kind.as_str() {
            "class" => self.classify_class(node),
            "function" => self.classify_function(node),
            _ => None,
        }
    }

    fn classify_class(&self, node: &RawNode) -> Option<RawNode> {
        let bases = node.properties.get("bases")
            .and_then(|b| b.as_array())
            .map(|arr| arr.iter().filter_map(|v| v.as_str()).collect::<Vec<_>>())
            .unwrap_or_default();

        // Order matters: check more specific types before generic ones.
        // Admin, ViewSet, Serializer all contain "Model" in their base names,
        // so they must be checked before is_django_model.

        if self.is_admin(&bases) {
            let mut classified = node.clone();
            classified.kind = "django_admin".into();
            return Some(classified);
        }

        if self.is_django_view(&bases) {
            let mut classified = node.clone();
            classified.kind = "django_view".into();
            return Some(classified);
        }

        if self.is_serializer(&bases) {
            let mut classified = node.clone();
            classified.kind = "serializer".into();
            return Some(classified);
        }

        if self.is_form(&bases) {
            let mut classified = node.clone();
            classified.kind = "django_form".into();
            return Some(classified);
        }

        if self.is_middleware(&bases) || self.looks_like_middleware(&node.label) {
            let mut classified = node.clone();
            classified.kind = "middleware".into();
            return Some(classified);
        }

        if self.is_consumer(&bases) {
            let mut classified = node.clone();
            classified.kind = "websocket_consumer".into();
            return Some(classified);
        }

        if self.is_management_command(&bases) {
            let mut classified = node.clone();
            classified.kind = "management_command".into();
            return Some(classified);
        }

        if self.is_django_model(&bases) {
            let mut classified = node.clone();
            classified.kind = "django_model".into();
            return Some(classified);
        }

        None
    }

    fn classify_function(&self, node: &RawNode) -> Option<RawNode> {
        let decorators = node.properties.get("decorators")
            .and_then(|d| d.as_array())
            .map(|arr| arr.iter().filter_map(|v| v.as_str()).collect::<Vec<_>>())
            .unwrap_or_default();

        if decorators.iter().any(|d| d.contains("api_view") || d.contains("action")) {
            let mut classified = node.clone();
            classified.kind = "django_view".into();
            return Some(classified);
        }

        if decorators.iter().any(|d| d.contains("receiver")) {
            let mut classified = node.clone();
            classified.kind = "signal_handler".into();
            return Some(classified);
        }

        None
    }

    fn is_admin(&self, bases: &[&str]) -> bool {
        bases.iter().any(|b| {
            b.contains("ModelAdmin")
                || b.contains("admin.ModelAdmin")
                || b.contains("TabularInline")
                || b.contains("StackedInline")
                || b.contains("AdminSite")
        })
    }

    fn is_django_model(&self, bases: &[&str]) -> bool {
        let model_bases = [
            "models.Model", "Model", "AbstractUser", "AbstractBaseUser",
            "PermissionsMixin", "TimeStampedModel", "AbstractModel",
        ];
        let non_model_indicators = [
            "Admin", "Serializer", "View", "Form", "Mixin", "Command",
            "Consumer", "Middleware", "Filter", "Permission", "Inline",
        ];
        bases.iter().any(|b| {
            let matches_model = model_bases.iter().any(|mb| b.contains(mb));
            let is_actually_something_else = non_model_indicators.iter().any(|nm| b.contains(nm));
            matches_model && !is_actually_something_else
        })
    }

    fn is_django_view(&self, bases: &[&str]) -> bool {
        let view_bases = [
            "View", "TemplateView", "ListView", "DetailView", "CreateView",
            "UpdateView", "DeleteView", "FormView", "RedirectView",
            "APIView", "GenericAPIView", "ModelViewSet", "ViewSet",
            "ViewSetMixin", "GenericViewSet",
        ];
        bases.iter().any(|b| view_bases.iter().any(|vb| b.contains(vb)))
    }

    fn is_serializer(&self, bases: &[&str]) -> bool {
        bases.iter().any(|b| {
            b.contains("Serializer")
                || b.contains("ModelSerializer")
                || b.contains("HyperlinkedModelSerializer")
        })
    }

    fn is_form(&self, bases: &[&str]) -> bool {
        bases.iter().any(|b| b.contains("Form") || b.contains("ModelForm"))
    }

    fn is_middleware(&self, bases: &[&str]) -> bool {
        bases.iter().any(|b| b.contains("Middleware") || b.contains("MiddlewareMixin"))
    }

    fn looks_like_middleware(&self, name: &str) -> bool {
        name.to_lowercase().contains("middleware")
    }

    fn is_consumer(&self, bases: &[&str]) -> bool {
        bases.iter().any(|b| {
            b.contains("WebsocketConsumer")
                || b.contains("AsyncWebsocketConsumer")
                || b.contains("JsonWebsocketConsumer")
                || b.contains("Consumer")
        })
    }

    fn is_management_command(&self, bases: &[&str]) -> bool {
        bases.iter().any(|b| b.contains("BaseCommand") || b.contains("Command"))
    }

    pub fn extract_model_fields(&self, source: &[u8], file_path: &Path) -> Result<Vec<ModelFieldInfo>, ExtractorError> {
        let mut parser = Parser::new();
        let language = tree_sitter_python::LANGUAGE;
        parser.set_language(&language.into())
            .map_err(|e| ExtractorError::ParseError("init".into(), e.to_string()))?;

        let tree = parser.parse(source, None)
            .ok_or_else(|| ExtractorError::ParseError(
                file_path.display().to_string(),
                "tree-sitter returned None".into(),
            ))?;

        let mut fields = Vec::new();
        self.visit_for_fields(tree.root_node(), source, &mut fields);
        Ok(fields)
    }

    fn visit_for_fields(&self, node: TsNode, source: &[u8], out: &mut Vec<ModelFieldInfo>) {
        if node.kind() == "class_definition" {
            let class_name = node.child_by_field_name("name")
                .and_then(|n| n.utf8_text(source).ok())
                .unwrap_or("");

            let bases = self.get_bases_from_node(node, source);
            if self.is_django_model(&bases.iter().map(|s| s.as_str()).collect::<Vec<_>>()) {
                self.extract_fields_from_class(node, source, class_name, out);
            }
        }

        let count = node.child_count();
        for i in 0..count {
            if let Some(child) = node.child(i) {
                self.visit_for_fields(child, source, out);
            }
        }
    }

    fn get_bases_from_node(&self, class_node: TsNode, source: &[u8]) -> Vec<String> {
        let mut bases = Vec::new();
        if let Some(arg_list) = class_node.child_by_field_name("superclasses") {
            let count = arg_list.child_count();
            for i in 0..count {
                if let Some(child) = arg_list.child(i)
                    && child.kind() != "(" && child.kind() != ")" && child.kind() != ","
                    && let Ok(text) = child.utf8_text(source)
                {
                    bases.push(text.trim().to_string());
                }
            }
        }
        bases
    }

    fn extract_fields_from_class(&self, class_node: TsNode, source: &[u8], class_name: &str, out: &mut Vec<ModelFieldInfo>) {
        if let Some(body) = class_node.child_by_field_name("body") {
            let count = body.child_count();
            for i in 0..count {
                if let Some(child) = body.child(i)
                    && child.kind() == "expression_statement"
                    && let Some(assignment) = child.child(0)
                    && assignment.kind() == "assignment"
                    && let Some(field_info) = self.parse_field_assignment(assignment, source, class_name)
                {
                    out.push(field_info);
                }
            }
        }
    }

    fn parse_field_assignment(&self, assignment: TsNode, source: &[u8], class_name: &str) -> Option<ModelFieldInfo> {
        let left = assignment.child_by_field_name("left")?;
        let right = assignment.child_by_field_name("right")?;

        let field_name = left.utf8_text(source).ok()?;
        let value_text = right.utf8_text(source).ok()?;

        if !value_text.contains("models.") && !value_text.contains("Field") {
            return None;
        }

        let field_type = self.extract_field_type(value_text);
        let relation_target = self.extract_relation_target(value_text);

        Some(ModelFieldInfo {
            model_name: class_name.to_string(),
            field_name: field_name.to_string(),
            field_type,
            relation_target,
            line: assignment.start_position().row as u32 + 1,
        })
    }

    fn extract_field_type(&self, value_text: &str) -> String {
        if let Some(paren_pos) = value_text.find('(') {
            value_text[..paren_pos].trim().to_string()
        } else {
            value_text.trim().to_string()
        }
    }

    fn extract_relation_target(&self, value_text: &str) -> Option<String> {
        let is_relation = value_text.contains("ForeignKey")
            || value_text.contains("OneToOneField")
            || value_text.contains("ManyToManyField");

        if !is_relation {
            return None;
        }

        if let Some(paren_pos) = value_text.find('(') {
            let args = &value_text[paren_pos + 1..];
            let first_arg = args.split(',').next()?;
            let target = first_arg.trim()
                .trim_matches(|c| c == '\'' || c == '"')
                .trim();
            if !target.is_empty() && target != "self" {
                return Some(target.to_string());
            }
        }
        None
    }

    pub fn extract_url_patterns(&self, source: &[u8], file_path: &Path) -> Result<Vec<RawNode>, ExtractorError> {
        let content = std::str::from_utf8(source)
            .map_err(|e| ExtractorError::ParseError(file_path.display().to_string(), e.to_string()))?;

        let mut urls = Vec::new();
        let file_str = file_path.to_string_lossy().to_string();

        for (line_num, line) in content.lines().enumerate() {
            let trimmed = line.trim();
            if (trimmed.contains("path(") || trimmed.contains("re_path(") || trimmed.contains("url("))
                && let Some(url_info) = self.parse_url_line(trimmed)
            {
                urls.push(RawNode {
                    kind: "django_url".into(),
                    label: url_info.pattern.clone(),
                    file_path: file_str.clone(),
                    line_start: line_num as u32 + 1,
                    line_end: line_num as u32 + 1,
                    properties: serde_json::json!({
                        "pattern": url_info.pattern,
                        "view": url_info.view_name,
                        "name": url_info.name,
                    }),
                });
            }
        }

        Ok(urls)
    }

    fn parse_url_line(&self, line: &str) -> Option<UrlInfo> {
        let start = line.find('(')? + 1;
        let args_str = &line[start..];

        let parts: Vec<&str> = args_str.splitn(3, ',').collect();
        if parts.is_empty() {
            return None;
        }

        let pattern = parts[0].trim()
            .trim_matches(|c| c == '\'' || c == '"')
            .to_string();

        let view_name = parts.get(1)
            .map(|v| v.trim().trim_end_matches(')').trim().to_string())
            .unwrap_or_default();

        let name = args_str.split("name=").nth(1).map(|name_part| {
            name_part.trim()
                .trim_matches(|c| c == '\'' || c == '"' || c == ')' || c == ',')
                .to_string()
        });

        Some(UrlInfo { pattern, view_name, name })
    }

    pub fn extract_settings_middleware(&self, source: &[u8]) -> Vec<String> {
        let content = match std::str::from_utf8(source) {
            Ok(c) => c,
            Err(_) => return Vec::new(),
        };

        let mut middleware = Vec::new();
        let mut in_middleware = false;

        for line in content.lines() {
            let trimmed = line.trim();
            if trimmed.starts_with("MIDDLEWARE") && trimmed.contains('[') {
                in_middleware = true;
                continue;
            }
            if in_middleware {
                if trimmed.contains(']') {
                    break;
                }
                let entry = trimmed.trim_matches(|c| c == '\'' || c == '"' || c == ',').trim();
                if !entry.is_empty() && !entry.starts_with('#') {
                    middleware.push(entry.to_string());
                }
            }
        }

        middleware
    }
}

#[derive(Debug, Clone)]
pub struct ModelFieldInfo {
    pub model_name: String,
    pub field_name: String,
    pub field_type: String,
    pub relation_target: Option<String>,
    pub line: u32,
}

#[derive(Debug)]
struct UrlInfo {
    pattern: String,
    view_name: String,
    name: Option<String>,
}
