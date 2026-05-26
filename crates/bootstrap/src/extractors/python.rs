use std::path::Path;
use tree_sitter::{Node as TsNode, Parser};

use super::{ExtractorError, RawNode};

pub struct PythonExtractor {
    parser: Parser,
}

impl PythonExtractor {
    pub fn new() -> Result<Self, ExtractorError> {
        let mut parser = Parser::new();
        let language = tree_sitter_python::LANGUAGE;
        parser.set_language(&language.into())
            .map_err(|e| ExtractorError::ParseError("init".into(), e.to_string()))?;
        Ok(Self { parser })
    }

    pub fn parse_file(&mut self, source: &[u8], file_path: &Path) -> Result<Vec<RawNode>, ExtractorError> {
        let tree = self.parser.parse(source, None)
            .ok_or_else(|| ExtractorError::ParseError(
                file_path.display().to_string(),
                "tree-sitter returned None".into(),
            ))?;

        let mut nodes = Vec::new();
        let root = tree.root_node();
        self.visit_node(root, source, file_path, &mut nodes);
        Ok(nodes)
    }

    fn visit_node(&self, node: TsNode, source: &[u8], file_path: &Path, out: &mut Vec<RawNode>) {
        match node.kind() {
            "function_definition" => {
                if let Some(raw) = self.extract_function(node, source, file_path) {
                    out.push(raw);
                }
            }
            "class_definition" => {
                if let Some(raw) = self.extract_class(node, source, file_path) {
                    out.push(raw);
                }
            }
            "import_statement" | "import_from_statement" => {
                if let Some(raw) = self.extract_import(node, source, file_path) {
                    out.push(raw);
                }
            }
            "decorated_definition" => {
                let decorators = self.collect_decorators(node, source);
                if let Some(child) = node.child_by_field_name("definition") {
                    match child.kind() {
                        "function_definition" => {
                            if let Some(mut raw) = self.extract_function(child, source, file_path) {
                                if let serde_json::Value::Object(ref mut map) = raw.properties {
                                    map.insert("decorators".into(), serde_json::json!(decorators));
                                }
                                out.push(raw);
                            }
                        }
                        "class_definition" => {
                            if let Some(mut raw) = self.extract_class(child, source, file_path) {
                                if let serde_json::Value::Object(ref mut map) = raw.properties {
                                    map.insert("decorators".into(), serde_json::json!(decorators));
                                }
                                out.push(raw);
                            }
                        }
                        _ => {}
                    }
                }
                return;
            }
            _ => {}
        }

        let child_count = node.child_count();
        for i in 0..child_count {
            if let Some(child) = node.child(i) {
                self.visit_node(child, source, file_path, out);
            }
        }
    }

    fn extract_function(&self, node: TsNode, source: &[u8], file_path: &Path) -> Option<RawNode> {
        let name_node = node.child_by_field_name("name")?;
        let name = name_node.utf8_text(source).ok()?;

        let params = node.child_by_field_name("parameters")
            .and_then(|p| p.utf8_text(source).ok())
            .unwrap_or("()");

        Some(RawNode {
            kind: "function".into(),
            label: name.to_string(),
            file_path: file_path.to_string_lossy().to_string(),
            line_start: node.start_position().row as u32 + 1,
            line_end: node.end_position().row as u32 + 1,
            properties: serde_json::json!({
                "parameters": params,
            }),
        })
    }

    fn extract_class(&self, node: TsNode, source: &[u8], file_path: &Path) -> Option<RawNode> {
        let name_node = node.child_by_field_name("name")?;
        let name = name_node.utf8_text(source).ok()?;

        let bases = self.collect_bases(node, source);

        Some(RawNode {
            kind: "class".into(),
            label: name.to_string(),
            file_path: file_path.to_string_lossy().to_string(),
            line_start: node.start_position().row as u32 + 1,
            line_end: node.end_position().row as u32 + 1,
            properties: serde_json::json!({
                "bases": bases,
            }),
        })
    }

    fn extract_import(&self, node: TsNode, source: &[u8], file_path: &Path) -> Option<RawNode> {
        let text = node.utf8_text(source).ok()?;

        Some(RawNode {
            kind: "import".into(),
            label: text.trim().to_string(),
            file_path: file_path.to_string_lossy().to_string(),
            line_start: node.start_position().row as u32 + 1,
            line_end: node.end_position().row as u32 + 1,
            properties: serde_json::json!({}),
        })
    }

    fn collect_bases(&self, class_node: TsNode, source: &[u8]) -> Vec<String> {
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

    fn collect_decorators(&self, decorated_node: TsNode, source: &[u8]) -> Vec<String> {
        let mut decorators = Vec::new();
        let count = decorated_node.child_count();
        for i in 0..count {
            if let Some(child) = decorated_node.child(i)
                && child.kind() == "decorator"
                && let Ok(text) = child.utf8_text(source)
            {
                decorators.push(text.trim_start_matches('@').trim().to_string());
            }
        }
        decorators
    }
}

impl super::LanguageExtractor for PythonExtractor {
    fn language(&self) -> &str {
        "python"
    }

    fn file_extensions(&self) -> &[&str] {
        &["py"]
    }

    fn extract_symbols(&self, source: &[u8], path: &Path) -> Result<Vec<RawNode>, ExtractorError> {
        let mut parser = Parser::new();
        let language = tree_sitter_python::LANGUAGE;
        parser.set_language(&language.into())
            .map_err(|e| ExtractorError::ParseError("init".into(), e.to_string()))?;

        let tree = parser.parse(source, None)
            .ok_or_else(|| ExtractorError::ParseError(
                path.display().to_string(),
                "tree-sitter returned None".into(),
            ))?;

        let mut nodes = Vec::new();
        let root = tree.root_node();
        self.visit_node(root, source, path, &mut nodes);
        Ok(nodes)
    }
}
