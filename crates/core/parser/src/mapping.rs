use std::path::Path;

use anyhow::{Result, anyhow};
use arborium::{get_language, tree_sitter};
use facet::Facet;

use crate::{
    ast::{FunctionContract, SpecItem},
    parser::{Parser, parse_spec_items},
};



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
        if node_is_comment(&node) {
            let text = &source[node.byte_range()];
            if let Some(item) = self.extract_annotation(text) {
                // Find the next significant sibling
                let target_range = self
                    .find_next_significant_node(node)
                    .map(|target| target.byte_range());

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

    fn extract_annotation(&self, comment: &str) -> Option<SpecItem> {
        // Look for // @[...] or /* @[...] */
        let trimmed = comment.trim();

        let payload = if trimmed.starts_with("// @[") && trimmed.ends_with("]") {
            &trimmed[5..trimmed.len() - 1]
        } else if trimmed.starts_with("/* @[") && trimmed.ends_with("] */") {
            &trimmed[5..trimmed.len() - 3]
        } else {
            return None;
        };

        let mut parser = Parser::new(payload);
        Some(parser.parse_spec_item())
    }

    fn find_next_significant_node<'a>(
        &self,
        node: tree_sitter::Node<'a>,
    ) -> Option<tree_sitter::Node<'a>> {
        let mut current = node;
        while let Some(sibling) = current.next_sibling() {
            if !node_is_comment(&sibling) && sibling.is_named() {
                return Some(sibling);
            }
            current = sibling;
        }

        None
    }
}

fn node_is_comment(node: &tree_sitter::Node) -> bool {
    matches!(node.kind(), "comment" | "line_comment" | "block_comment")
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
    fn test_extract_contract() {
        let mapper = StructuralMapper::new();
        match mapper.extract_annotation("// @[let add(x: Int): Int]").unwrap() {
            SpecItem::FunctionContract(c) => assert_eq!(c.name, "add"),
            _ => panic!("expected contract"),
        }
    }

    #[test]
    fn test_extract_type_alias() {
        let mapper = StructuralMapper::new();
        match mapper.extract_annotation("// @[let Nat = Int | it >= 0]").unwrap() {
            SpecItem::TypeAlias(a) => assert_eq!(a.name, "Nat"),
            _ => panic!("expected type alias"),
        }
    }

    #[test]
    fn test_extract_assert() {
        let mapper = StructuralMapper::new();
        match mapper.extract_annotation("// @[@assert x > 0]").unwrap() {
            SpecItem::Assertion(_) => {},
            _ => panic!("expected assertion"),
        }
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
