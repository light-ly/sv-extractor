#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct Port {
    name: String,
    direction: String,
    port_type: String,
    width: String,
    width_expression: Option<String>,
}

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct Define {
    name: String,
    value: String
}

#[derive(Debug, Clone)]
pub struct Module {
    name: String,
    ports: Vec<Port>
}

#[derive(Debug)]
pub struct HdlInfo {
    defines: Vec<Define>,
    modules: Vec<Module>
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
