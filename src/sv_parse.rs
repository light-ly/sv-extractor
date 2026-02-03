
use std::collections::HashMap;
use std::path::PathBuf;
use lazy_static::lazy_static;
use rhai::{Engine, Scope};
use regex::{Regex, Captures};
use sv_parser::{Iter, Locate, Node, RefNode, SyntaxTree, parse_sv, unwrap_node};

use crate::hdl_info::HdlInfo;

pub fn parse_module(syntax_tree: &SyntaxTree) -> Result<HdlInfo, std::io::Error> {
    let mut hdl_info: HdlInfo = HdlInfo::new();

    let mut ansi_port_last_dir = "";
    let mut define_map: HashMap<String, String> = HashMap::new();

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

                    hdl_info.add_define(name, value);
                    define_map.insert(name.to_string(), value.to_string());
                }
            }
            RefNode::ModuleDeclaration(x) => {
                let id = unwrap_node!(x, ModuleIdentifier).unwrap();
                let id = get_identifier(id).unwrap();
                let name = syntax_tree.get_str(&id).unwrap();

                hdl_info.add_module(name);
            }
            RefNode::PortDeclaration(x) => {
                if let Some(id) = unwrap_node!(x, InputDeclaration, OutputDeclaration, InoutDeclaration) {
                    let id = get_identifier(id).unwrap();
                    let direction = syntax_tree.get_str(&id).unwrap();

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

                    let (width, width_expression) = match unwrap_node!(x, PackedDimensionRange) {
                        Some(RefNode::PackedDimensionRange(x)) => {
                            parse_packed_dimension_range(syntax_tree, &x.clone(), &define_map)
                        }
                        _ => ("1".to_string(), None)
                    };

                    if let Some(RefNode::ListOfPortIdentifiers(x)) = unwrap_node!(x, ListOfPortIdentifiers) {
                        for node in x {
                            if let RefNode::PortIdentifier(x) = node {
                                let id = unwrap_node!(x, Identifier).unwrap();
                                let id = get_identifier(id).unwrap();
                                let name = syntax_tree.get_str(&id).unwrap();

                                hdl_info.add_ports(name, direction, port_type, width.as_str(), &width_expression);
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

                    let (width, width_expression) = match unwrap_node!(x, PackedDimensionRange) {
                        Some(RefNode::PackedDimensionRange(x)) => {
                            parse_packed_dimension_range(syntax_tree, &x.clone(), &define_map)
                        }
                        _ => ("1".to_string(), None)
                    };

                    hdl_info.add_ports(name, ansi_port_last_dir, port_type, width.as_str(), &width_expression);
                }
            }
            // Can add process of comment, parameter,instantiation and keyword
            _ =>  ()
        }
    }

    Ok(hdl_info)
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

fn parse_packed_dimension_range(
    syntax_tree: &SyntaxTree,
    x: &sv_parser::PackedDimensionRange,
    defines: &HashMap<String, String>,
) -> (String, Option<String>) {
    let (expr, _) = parse_expression(syntax_tree, x);
    if expr == "unknown" {
        return ("unknown".to_string(), None);
    }

    let width_bits = compute_packed_range_width_bits(&expr, defines)
        .map(|w| w.to_string())
        .unwrap_or_else(|| expr.clone());

    (width_bits, Some(expr))
}

fn compute_packed_range_width_bits(expr: &str, defines: &HashMap<String, String>) -> Option<i64> {
    // Expect forms like:
    // - "[MSB:LSB]"  => width = abs(MSB-LSB)+1
    // - "[N]"        => width = N+1? (SV single bit select is 1 bit; keep conservative and return None)
    let mut s = expr.trim();
    if !s.starts_with('[') || !s.ends_with(']') {
        return None;
    }
    s = &s[1..s.len() - 1];

    let mut parts = s.splitn(2, ':');
    let left = parts.next()?.trim();
    let right = parts.next().map(str::trim);

    // Only handle real ranges with ':'
    let right = right?;
    let msb = eval_int_expr(left, defines)?;
    let lsb = eval_int_expr(right, defines)?;

    Some((msb - lsb).abs() + 1)
}

fn create_sv_engine() -> Engine {
    let mut engine = Engine::new();

    engine.register_fn("clog2", |n: i64| {
        if n <= 1 {
            0i64
        } else {
            (n as f64).log2().ceil() as i64
        }
    });

    engine.register_fn("pow", |base: i64, exp: i64| {
        if exp < 0 { return 0i64; }
        base.pow(exp as u32)
    });

    engine
}

fn preprocess_for_rhai(input: &str) -> String {
    let mut s = parse_sv_number(input);
    s = s.replace("**", " `pow` ");
    s = s.replace("$clog2", "clog2");
    s
}

fn eval_int_expr(input: &str, defines: &HashMap<String, String>) -> Option<i64> {
    let engine = create_sv_engine();
    let mut scope = Scope::new();

    for (key, value) in defines {
        let clean_val = preprocess_for_rhai(value);
        if let Ok(v) = engine.eval_expression_with_scope::<i64>(&mut scope, &clean_val) {
            scope.push(key.clone(), v);
        }
    }

    let clean_input = preprocess_for_rhai(input);

    match engine.eval_expression_with_scope::<i64>(&mut scope, &clean_input) {
        Ok(res) => Some(res),
        Err(e) => {
            eprintln!("Evaluation error: {}", e);
            None
        }
    }
}

fn parse_sv_number(lit: &str) -> String {
    lazy_static! {
        static ref SV_NUM_RE: Regex = Regex::new(r"(?i)(\d+)?'([sdhbo])([0-9a-f_xz]+)").unwrap();
    }

    SV_NUM_RE.replace_all(lit, |caps: &Captures| {
        let base = caps[2].to_ascii_lowercase();
        let digits = caps[3].replace('_', "");

        let parsed_val = match base.as_str() {
            "h" => i64::from_str_radix(&digits, 16),
            "b" => i64::from_str_radix(&digits, 2),
            "o" => i64::from_str_radix(&digits, 8),
            "d" | "s" => digits.parse::<i64>(),
            _ => return "unknown".to_string(),
        };

        parsed_val.map(|v| v.to_string()).unwrap_or_else(|_| "unknown".to_string())
    }).to_string()
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

pub fn parse_file(path: &PathBuf) -> Result<HdlInfo, std::io::Error> {
    let defines = HashMap::new();
    let includes: Vec<PathBuf> = Vec::new();

    // Parse
    let result = parse_sv(&path, &defines, &includes, false, false);
    match result {
        Ok((syntax_tree, _)) => parse_module(&syntax_tree),
        Err(e) => Err(std::io::Error::new(std::io::ErrorKind::Other, format!("parse_file failed [{}]: {}", path.display(), e)))
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

        let hdl_info = parse_file(&path).expect("parse_file failed");

        // 基本正确性检查：模块名和端口数量
        assert!(!hdl_info.get_modules().is_empty());
        println!("Module: {:#?}", hdl_info);
    }
}

