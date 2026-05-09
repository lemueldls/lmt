use std::path::Path;

use anyhow::{Result, anyhow};
use arborium::{get_language, tree_sitter};
use facet::Facet;

use crate::{ast::FunctionContract, parser::Parser};

pub struct StructuralMapper {
    // We can add a GrammarStore here if needed for caching
}

#[derive(Debug, Clone, PartialEq, Facet)]
pub struct Mapping {
    pub target_range: std::ops::Range<usize>,
    pub contract: FunctionContract,
}

impl StructuralMapper {
    pub fn new() -> Self {
        Self {}
    }

    pub fn map_file(&self, path: &Path) -> Result<Vec<Mapping>> {
        let extension = path
            .extension()
            .and_then(|ext| ext.to_str())
            .ok_or_else(|| anyhow!("No extension for file {:?}", path))?;

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
        mappings: &mut Vec<Mapping>,
    ) {
        if matches!(node.kind(), "comment" | "line_comment" | "block_comment") {
            let text = &source[node.byte_range()];
            if let Some(annotation) = self.extract_annotation(text) {
                // Find the next significant sibling
                if let Some(target) = self.find_next_significant_node(node) {
                    let mut parser = Parser::new(annotation);
                    let contract = parser.parse_function_contract();

                    mappings.push(Mapping {
                        target_range: target.byte_range(),
                        contract,
                    });
                }
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

    fn extract_annotation<'a>(&self, comment: &'a str) -> Option<&'a str> {
        // Look for // l[...] or /* l[...] */
        let trimmed = comment.trim();

        if trimmed.starts_with("// l[") && trimmed.ends_with("]") {
            Some(&trimmed[5..trimmed.len() - 1])
        } else if trimmed.starts_with("/* l[") && trimmed.ends_with("] */") {
            Some(&trimmed[5..trimmed.len() - 3])
        } else {
            None
        }
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
}
