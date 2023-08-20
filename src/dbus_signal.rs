use genco::prelude::*;
use xmltree::Element;
use crate::dbus_common::*;


pub struct DbusSignal {
    pub name : String,
    pub args : Vec<DbusMethodArg>
}

impl DbusSignal {

    pub fn new(elem : &mut Element) -> Result<DbusSignal, ()>
    {
        if let Some(name) = elem.attributes.get(&NAME_ATTRIBUTE)
        {
            let name_str = name.to_string();
            let mut args = Vec::new();

            while let Some(arg_elem) = elem.take_child("arg")
            {
                let arg_type = get_dbus_type(&arg_elem);

                args.push(DbusMethodArg {
                    name : arg_elem.attributes.get(&NAME_ATTRIBUTE).unwrap().clone(),
                    arg_type
                });
            }

            Ok(DbusSignal{
               name: name_str,
               args})
        }
        else
        {
            Err(())
        }
    }

    pub fn get_tokens(&self) -> rust::Tokens {

        let generated_code : rust::Tokens = quote! {
            #[derive(Debug)]
            pub struct $(&self.name) {
                $(for arg in &self.args => pub $(&arg.name) : $(&arg.arg_type.get_type_decl()),$['\r'] )
            }

            impl dbus::arg::ReadAll for $(&self.name) {
                fn read(i: &mut dbus::arg::Iter) -> Result<Self, dbus::arg::TypeMismatchError> {
                    Ok(Self{$(for arg in &self.args => $(&arg.name) : i.read()?,)})
                }
            }

            impl dbus::message::SignalArgs for $(&self.name) {
                const NAME: &'static str = $(quoted (&self.name));
                const INTERFACE: &'static str = INTERFACE_NAME;
            }
        };
        generated_code
    }
}
