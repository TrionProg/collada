use Error;
use XMLElement;
use xmltree::Element;

use std::collections::HashMap;
use std::collections::hash_map::Entry;
use std::rc::Rc;

use Node;
use Document;

pub struct Scene{
    pub id:String,
    pub name:String,
    pub nodes:Vec<Node>,
}

impl Scene{
    pub fn parse(scene:&Element, document:&mut Document) -> Result<Scene,Error>{
        let id=scene.get_attribute("id")?.clone();
        let name=scene.get_attribute("name")?.clone();

        let mut nodes=Vec::new();

        for node_element in scene.children.iter(){
            if node_element.name.as_str()=="node" {
                let node=Node::parse(node_element, document)?;

                nodes.push(node);
            }
        }

        Ok(
            Scene{
                id:id,
                name:name,
                nodes:nodes,
            }
        )
    }
}

pub fn parse_scenes(root:&Element, document:&mut Document) -> Result<(), Error>{
    let scenes_element=root.get_element("library_visual_scenes")?;

    for scene_element in scenes_element.children.iter(){
        if scene_element.name.as_str()=="visual_scene" {
            let scene=Scene::parse(scene_element, document)?;

            match document.scenes.entry(scene.id.clone()){
                Entry::Occupied(_) => return Err(Error::Other( format!("Dublicate scene with id \"{}\"", &scene.id) )),
                Entry::Vacant(entry) => { entry.insert(Rc::new(scene)); },
            }
        }
    }

    Ok(())
}
