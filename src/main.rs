extern crate xmltree;

#[macro_use]
extern crate lazy_static;

mod crate_files;
mod dbus_common;
mod dbus_enum;
mod dbus_error;
mod dbus_interface;
mod dbus_signal;
mod dbus_services;
mod dbus_struct;

use crate_files::CrateFiles;
use dbus_common::{CodeGenerator, make_lib_output_writer, Args, EXT_BUSNAME};
use dbus_enum::DbusEnum;
use dbus_interface::DbusInterface;
use dbus_services::DbusServices;
use dbus_struct::DbusStruct;

use clap::Parser;
use std::fs::File;
use std::path::PathBuf;
use xmltree::Element;
use std::collections::{HashMap, HashSet};
use std::rc::Rc;
use convert_case::{Case, Casing};

fn make_generator(root_element: &mut Element, services : &DbusServices) -> Option<Rc<dyn CodeGenerator>>
{
    if let Some(enum_element) = root_element.get_mut_child("enum")
    {
        // Enums
        return Some(Rc::new(DbusEnum::new(enum_element).unwrap()));
    }
    else if let Some(struct_element) = root_element.get_mut_child("struct")
    {
        // Structs
        return Some(Rc::new(DbusStruct::new(struct_element).unwrap()));
    }
    else if let Some(interface_element) = root_element.get_mut_child("interface")
    {
        return Some(Rc::new(DbusInterface::new(interface_element, services).unwrap()));
    }
    else
    {
        return None;
    }
}

fn make_output_dir_name(output_dir_name: &std::path::PathBuf,
    project_name: &str) -> std::path::PathBuf
{
    let mut output_dir_name = output_dir_name.clone();
    output_dir_name.push(project_name.to_case(Case::Snake));
    output_dir_name
}

fn open_and_parse(file_name : &PathBuf) -> Element
{
    let input_file = File::open(file_name).unwrap();
    Element::parse(input_file).unwrap()
}

fn main() {

    let args = Args::parse();

    let mut services : Option<DbusServices> = Option::None;
    let mut client_libs = HashSet::new();
    let mut output_writers = HashMap::new();
    let mut error_types : HashMap<String, Rc<dyn CodeGenerator>> = HashMap::new();
    let mut input_xmls = Vec::new();

    for file_name in args.input_files
    {
        let mut root_element = open_and_parse(&file_name);

        // Annoyingly, we first have to parse all the input XMLs to find the ones
        // containing the service infomation (which can have any name)
        if let Some(bus_name) = root_element.attributes.get_mut(&EXT_BUSNAME).cloned()
        {
            if services.is_none()
            {
                services = Some(DbusServices::new(bus_name.as_str()));
            }

            let mut s = services.unwrap();
            s.parse_services(&mut root_element);
            services = Some(s);
        }
        else
        {
            input_xmls.push(root_element);
        }
    }

    if services.is_none()
    {
        panic!("No service info found in input XMLs");
    }

    for mut root_element in input_xmls
    {
        match make_generator(&mut root_element, services.as_ref().unwrap())
        {
            Some(g) => {
                let project_name = g.project_name().clone();
                client_libs.insert(project_name);
                let mut output_src_dir = make_output_dir_name(&args.output_dir, g.project_name());

                output_src_dir.push("src");
                let output_writer
                    = output_writers
                    .entry(output_src_dir.clone())
                    .or_insert_with(|| make_lib_output_writer(&output_src_dir, "lib.rs").unwrap());

                g.generate(output_writer).unwrap();
                for error_generator in g.error_types()
                {
                    error_types.insert(error_generator.name().clone(), error_generator);
                }
            },
            None => println!("Unhandled element type")
        }
    }

    for error in error_types.values()
    {
        let mut output_src_dir = make_output_dir_name(&args.output_dir,
            error.project_name());
        output_src_dir.push("src");

        let output_writer
            = output_writers
            .entry(output_src_dir.clone())
            .or_insert_with(|| make_lib_output_writer(&output_src_dir, "lib.rs").unwrap());

        error.generate(output_writer).unwrap();
    }

    // Generate the Cargo package file
    for name in client_libs
    {
        let output_dir = make_output_dir_name(&args.output_dir, &name);
        let cargo_file = CrateFiles::new(&name, output_dir);
        cargo_file.generate().unwrap();
    }

}
