use genco::prelude::*;
use std::io::BufWriter;
use std::fs::File;
use xmltree::Element;
use std::io::Write;

use crate::dbus_common::*;

pub struct DbusStruct {
    /// Unqualified name of the struct
    name : String,
    /// Name of the project containing the struct, minus the first word
    project_name : String,
    /// Names of struct members
    members : Vec<String>,
    /// The Rust types of the members as exposed on API.
    /// These types may be user-defined in the case of enums
    member_ext_types : Vec<DbusType>
}

impl DbusStruct {

    pub fn new(elem : &mut Element) -> Result<DbusStruct, ()> {
        if let Some(name) = elem.attributes.get(&NAME_ATTRIBUTE)
        {
            let name_str = name.to_string();
            let tokens : Vec<&str> = name_str.rsplit(".").collect();

            let mut members : Vec<String> = Vec::new();
            let mut member_ext_types : Vec<DbusType> = Vec::new();

            while let Some(child) = elem.take_child("member")
            {
                members.push(prefix_keywords(child.attributes.get(&NAME_ATTRIBUTE).unwrap()));
                member_ext_types.push(get_dbus_type(&child));
            }

            Ok( DbusStruct { name: tokens[0].to_string(),
                project_name : tokens[1].to_string(),
                members,
                member_ext_types} )
        }
        else
        {
            Err(())
        }
    }
}

impl CodeGenerator for DbusStruct {

    fn generate(&self, output_writer : &mut BufWriter<File>) -> std::io::Result<()>{

        let name = &self.name;
        let members = &self.members;
        let member_ext_types = &self.member_ext_types;

        let mut member_initialisers : Vec<String> = Vec::new();

        for (n, _) in members.into_iter().enumerate()
        {
            member_initialisers.push("members.".to_string() + &n.to_string());
        }

        // See https://docs.rs/genco/0.17.2/genco/macro.quote.html
        let generated_code : rust::Tokens = quote! {

            pub type $(name)Message = ($(for t in member_ext_types => $(t.get_type_decl()), ));

            #[derive(Debug, Clone)]
            pub struct $name {
                $(for (m, t) in members.into_iter().zip(member_ext_types) join(, ) => $['\r']pub $m : $(t.get_type_decl()))
            }

            impl $name {

                pub fn new(members : $(name)Message) -> Self {

                    $name {
                        $(for (m, i) in members.into_iter().zip(member_initialisers) join(, ) => $['\r']$m : $i)
                    }
                }
            }

            impl dbus::arg::ReadAll for $name {
                fn read(i: &mut dbus::arg::Iter) -> Result<Self, dbus::arg::TypeMismatchError> {
                    Ok($name {
                        $(for m in members => $['\r']$m : i.read().unwrap(),)
                    })
                }
            }

            impl dbus::arg::Arg for $(name) {

                const ARG_TYPE : ArgType = ArgType::Struct;

                fn signature() -> Signature<'static> {
                    <$(name)Message as dbus::arg::Arg>::signature()
                }
            }

            impl<'a> dbus::arg::Get<'a> for $(name) {

                fn get(i: &mut Iter<'a>) -> Option<Self>
                {
                    Some($(name)::new(<$(name)Message as dbus::arg::Get>::get(i)?))
                }
            }

            impl dbus::arg::Append for $(name) {
                fn append_by_ref(&self, i: &mut IterAppend<'_>) {
                    $(for m in members => $['\r']self.$m.append_by_ref(i);)
                }

                fn append(self, i: &mut IterAppend<'_>) {
                    $(for m in members => $['\r']self.$m.append_by_ref(i);)
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
