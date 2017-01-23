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
    pub nodes:HashMap<String,Node>,
}

//TODO: Store cameras, geoms, light in separate hash maps

impl Scene{
    pub fn parse(scene:&Element, document:&mut Document) -> Result<Scene,Error>{
        let id=scene.get_attribute("id")?.clone();
        let name=scene.get_attribute("name")?.clone();

        let mut nodes=HashMap::new();

        for node_element in scene.children.iter(){
            if node_element.name.as_str()=="node" {
                let node=Node::parse(node_element, document)?;

                match nodes.entry(node.name.clone()){
                    Entry::Occupied(_) => return Err(Error::Other( format!("Dublicate node with name \"{}\"",node.name) )),
                    Entry::Vacant(entry) => {
                        entry.insert( node );
                    },
                }
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

    pub fn print_tree(&self, last_scene:bool){
        use print_branch;
        use print_tab;

        print_tab(true);
        print_branch(last_scene);
        println!("Source id:\"{}\" name:\"{}\"",self.id,self.name);

        if self.nodes.len()>1 {
            for (_,node) in self.nodes.iter().take(self.nodes.len()-1){
                node.print_tree(last_scene,false);
            }
        }

        match self.nodes.iter().last(){
            Some((_,node)) => node.print_tree(last_scene,true),
            None => {},
        }
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
