use std::io::Write;
use crate::dbus_common::make_output_writer;
use convert_case::{Case, Casing};

static DBUS_VERSION: &str = "0.9.7";

pub struct CrateFiles
{
    project_name : String,
    output_dir : std::path::PathBuf
}

impl CrateFiles
{
    pub fn new(project_name: &str,
        output_dir : std::path::PathBuf) -> Self
    {
        CrateFiles { project_name : project_name.to_string().to_case(Case::Snake), output_dir }
    }

    pub fn generate(&self) -> std::io::Result<()>{
        let mut output_cargo_writer = make_output_writer(&self.output_dir, "Cargo.toml")?;

        write!(output_cargo_writer, r##"
[package]
name = "{}"
version = "0.1.0"
edition = "2021"

[dependencies]
dbus = "{}"
enum_primitive = "*"
"##, self.project_name, DBUS_VERSION)?;

        Ok(())
    }
}
