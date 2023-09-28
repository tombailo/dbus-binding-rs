extern crate xmltree;

use xmltree::Element;
use std::collections::HashMap;

#[derive(Clone)]
pub struct DbusServiceInfo {
    pub default_bus_name : String,
    pub object_path : String,
    pub interface_name : String
}

pub struct DbusServices {

    default_bus_name : String,
    services: HashMap<String, DbusServiceInfo>
}

impl DbusServices {

    pub fn new(bus_name: &str) -> Self {

        DbusServices { default_bus_name : bus_name.into(), services : HashMap::new() }
    }

    pub fn parse_services(&mut self, elem : &mut Element) {

        while let Some(node_elem) = elem.take_child("node")
        {
            if let Some(object_path) = node_elem.get_attribute("name")
            {
                if let Some(interface_element) = node_elem.get_child("interface")
                {
                    if let Some(interface_name) = interface_element.get_attribute("name")
                    {
                        println!("{} {}", object_path, interface_name);
                        self.services.insert(interface_name.clone(),
                            DbusServiceInfo{ default_bus_name: self.default_bus_name.clone(),
                                object_path: "/".to_string() + object_path,
                                interface_name : interface_name.clone()});
                    }
                }
            }
        }
    }

    pub fn get_service_info(&self, interface : &str) -> DbusServiceInfo {
        match self.services.get(interface)
        {
            Some(service) => service.clone(),
            None => {
                println!("Returning default service info");
                DbusServiceInfo { default_bus_name: self.default_bus_name.clone(),
                    object_path: "".into(),
                    interface_name: "".into() }
            }
        }
    }
}
