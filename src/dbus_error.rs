use genco::prelude::*;
use xmltree::Element;
use crate::dbus_common::*;
use std::io::BufWriter;
use std::fs::File;
use std::io::Write;

pub struct DbusError {
    name : String,
    project_name : String,
    dbus_name : String
}

impl DbusError {

    pub fn new(elem : &mut Element) -> Result<DbusError, ()>
    {
        if let Some(name) = elem.attributes.get(&NAME_ATTRIBUTE)
        {
            let name_str = name.to_string();
            let tokens : Vec<&str> = name_str.rsplit(".").collect();
            Ok(DbusError{ name : tokens[0].to_string(),
                project_name : tokens[2].to_string(),
                dbus_name : name_str
            })
        }
        else
        {
            Err(())
        }
    }
}

impl CodeGenerator for DbusError
{
    fn generate(&self, output_writer : &mut BufWriter<File>) -> std::io::Result<()> {
        let name = &self.name;
        let generated_code : rust::Tokens = quote! {

            #[derive(Debug, Clone)]
            pub struct $(name) {
                pub message : String
            }

            impl $(name) {
                pub const DBUS_NAME : &'static str = $(quoted (&self.dbus_name));
            }

            impl fmt::Display for $(name) {

                fn fmt(&self, f: &mut fmt::Formatter) -> Result<(), fmt::Error> {
                    write!(f, "{}", self.message)
                }
            }

            impl std::error::Error for $(name) {

                fn description(&self) -> &str {
                    &self.message
                }
            }
        };
        let generated_string = generated_code.to_file_string().unwrap();
        output_writer.write_all(generated_string.as_bytes())?;
        Ok(())
    }

    fn name(&self) -> &String
    {
        &self.name
    }

    fn project_name(&self) -> &String
    {
        &self.project_name
    }

    fn error_types(&self) -> Vec<std::rc::Rc<dyn CodeGenerator>> {
        Vec::new()
    }
}
