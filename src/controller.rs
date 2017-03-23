use Error;
use XMLElement;
use xmltree::Element;

use std::collections::HashMap;
use std::collections::hash_map::Entry;
use std::sync::Arc;

use std::fmt::Display;
use std::fmt;

use Asset;

use Bone;
use Skin;

pub enum Controller{
    Model,
    Bone(Arc<Bone>),
    Skin(Arc<Skin>),
}

impl Display for Controller{
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self{
            Controller::Model => write!(f, "Controller: model positions"),
            Controller::Bone( ref bone ) => write!(f, "Controller: {}", bone),
            Controller::Skin( ref skin ) => write!(f, "Controller: {}", skin),
        }
    }
}

pub fn parse_controllers(root:&Element, asset:&Asset) -> Result< (HashMap<String,Arc<Skin>>,HashMap<String,Arc<Skin>>), Error>{
    let controllers_element=root.get_element("library_controllers")?;

    let mut skins=HashMap::new();
    let mut skins_by_id=HashMap::new();

    for controller_element in controllers_element.children.iter(){
        if controller_element.name.as_str()=="controller" {
            let controller_id=controller_element.get_attribute("id")?.clone();

            for skin_element in controller_element.children.iter() {
                if skin_element.name.as_str()=="skin" {
                    let skin=Arc::new( Skin::parse(skin_element, controller_id, asset)? );

                    match skins_by_id.entry(skin.id.clone()){
                        Entry::Occupied(_) =>
                            return Err(Error::Other( format!("Skin with id \"{}\" already exists",&skin.id) )),
                        Entry::Vacant(entry) => {
                            entry.insert(skin.clone());
                        },
                    }

                    match skins.entry(skin.geometry_id.clone()){
                        Entry::Occupied(_) =>
                            return Err(Error::Other( format!("Geometry with id \"{}\" already has skin",&skin.geometry_id) )),
                        Entry::Vacant(entry) => {
                            entry.insert(skin);
                        },
                    }

                    break;
                }
            }
        }
    }

    Ok( (skins, skins_by_id) )
}
