
use std::collections::HashMap;
use std::path::PathBuf;
use sv_parser::{parse_sv, unwrap_node, SyntaxTree, Locate, RefNode};

#[derive(Debug)]
pub struct Port {
    name: String,
    direction: String,
    port_type: String,
    width: String,
}

#[derive(Debug)]
pub struct Define {
    name: String,
    value: String
}

#[derive(Debug)]
pub struct Module {
    name: String,
    ports: Vec<Port>,
    defines: Vec<Define>
}

pub fn parse_module(
    syntax_tree: &SyntaxTree,
    path: &PathBuf
) -> Result<Module, std::io::Error> {
    let mut module: Module = Module {
        name: "".to_string(),
        ports: Vec::new(),
        defines: Vec::new()
    };

    for node in syntax_tree {
        match node {
            RefNode::TextMacroDefinition(x) => {
                if let Some(start) = unwrap_node!(x, TextMacroDefinition) {
                    let name = if let Some(name) = unwrap_node!(x, TextMacroName) {
                        let name = get_identifier(name).unwrap();
                        syntax_tree.get_str(&name).unwrap()
                    } else {
                        "unknown"
                    };

                    let value = if let Some(RefNode::MacroText(x)) = unwrap_node!(x, MacroText) {
                        let replacement = x.nodes.0;
                        syntax_tree.get_str(&replacement).unwrap()
                    } else {
                        "unknown"
                    };

                    module.defines.push(Define { name: name.to_string(), value: value.to_string() });
                }
            }
            RefNode::ModuleDeclaration(x) => {
                let id = unwrap_node!(x, ModuleIdentifier).unwrap();
                let id = get_identifier(id).unwrap();
                let name = syntax_tree.get_str(&id).unwrap();

                module.name = name.to_string();
            }
            // TODO: parse port expression
            RefNode::PortDeclaration(x) => {
            }
            RefNode::AnsiPortDeclaration(x) => {

            }
            // Can add process of comment, parameter, instantiation and keyword
            _ =>  ()
        }
    }

    Ok(module)
}
      
pub fn get_identifier(node: RefNode) -> Option<Locate> {
    // unwrap_node! can take multiple types
    match unwrap_node!(node, SimpleIdentifier, EscapedIdentifier, Keyword) {
        Some(RefNode::SimpleIdentifier(x)) => {
            return Some(x.nodes.0);
        }
        Some(RefNode::EscapedIdentifier(x)) => {
            return Some(x.nodes.0);
        }
        Some(RefNode::Keyword(x)) => {
            return Some(x.nodes.0);
        }
        _ => None,
    }
}

pub fn parse_file(path: &PathBuf) -> Result<Module, std::io::Error> {
    let defines = HashMap::new();
    let includes: Vec<PathBuf> = Vec::new();

    // Parse
    let result = parse_sv(&path, &defines, &includes, false, false);
    if let Ok((syntax_tree, _)) = result {
        parse_module(&syntax_tree, path)
    } else {
        Err(std::io::Error::new(std::io::ErrorKind::Other, "parse_file failed"))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    #[test]
    fn parse_basic_module_file() {
        let path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("tests")
            .join("sv")
            .join("basic_module.sv");

        let module = parse_file(&path).expect("parse_file failed");

        // 基本正确性检查：模块名和端口数量
        assert_eq!(module.name, "basic_module");
        println!("Module: {:#?}", module);
        // assert!(!module.ports.is_empty());
    }
}

