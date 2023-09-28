extern crate xmltree;

use clap::Parser;
use std::{fs, io::BufWriter};
use std::rc::Rc;
use std::io::Write;
use genco::prelude::*;
use xmltree::{AttributeName, Element};

const RUST_KEYWORDS: [&str; 57] = [
    "as",
    "break",
    "const",
    "continue",
    "crate",
    "dyn",
    "else",
    "enum",
    "extern",
    "false",
    "fn",
    "for",
    "if",
    "impl",
    "in",
    "let",
    "loop",
    "match",
    "mod",
    "move",
    "mut",
    "pub",
    "ref",
    "return",
    "Self",
    "self",
    "static",
    "struct",
    "super",
    "trait",
    "true",
    "type",
    "union",
    "unsafe",
    "use",
    "where",
    "while",

    "abstract",
    "alignof",
    "async",
    "await",
    "become",
    "box",
    "do",
    "final",
    "macro",
    "offsetof",
    "override",
    "priv",
    "proc",
    "pure",
    "sizeof",
    "try",
    "typeof",
    "unsized",
    "virtual",
    "yield",
];

#[derive(Parser, Debug)]
pub struct Args {
    #[arg(short, long)]
    pub output_dir: std::path::PathBuf,

    #[arg(long, default_value = "ext")]
    pub ext_prefix: String,

    #[arg(long, default_value = "http://extensions.somewhere.com/schemas/dbus-extensions-v1.0")]
    pub ext_namespace: String,

    pub input_files: Vec<std::path::PathBuf>
}

fn get_ext_prefix_override() -> String
{
    let args = Args::parse();

    args.ext_prefix
}

fn get_ext_namespace_override() -> String
{
    let args = Args::parse();

    args.ext_namespace
}

lazy_static! {
    pub static ref EXT_TYPE_ATTRIBUTE : AttributeName = AttributeName{local_name : "type".to_string(),
     prefix : Some(get_ext_prefix_override()),
     namespace : Some(get_ext_namespace_override())};

    pub static ref TYPE_ATTRIBUTE : AttributeName = AttributeName{local_name : "type".to_string(),
     prefix : None,
     namespace : None};

    pub static ref NAME_ATTRIBUTE : AttributeName = AttributeName{local_name : "name".to_string(),
     prefix : None,
     namespace : None};

    pub static ref DIRECTION_ATTRIBUTE : AttributeName = AttributeName{local_name : "direction".to_string(),
     prefix : None,
     namespace : None};

    pub static ref VALUE_ATTRIBUTE : AttributeName = AttributeName{local_name : "value".to_string(),
     prefix : None,
     namespace : None};

    pub static ref EXT_BUSNAME : AttributeName = AttributeName{local_name : "busname".to_string(),
     prefix : Some(get_ext_prefix_override()),
     namespace : Some(get_ext_namespace_override())};

}

/// Trait that the different kinds of code generator must implement
/// allowing us to get common info about them.
pub trait CodeGenerator {

    fn name(&self) -> &String;

    fn project_name(&self) -> &String;

    fn generate(&self, output_writer : &mut BufWriter<fs::File>) -> std::io::Result<()>;

    fn error_types(&self) -> Vec<Rc<dyn CodeGenerator>>;
}

pub fn dbus_type_2_rust_type (dbus_type : &str)-> String
{
    if dbus_type == "s"
    {
        return "String".to_string();
    }
    else if dbus_type == "u"
    {
        return "u32".to_string();
    }
    else if dbus_type == "i"
    {
        return "i32".to_string();
    }
    else if dbus_type == "b"
    {
        return "bool".to_string();
    }
    else if dbus_type == "x"
    {
        return "i64".to_string();
    }
    else if dbus_type == "d"
    {
        return "f64".to_string();
    }
    else if dbus_type == "t"
    {
        return "u64".to_string();
    }
    else if dbus_type == "y"
    {
        return "u8".to_string();
    }
    else if dbus_type == "n"
    {
        return "i16".to_string();
    }
    else if dbus_type == "q"
    {
        return "u16".to_string();
    }
    else if dbus_type.starts_with("(")
    {
        // This is a Dbus struct, represented as a tuple in Rust.
        // Structs can be nested inside each other
        let mut nesting_level = 0;
        let mut rust_type = "(".to_string();
        for c in dbus_type[1..].chars()
        {
            let mut b = [0; 2];
            if c == '('
            {
                nesting_level += 1;
                rust_type.push('(');
                continue;
            }
            if c == ')'
            {
                nesting_level -= 1;
                rust_type.push(')');
                if nesting_level == 0
                {
                    return rust_type;
                }
                else
                {
                    continue;
                }
            }

            rust_type += dbus_type_2_rust_type(c.encode_utf8(&mut b)).as_str();
            rust_type.push(',');
        }
        return rust_type;
    }
    else if dbus_type.starts_with("a{")
    {
        // This is a dict
        return "HashMap<".to_string()
            + &dbus_type_2_rust_type(&dbus_type[2..dbus_type.len()-2])
            + ", "
            + &dbus_type_2_rust_type(&dbus_type[3..dbus_type.len()-1])
            + ">";
    }
    else if dbus_type.starts_with('a')
    {
        // This is an array
        return "Vec<".to_string() + &dbus_type_2_rust_type(&dbus_type[1..]) + ">";
    }
    else
    {
        panic!("Unknown DBus type: {}", dbus_type);
    }
}

pub fn make_output_writer(path : &std::path::PathBuf, file : &str) -> std::io::Result<BufWriter<fs::File>>
{
    fs::create_dir_all(path)?;
    let mut output_file_path = path.clone();
    output_file_path.push(file.clone());
    if output_file_path.extension().is_none() {
        output_file_path.set_extension("rs");
    }
    let output_file = fs::OpenOptions::new()
        .write(true)
        .truncate(true)
        .create(true)
        .open(output_file_path)?;
    let output_writer = BufWriter::new(output_file);

    Ok(output_writer)
}


pub fn make_lib_output_writer(path : &std::path::PathBuf, file : &str) -> std::io::Result<BufWriter<fs::File>>
{
    let mut output_writer = make_output_writer(path, file)?;
 
    // Write out pragmas and use directives that must be at
    // the top of the file
    let generated_code : rust::Tokens = quote! {
        #![allow(non_camel_case_types)]$['\r']
        #![allow(non_snake_case)]

        extern crate dbus;
        #[macro_use] extern crate enum_primitive;

        use crate::enum_primitive::FromPrimitive;
        use dbus::arg::*;
        use dbus::Signature;
        use dbus::blocking::Connection;
        use dbus::blocking::Proxy;
        use dbus::message::Message;
        use dbus::strings::{Path, BusName};
        use std::fmt;
        use std::time::Duration;
        use std::collections::HashMap;
        use std::ops::Drop;

        static DEFAULT_TIMEOUT : Duration = Duration::from_millis(5000);

        // Define a polymorphic error type
        type Error = Box<dyn std::error::Error>;


    };
    let generated_string = generated_code.to_file_string().unwrap();

    output_writer.write_all(generated_string.as_bytes())?;

    Ok(output_writer)
}

pub fn remove_square_brackets(ext_type : &str) -> &str
{
    if ext_type.starts_with('[')
    {
        return &ext_type[1..(ext_type.len() - 1)];
    }
    else {
        return ext_type;
    }
}

#[derive(Clone, Debug)]
pub struct DbusType
{
    pub type_name : String,
    pub contained_types : Vec<DbusType>,
    pub is_returned_object : bool
}

impl DbusType {

    pub fn get_type_decl(&self) -> String
    {
        let mut type_decl = self.type_name.clone();

        if !self.contained_types.is_empty()
        {
            type_decl += "<";
            let mut i = 0;
            while i < self.contained_types.len()
            {
                type_decl += &self.contained_types[i].get_type_decl();
                if i < self.contained_types.len() - 1
                {
                    type_decl += ", ";
                }
                i += 1;
            }
            type_decl += ">";
        }
        type_decl
    }
}

pub struct DbusMethodArg {
    pub name : String,
    pub arg_type : DbusType
}

impl DbusMethodArg {

    pub fn get_arg_declaration(&self) -> rust::Tokens
    {
        quote!{ $(&self.name) : &$(&self.arg_type.type_name) }
    }
}

pub fn get_dbus_type(elem : &Element) -> DbusType
{
    if let Some(ext_type) = elem.attributes.get(&EXT_TYPE_ATTRIBUTE)
    {
        let is_returned_object = elem.attributes.get(&TYPE_ATTRIBUTE).unwrap() == "o";
        // This is a map
        if ext_type.starts_with("a{")
        {
            let mut contained_types = Vec::new();
            let mut i = 2;

            while i < ext_type.len()
            {
                if let Some(c) = ext_type.chars().nth(i)
                {
                    if c == '['
                    {
                        if let Some(mut close_bracket) = ext_type.as_str()[i..].find(']')
                        {
                            close_bracket += i;
                            i += 1;
                            contained_types.push(DbusType { type_name: ext_type.as_str()[i..close_bracket].to_string(),
                                contained_types: Vec::new(),
                                is_returned_object
                            });
                            i = close_bracket;
                        }
                    }
                    else if c != '}'
                    {
                        contained_types.push(DbusType { type_name: dbus_type_2_rust_type(&String::from(c)),
                            contained_types: Vec::new(),
                            is_returned_object
                        });
                    }
                }
                i += 1;
            }

            return DbusType{ type_name: "HashMap".to_string(),
                             contained_types,
                             is_returned_object};
        }
        else if ext_type.starts_with("a")
        {
            // This is an array
            return DbusType{ type_name: "Vec".to_string(),
                contained_types: vec![DbusType{
                    type_name: remove_square_brackets(&ext_type[1..]).to_string(),
                    contained_types: Vec::new(),
                is_returned_object}],
                is_returned_object
            };
        }
        // This is a regular extension type
        DbusType{ type_name: remove_square_brackets(ext_type).to_string(), contained_types: Vec::new(), is_returned_object }
    }
    else
    {
        DbusType{
            type_name: dbus_type_2_rust_type(elem.attributes.get(&TYPE_ATTRIBUTE).unwrap()).to_string(),
            contained_types: Vec::new(),
            is_returned_object: false
        }
    }

}

pub fn prefix_keywords(identifier : &str) -> String
{
    let mut r = identifier.to_string();
    if RUST_KEYWORDS.iter().any(|i| i == &r) { r.push('_') };
    r
}
