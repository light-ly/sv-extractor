
use std::collections::HashMap;
use std::path::PathBuf;
use sv_parser::{Iter, Locate, Node, RefNode, SyntaxTree, parse_sv, unwrap_node};

#[allow(dead_code)]
#[derive(Debug)]
pub struct Port {
    name: String,
    direction: String,
    port_type: String,
    width: String,
}

#[allow(dead_code)]
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

pub fn parse_module(syntax_tree: &SyntaxTree) -> Result<Module, std::io::Error> {
    let mut module: Module = Module {
        name: "".to_string(),
        ports: Vec::new(),
        defines: Vec::new()
    };

    let mut ansi_port_last_dir = "";

    for node in syntax_tree {
        match node {
            RefNode::TextMacroDefinition(x) => {
                if let Some(_) = unwrap_node!(x, TextMacroDefinition) {
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
            RefNode::PortDeclaration(x) => {
                if let Some(id) = unwrap_node!(x, InputDeclaration, OutputDeclaration, InoutDeclaration) {
                    let id = get_identifier(id).unwrap();
                    let dir_type = syntax_tree.get_str(&id).unwrap();

                    let port_type = match unwrap_node!(x, DataType, ImplicitDataType) {
                        Some(RefNode::DataType(x)) => {
                            let id = unwrap_node!(x, Keyword);
                            if id != None {
                                syntax_tree.get_str(&get_identifier(id.unwrap()).unwrap()).unwrap()
                            } else {
                                "unknown"
                            }
                        },
                        Some(RefNode::ImplicitDataType(_)) => "wire",
                        _ => "unknown"
                    };

                    let width = match unwrap_node!(x, PackedDimensionRange) {
                        Some(RefNode::PackedDimensionRange(x)) => {
                            let (width, _) = parse_expression(&syntax_tree, &x.clone());
                            width
                        }
                        _ => "1".to_string()
                    };

                    if let Some(RefNode::ListOfPortIdentifiers(x)) = unwrap_node!(x, ListOfPortIdentifiers) {
                        for node in x {
                            if let RefNode::PortIdentifier(x) = node {
                                let id = unwrap_node!(x, Identifier).unwrap();
                                let id = get_identifier(id).unwrap();
                                let name = syntax_tree.get_str(&id).unwrap();

                                module.ports.push(Port { name: name.to_string(), direction: dir_type.to_string(), port_type: port_type.to_string(), width: width.clone() });
                            }
                        }
                    }
                }
            }
            RefNode::AnsiPortDeclaration(x) => {
                if let Some(id) = unwrap_node!(x, PortIdentifier) {
                    let name_locate = get_identifier(id).unwrap();
                    let name = syntax_tree.get_str(&name_locate).unwrap();

                    let id = unwrap_node!(x, PortDirection);
                    if id != None {
                        let id = id.unwrap();
                        let dir_locate = get_identifier(id).unwrap();
                        ansi_port_last_dir = syntax_tree.get_str(&dir_locate).unwrap();
                    };

                    let port_type = if unwrap_node!(x, AnsiPortDeclarationVariable) != None {
                        "wire"
                    } else {
                        match unwrap_node!(x, DataType, ImplicitDataType) {
                            Some(RefNode::DataType(x)) => {
                                let id = unwrap_node!(x, Keyword);
                                if id != None {
                                    syntax_tree.get_str(&get_identifier(id.unwrap()).unwrap()).unwrap()
                                } else {
                                    "unknown"
                                }
                            },
                            Some(RefNode::ImplicitDataType(_)) => "wire",
                            _ => "unknown"
                        }
                    };

                    let width = match unwrap_node!(x, PackedDimensionRange) {
                        Some(RefNode::PackedDimensionRange(x)) => {
                            let (width, _) = parse_expression(&syntax_tree, &x.clone());
                            width
                        }
                        _ => "1".to_string()
                    };

                    module.ports.push(Port { name: name.to_string(), direction: ansi_port_last_dir.to_string(), port_type: port_type.to_string(), width: width.clone() });
                }
            }
            // Can add process of comment, parameter,instantiation and keyword
            _ =>  ()
        }
    }

    Ok(module)
}

fn parse_expression<'a, N>(syntax_tree: &SyntaxTree, x: &'a N) -> (String, Option<Locate>)
where
    N: Node<'a>,
{
    let mut last_locate = Locate { offset: 0, line: 0, len: 0 };
    let mut expression = String::new();

    for node in Iter::new(x.next()) {
        // println!("parse expression::node {:#?}", node);
        match unwrap_node!(node, SimpleIdentifier, Symbol, UnsignedNumber, HexNumber, OctalNumber, BinaryNumber) {
            Some(RefNode::SimpleIdentifier(x)) => {
                let locate = x.nodes.0;
                if locate != last_locate {
                    last_locate = locate;
                    let s = syntax_tree.get_str(&locate).unwrap();
                    expression = expression + s;
                    // println!("parse expression {}", s);
                }
            }
            Some(RefNode::Symbol(x)) => {
                let locate = x.nodes.0;
                if locate != last_locate {
                    last_locate = locate;
                    let s = syntax_tree.get_str(&x.nodes.0).unwrap();
                    expression = expression + s;
                    // println!("parse expression {}", s);
                }
            }
            Some(RefNode::UnsignedNumber(x)) => {
                let locate = x.nodes.0;
                if locate != last_locate {
                    last_locate = locate;
                    let s = syntax_tree.get_str(&x.nodes.0).unwrap();
                    expression = expression + s;
                    // println!("parse expression {}", s);
                }
            }
            Some(RefNode::HexNumber(x)) => {
                let locate = x.nodes.1.nodes.0;
                if locate != last_locate {
                    last_locate = locate;
                    let size = if x.nodes.0 != None { syntax_tree.get_str(&x.nodes.0).unwrap() } else { "" };
                    let base = syntax_tree.get_str(&x.nodes.1.nodes.0).unwrap();
                    let number = syntax_tree.get_str(&x.nodes.2.nodes.0).unwrap();
                    expression = expression + size + base + number;
                    // println!("parse expression {}", expression);
                }
            }
            Some(RefNode::OctalNumber(x)) => {
                let locate = x.nodes.1.nodes.0;
                if locate != last_locate {
                    last_locate = locate;
                    let size = if x.nodes.0 != None { syntax_tree.get_str(&x.nodes.0).unwrap() } else { "" };
                    let base = syntax_tree.get_str(&x.nodes.1.nodes.0).unwrap();
                    let number = syntax_tree.get_str(&x.nodes.2.nodes.0).unwrap();
                    expression = expression + size + base + number;
                    // println!("parse expression {}", expression);
                }
            }
            Some(RefNode::BinaryNumber(x)) => {
                let locate = x.nodes.1.nodes.0;
                if locate != last_locate {
                    last_locate = locate;
                    let size = if x.nodes.0 != None { syntax_tree.get_str(&x.nodes.0).unwrap() } else { "" };
                    let base = syntax_tree.get_str(&x.nodes.1.nodes.0).unwrap();
                    let number = syntax_tree.get_str(&x.nodes.2.nodes.0).unwrap();
                    expression = expression + size + base + number;
                    // println!("parse expression {}", expression);
                }
            }
            _ => ()
        }
    }
    if expression == "" {
        ("unknown".to_string(), None)
    } else {
        // println!("parse function lastlocate {:?}", last_locate);
        (expression, Some(last_locate))
    }
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
        parse_module(&syntax_tree)
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

