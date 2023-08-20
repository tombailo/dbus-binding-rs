extern crate xmltree;

use genco::prelude::*;
use std::io::BufWriter;
use xmltree::Element;
use std::fs::File;
use std::io::Write;

use crate::dbus_common::*;

pub struct DbusEnum {

    name : String,
    project_name : String,
    names : Vec<String>,
    values : Vec<String>,
}

impl DbusEnum {

    pub fn new(elem : &mut Element) -> Result<DbusEnum, ()> {
        if let Some(name) = elem.attributes.get(&NAME_ATTRIBUTE)
        {
            let name_str = name.to_string();
            let tokens : Vec<&str> = name_str.rsplit(".").collect();
            let mut names : Vec<String> = Vec::new();
            let mut values : Vec<String> = Vec::new();

            while let Some(child) = elem.take_child("enumvalue")
            {
                names.push(child.attributes.get(&NAME_ATTRIBUTE).unwrap().to_string());
                values.push(child.attributes.get(&VALUE_ATTRIBUTE).unwrap().to_string());
            }

            Ok(DbusEnum { name : tokens[0].to_string(),
                project_name : tokens[1].to_string(),
                names,
                values})
        }
        else
        {
            Err(())
        }
    }

}

impl CodeGenerator for DbusEnum {

    fn generate(&self, output_writer : &mut BufWriter<File>) -> std::io::Result<()>{

        let enum_name = &self.name;
        let enum_names = &self.names;
        let enum_values = &self.values;

        // See https://docs.rs/genco/0.17.2/genco/macro.quote.html
        let generated_code : rust::Tokens = quote! {

            enum_from_primitive! {
            #[derive(Clone, Copy, Debug)]
            pub enum $enum_name {
                $(for (n, v) in enum_names.into_iter().zip(enum_values) join(, ) => $['\r']$n = $v)
            }
            }

            impl $enum_name
            {
                pub fn new(val : i32) -> Self
                {
                    // TODO The .unwrap() will terminate the process if we get invalid enum values
                    // from DBus :-S
                    $enum_name::from_i32(val).unwrap()
                }
            }


            impl dbus::arg::ReadAll for $enum_name {
                fn read(i: &mut dbus::arg::Iter) -> Result<Self, dbus::arg::TypeMismatchError> {
                    Ok($enum_name::from_i32(i.read().unwrap()).unwrap())
                }
            }

            impl<'a> dbus::arg::Get<'a> for $enum_name {
                fn get(i: &mut dbus::arg::Iter<'a>) -> Option<Self> {
                    match i.get()
                    {
                        Some(value) => $enum_name::from_i32(value),
                        None => None
                    }
                }
            }

            impl dbus::arg::Append for $enum_name {
                fn append_by_ref(&self, i: &mut IterAppend<'_>) {
                    i.append(*self as i32);
                }

                fn append(self, i: &mut IterAppend<'_>) {
                    i.append(self as i32);
                }
            }
            
            impl dbus::arg::Arg for $enum_name {
                const ARG_TYPE : ArgType = ArgType::Int32;

                fn signature() -> Signature<'static> {
                    <i32 as dbus::arg::Arg>::signature()
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
