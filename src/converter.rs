use std::{fs, path::PathBuf};

use indoc::formatdoc;

use crate::hdl_info::{HdlInfo, Module, Port};

#[derive(Default)]
pub struct ChiselConverter {
    split_bundle: bool
}

impl ChiselConverter {
    pub fn builder() -> Self {
        Self::default()
    }

    pub fn split_bundle(mut self) -> Self {
        self.split_bundle = true;
        self
    }

    pub fn emit_chisel_string(&self, hdl_info: &HdlInfo) -> Vec<String> {
        hdl_info.get_modules().iter().map(|m| {
            module_to_chisel(m, self.split_bundle)
        }).collect::<Vec<String>>()
    }

    pub fn emit_chisel(&self, path: &PathBuf, hdl_info: &HdlInfo) {
        hdl_info.get_modules().iter().for_each(|m| {
            write_to_file(&path, &m.get_name(), &module_to_chisel(m, self.split_bundle), "scala");
        });
    }
}

fn write_to_file(path: &PathBuf, name: &str, contents: &str, suffix: &str) {
    if let Err(e) = fs::create_dir_all(&path) {
        eprintln!("Failed to create write file path [{}]: {}", path.display(), e);
    }
    if let Err(e) = fs::write(path.join(format!("{}.{}", name, suffix)), contents) {
        eprintln!("Failed to write {} to file [{}]: {}", name.to_string(), path.display(), e);
    }
}

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

fn port_to_chisel(port: &Port) -> String {
    format!("val {} = {}(UInt({}.W))", port.get_name(), port.get_direction(), port.get_width())
}

fn module_to_chisel(module: &Module, split_bundle: bool) -> String {
    let ports = module.get_ports().iter().map(|p| port_to_chisel(p)).collect::<Vec<String>>().join("\n");
    let ports = indent_block(&ports, 8);
    if split_bundle {
        let bundle_name = module.get_name() + "_Bundle";
        formatdoc! {"
            import chisel3._

            class {} extends Bundle {{
            {}
            }}

            class {} extends BlackBox {{
                val io = IO(new {})
            }}
        ", bundle_name, ports, module.get_name(), bundle_name}
    } else {
        formatdoc! {"
            import chisel3._

            class {} extends BlackBox {{
                val io = IO(new Bundle {{
            {}
                }})
            }}
        ", module.get_name(), ports}
    }
}
