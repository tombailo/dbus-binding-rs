use genco::prelude::*;
use xmltree::Element;
use crate::dbus_common::*;
use crate::dbus_error::DbusError;
use crate::dbus_services::*;
use crate::dbus_signal::DbusSignal;
use std::{collections::HashMap, rc::Rc};
use std::io::BufWriter;
use std::fs::File;
use std::io::Write;


struct DbusMethod {
    pub return_type : DbusType,
    pub name : String,
    pub args : Vec<DbusMethodArg>,
    pub errors : Vec<String>

}

impl DbusMethod {

    pub fn get_signature(&self) -> rust::Tokens
    {
        // DBus methods return Result<Something, Error>
        let mut return_type = "-> Result<".to_string();

        if ! self.return_type.type_name.is_empty()
        {
            if self.return_type.is_returned_object
            {
                return_type += "self::";
            }
            return_type += &self.return_type.get_type_decl();
            if self.return_type.is_returned_object
            {
                return_type += "::Interface";
            }
        }
        else
        {
            // If the method return type is "void", the actual return type will
            // be Result<(), Error>
            return_type += "()";
        }
        return_type += ", Error>";

        quote! {pub fn $(&self.name)(&self$(if ! &self.args.is_empty() =>, )$(for arg in &self.args join(, ) => $(arg.get_arg_declaration()))) $return_type }
    }

    /// Returns the data structure that DBus gives us when this method is called
    pub fn get_message_type(&self) -> rust::Tokens
    {
        if self.return_type.is_returned_object
        {
            quote!(dbus::Path)
        }
        else
        {
            let return_type = &self.return_type;
            quote!($(&return_type.get_type_decl()))
        }
    }

    /// Generate code to convert the return from a DBus call to a value wrapped in an Ok()
    pub fn to_okay(&self) -> rust::Tokens
    {
        if self.return_type.is_returned_object
        {
            quote!(Ok($(&self.return_type.get_type_decl())::Interface::new(self.proxy.connection, self.proxy.destination.clone(), Some(return_val.0))))
        }
        else {
            quote!(Ok(return_val.0))
        }
    }
}

pub struct DbusInterface {
    name : String,
    project_name : String,
    service_info : DbusServiceInfo,
    methods : Vec<DbusMethod>,
    /// Error types that also need to be generated
    possible_errors : HashMap<String, Rc<dyn CodeGenerator>>,
    signals : Vec<DbusSignal>
}

impl DbusInterface {

    pub fn new(elem : &mut Element, services : &DbusServices) -> Result<DbusInterface, ()>
    {
        if let Some(name) = elem.attributes.get(&NAME_ATTRIBUTE)
        {
            let name_str = name.to_string();

            let service_info = services.get_service_info(&name_str);

            let tokens : Vec<&str> = name_str.rsplit(".").collect();
            let project_name = tokens[1].to_string();

            let mut methods : Vec<DbusMethod> = Vec::new();
            let mut possible_errors : HashMap<String, Rc<dyn CodeGenerator>> = HashMap::new();
            let mut signals : Vec<DbusSignal> = Vec::new();

            while let Some(mut method_elem) = elem.take_child("method")
            {
                let mut return_type = DbusType{type_name : "".to_string(), contained_types : Vec::new(), is_returned_object: false};
                let mut args = Vec::new();
                let mut method_errors = Vec::new();

                while let Some(arg_elem) = method_elem.take_child("arg")
                {
                    if arg_elem.attributes.get(&DIRECTION_ATTRIBUTE).unwrap() == "in"
                    {
                        let arg_type = get_dbus_type(&arg_elem);
                        
                        args.push(DbusMethodArg {
                            name : arg_elem.attributes.get(&NAME_ATTRIBUTE).unwrap().clone(),
                            arg_type
                        });
                    }
                    else
                    {
                        return_type = get_dbus_type(&arg_elem);
                    }
                }

                while let Some(mut possible_errors_elem) = method_elem.take_child("possible-errors")
                {
                    while let Some(mut error_elem) = possible_errors_elem.take_child("error")
                    {
                        let error = Rc::new(DbusError::new(&mut error_elem).unwrap());
                        method_errors.push(error.name().clone());
                        possible_errors.insert(error.name().clone(), error);
                    }
                }

                methods.push(
                    DbusMethod {
                        name : method_elem.attributes.get(&NAME_ATTRIBUTE).unwrap().to_string(),
                        return_type,
                        args,
                        errors : method_errors
                    });
            }


            while let Some(mut signal_elem) = elem.take_child("signal")
            {
                signals.push(DbusSignal::new(&mut signal_elem).unwrap());
            }

            Ok(DbusInterface{  name: tokens[0].to_string(),
                project_name,
                service_info,
                methods,
                possible_errors,
                signals })
        }
        else
        {
            Err(())
        }
    }
}

impl CodeGenerator for DbusInterface {

    fn generate(&self, output_writer : &mut BufWriter<File>) -> std::io::Result<()> {

        let name = &self.name;
        let methods = &self.methods;
        let service_info = &self.service_info;

 
        // See https://docs.rs/genco/0.17.2/genco/macro.quote.html
        let generated_code : rust::Tokens = quote! {
            pub mod $name {

                use super::*;

                // TODO Make these Path objects?
                const OBJECT_PATH : &'static str = $(quoted (&service_info.object_path));
                const INTERFACE_NAME : &'static str = $(quoted (&service_info.interface_name));

                // Signals associated with this interface
                $(for signal in &self.signals => $(signal.get_tokens())$['\r'])

                pub struct Interface<'a> {
                    proxy : Proxy<'a, &'a Connection>
                }

                impl<'a> Interface<'a> {

                    pub fn new(connection : &'a Connection, bus_name: BusName<'a>, mut path : Option<Path<'a>>) -> Self {

                        if path.is_none()
                        {
                            path = Some(Path::new(OBJECT_PATH).unwrap());
                        }

                        let proxy = connection.with_proxy(bus_name,
                            path.unwrap(),
                            DEFAULT_TIMEOUT);

                        Interface {
                            proxy
                        }
                    }


                    // For each DBus API method...
                    $(for method in methods => $['\r']$(method.get_signature())
                    {
                        $(if ! method.return_type.type_name.is_empty()
                        {
                            // ...if it's not void ...
                            //...call the method. It returns a Result<Something>
                            $['\r']let dbus_return_val : Result<($(method.get_message_type()),), dbus::Error>
                                = self.proxy.method_call(INTERFACE_NAME, $(quoted (&method.name)),
                                    ($(for arg in &method.args => $(&arg.name), )));

                            // Check the return, map errors is necessary
                            match dbus_return_val {
                                Ok(return_val) => $(&method.to_okay()),
                                Err(err) => {
                                    match err.name() {
                                        $(for err in &method.errors => 
                                            $['\r']Some(dbus_name) if dbus_name == $(err)::DBUS_NAME => Err(Box::new($(err){ message: err.message().unwrap().to_string() })),)
                                        _ => Err(Box::new(err))
                                    }
                                }
                            }
                        }
                        else
                        {
                            // ...if it is void ...
                            //...call the method. It returns a Result<()>
                            $['\r']let dbus_return_val : Result<(), dbus::Error>
                                = self.proxy.method_call(INTERFACE_NAME, $(quoted (&method.name)),
                                    ($(for arg in &method.args => $(&arg.name), )));
                            // Check the return, map errors is necessary
                            match dbus_return_val {
                                Ok(return_val) => Ok(()),
                                Err(err) => {
                                    match err.name() {
                                        $(for err in &method.errors => 
                                            $['\r']Some(dbus_name) if dbus_name == $(err)::DBUS_NAME => Err(Box::new($(err){ message: err.message().unwrap().to_string() })),)
                                        _ => Err(Box::new(err))
                                    }
                                }
                            }
                        })
                    })

                    // For each signal...
                    $(for signal in &self.signals => 
                        $['\r']pub fn listen$(&signal.name)<F>(&self, callback: F)
                            where F: Fn($(&signal.name)) -> bool + Send + 'static {
                                self.proxy.match_signal(move |sig: $(&signal.name), _: &Connection, _: &Message| {
                                callback(sig);
                                true
                            }).unwrap();
                        })
                }

                impl<'a> Drop for Interface<'a> {

                    fn drop(&mut self) {
                        if self.proxy.path != OBJECT_PATH.into()
                        {
                            let _ : Result<(), dbus::Error> =
                                self.proxy.method_call("Zinc.DBus.RefCounted", "DropRef", (1,));
                        }
                    }
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
        self.possible_errors.clone().into_values().collect()
    }
}
