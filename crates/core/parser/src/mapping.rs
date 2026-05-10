use std::path::Path;

use anyhow::{Result, anyhow};
use arborium::{get_language, tree_sitter};
use facet::Facet;

use crate::{
    ast::{FunctionContract, SpecItem},
    parser::{Parser, parse_spec_items},
};

enum Annotation {
    Contract(String),
    TypeAlias(String),
    Assert(String),
    Other,
}

pub struct StructuralMapper {
    // We can add a GrammarStore here if needed for caching
}

#[derive(Debug, Clone, PartialEq, Facet)]
pub struct Mapping {
    pub target_range: std::ops::Range<usize>,
    pub contract: FunctionContract,
}

#[derive(Debug, Clone, PartialEq, Facet)]
pub struct SpecItemMapping {
    pub target_range: Option<std::ops::Range<usize>>,
    pub item: SpecItem,
}

impl StructuralMapper {
    pub fn new() -> Self {
        Self {}
    }

    pub fn map_file(&self, path: &Path) -> Result<Vec<Mapping>> {
        let items = self.map_file_items(path)?;
        let mut mappings = Vec::new();
        for item in items {
            if let (Some(target_range), SpecItem::FunctionContract(contract)) =
                (item.target_range, item.item)
            {
                mappings.push(Mapping {
                    target_range,
                    contract,
                });
            }
        }

        Ok(mappings)
    }

    pub fn map_file_items(&self, path: &Path) -> Result<Vec<SpecItemMapping>> {
        let extension = path
            .extension()
            .and_then(|ext| ext.to_str())
            .ok_or_else(|| anyhow!("No extension for file {:?}", path))?;

        if extension == "lmt" {
            let source = std::fs::read_to_string(path)?;
            return Ok(parse_spec_items(&source)
                .into_iter()
                .map(|item| {
                    SpecItemMapping {
                        target_range: None,
                        item,
                    }
                })
                .collect());
        }

        let lang_name = match extension {
            "rs" => "rust",
            "py" => "python",
            "js" => "javascript",
            "ts" => "typescript",
            _ => return Err(anyhow!("Unsupported language extension: {}", extension)),
        };

        let lang = get_language(lang_name)
            .ok_or_else(|| anyhow!("Language {} not enabled in arborium", lang_name))?;

        let source = std::fs::read_to_string(path)?;
        let mut parser = tree_sitter::Parser::new();
        parser.set_language(&lang)?;

        let tree = parser
            .parse(&source, None)
            .ok_or_else(|| anyhow!("Failed to parse file {:?}", path))?;

        let mut mappings = Vec::new();
        self.traverse_and_map(tree.root_node(), &source, &mut mappings);

        Ok(mappings)
    }

    fn traverse_and_map<'a>(
        &self,
        node: tree_sitter::Node<'a>,
        source: &str,
        mappings: &mut Vec<SpecItemMapping>,
    ) {
        if matches!(node.kind(), "comment" | "line_comment" | "block_comment") {
            let text = &source[node.byte_range()];
            if let Some(annotation) = self.extract_annotation(text) {
                // Find the next significant sibling
                let target_range = self
                    .find_next_significant_node(node)
                    .map(|target| target.byte_range());

                let item = match annotation {
                    Annotation::Contract(spec) => {
                        let mut parser = Parser::new(&spec);
                        SpecItem::FunctionContract(parser.parse_function_contract())
                    }
                    Annotation::TypeAlias(spec) => {
                        let mut parser = Parser::new(&spec);
                        SpecItem::TypeAlias(parser.parse_type_alias())
                    }
                    Annotation::Assert(spec) => {
                        let mut parser = Parser::new(&spec);
                        SpecItem::Assertion(parser.parse_assertion())
                    }
                    Annotation::Other => return,
                };

                mappings.push(SpecItemMapping { target_range, item });
            }
        }

        let mut cursor = node.walk();
        if cursor.goto_first_child() {
            loop {
                self.traverse_and_map(cursor.node(), source, mappings);
                if !cursor.goto_next_sibling() {
                    break;
                }
            }
        }
    }

    fn extract_annotation(&self, comment: &str) -> Option<Annotation> {
        // Look for // l[...] or /* l[...] */
        let trimmed = comment.trim();

        if trimmed.starts_with("// l[") && trimmed.ends_with("]") {
            Some(Self::parse_annotation_payload(
                &trimmed[5..trimmed.len() - 1],
            ))
        } else if trimmed.starts_with("/* l[") && trimmed.ends_with("] */") {
            Some(Self::parse_annotation_payload(
                &trimmed[5..trimmed.len() - 3],
            ))
        } else {
            None
        }
    }

    fn parse_annotation_payload(payload: &str) -> Annotation {
        let payload = payload.trim();
        if payload.starts_with("fn ") {
            return Annotation::Contract(payload.to_string());
        }

        if payload.starts_with("type ") {
            return Annotation::TypeAlias(payload.to_string());
        }

        if payload.starts_with("@assert") {
            return Annotation::Assert(payload.to_string());
        }

        Annotation::Other
    }

    fn find_next_significant_node<'a>(
        &self,
        node: tree_sitter::Node<'a>,
    ) -> Option<tree_sitter::Node<'a>> {
        let mut current = node;
        while let Some(sibling) = current.next_sibling() {
            if sibling.kind() != "comment" && sibling.is_named() {
                return Some(sibling);
            }
            current = sibling;
        }

        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_mapping_rust() {
        let mapper = StructuralMapper::new();
        let path = Path::new("tests/fixtures/simple.rs");
        let mappings = mapper.map_file(path).unwrap();

        assert_eq!(mappings.len(), 2);
        assert_eq!(mappings[0].contract.name, "add");
        assert_eq!(mappings[1].contract.name, "sub");
    }

    #[test]
    fn test_mapping_rust_contract_payload() {
        let mapper = StructuralMapper::new();
        let path = Path::new("tests/fixtures/contract_prefix.rs");
        let mappings = mapper.map_file(path).unwrap();

        assert_eq!(mappings.len(), 1);
        assert_eq!(mappings[0].contract.name, "mul");
    }

    #[test]
    fn test_parse_fn_annotation_payload() {
        match StructuralMapper::parse_annotation_payload("fn add(x: Int) -> Int") {
            Annotation::Contract(spec) => assert!(spec.starts_with("fn add")),
            Annotation::TypeAlias(_) | Annotation::Assert(_) | Annotation::Other => {
                panic!("expected contract annotation")
            }
        }
    }

    #[test]
    fn test_parse_type_annotation_payload() {
        match StructuralMapper::parse_annotation_payload("type Nat = { v: Int | v >= 0 }") {
            Annotation::Contract(_) => panic!("expected type annotation"),
            Annotation::TypeAlias(spec) => assert_eq!(spec, "type Nat = { v: Int | v >= 0 }"),
            Annotation::Assert(_) | Annotation::Other => panic!("expected type annotation"),
        }
    }

    #[test]
    fn test_parse_assert_annotation_payload() {
        match StructuralMapper::parse_annotation_payload("@assert x > 0") {
            Annotation::Assert(spec) => assert_eq!(spec, "@assert x > 0"),
            Annotation::Contract(_) | Annotation::TypeAlias(_) | Annotation::Other => {
                panic!("expected assert annotation")
            }
        }
    }

    #[test]
    fn test_legacy_prefix_payloads_are_ignored() {
        assert!(matches!(
            StructuralMapper::parse_annotation_payload("contract: fn add(x: Int) -> Int"),
            Annotation::Other
        ));
        assert!(matches!(
            StructuralMapper::parse_annotation_payload("type: Nat = { v: Int | v >= 0 }"),
            Annotation::Other
        ));
        assert!(matches!(
            StructuralMapper::parse_annotation_payload("assert: x > 0"),
            Annotation::Other
        ));
    }

    #[test]
    fn test_map_file_items_includes_type_alias() {
        let mapper = StructuralMapper::new();
        let path = Path::new("tests/fixtures/contract_prefix.rs");
        let mappings = mapper.map_file_items(path).unwrap();

        assert_eq!(mappings.len(), 2);
        match &mappings[1].item {
            SpecItem::TypeAlias(alias) => assert_eq!(alias.name, "Nat"),
            _ => panic!("expected type alias item"),
        }
    }

    #[test]
    fn test_map_file_items_lmt_program() {
        let mapper = StructuralMapper::new();
        let path = Path::new("tests/fixtures/simple.lmt");
        let mappings = mapper.map_file_items(path).unwrap();

        assert_eq!(mappings.len(), 3);
        assert!(
            mappings
                .iter()
                .all(|mapping| mapping.target_range.is_none())
        );
        assert!(matches!(mappings[0].item, SpecItem::TypeAlias(_)));
        assert!(matches!(mappings[1].item, SpecItem::FunctionContract(_)));
        assert!(matches!(mappings[2].item, SpecItem::Assertion(_)));
    }
}
