use serde::{Serialize, Deserialize};
use indoc::formatdoc;

#[allow(dead_code)]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Port {
    name: String,
    direction: String,
    port_type: String,
    width: String,
    width_expression: Option<String>,
}

#[allow(dead_code)]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Define {
    name: String,
    value: String
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Module {
    name: String,
    ports: Vec<Port>
}

#[derive(Debug, Serialize, Deserialize)]
pub struct HdlInfo {
    defines: Vec<Define>,
    modules: Vec<Module>
}

impl Port {
    fn to_chisel(&self) -> String {
        format!("val {} = {}(UInt({}.W))", self.name, self.direction, self.width)
    }
}

impl Module {
    fn indent_block(text: &str, indent_size: usize) -> String {
        let indent = " ".repeat(indent_size);
        text.lines()
            .map(|line| {
                if line.is_empty() {
                    line.to_string()
                } else {
                    format!("{}{}", indent, line)
                }
            })
            .collect::<Vec<String>>()
            .join("\n")
    }

    pub fn to_chisel(&self) -> String {
        let ports = self.ports.iter().map(|p| p.to_chisel()).collect::<Vec<String>>().join("\n");
        let ports = Self::indent_block(&ports, 8);
        formatdoc! {"
            import chisel3._

            class {} extends BlackBox {{
                val io = new Bundle {{
            {}
                }}
            }}
        ", self.name, ports}
    }

    pub fn get_name(&self) -> String {
        self.name.clone()
    }
}

impl HdlInfo {
    pub fn new() -> Self {
        HdlInfo {
            defines: Vec::new(),
            modules: Vec::new()
        }
    }

    pub fn add_module(&mut self, name: &str) {
        self.modules.push(Module { name: name.to_string(), ports: Vec::new() });
    }

    pub fn add_define(&mut self, name: &str, value: &str) {
        self.defines.push(Define { name: name.to_string(), value: value.to_string() });
    }

    pub fn add_ports(&mut self, name: &str, direction: &str, port_type: &str, width: &str, width_expression: &Option<String>) {
        if let Some(last_module) = self.modules.last_mut() {
            last_module.ports.push(Port {
                name: name.to_string(),
                direction: direction.to_string(),
                port_type: port_type.to_string(),
                width: width.to_string(),
                width_expression: width_expression.clone()
            });
        }
    }

    pub fn get_modules(&self) -> &Vec<Module> {
        &self.modules
    }

    pub fn merge_info(&mut self, info: &HdlInfo) {
        info.defines.iter().for_each(|d| self.defines.push(d.clone()));
        info.modules.iter().for_each(|m| self.modules.push(m.clone()));
    }
}
