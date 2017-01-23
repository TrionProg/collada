use Error;
use XMLElement;
use xmltree::Element;

use std::collections::HashMap;
use std::collections::hash_map::Entry;
use std::rc::Rc;

use node::parse_node;

use Node;
use Document;

use Camera;
use Geometry;

pub struct Scene{
    pub id:String,
    pub name:String,
    pub geometries:HashMap<String,Node<Geometry>>,
    pub cameras:HashMap<String,Node<Camera>>,
}

impl Scene{
    pub fn parse(scene:&Element, document:&mut Document) -> Result<Scene,Error>{
        let id=scene.get_attribute("id")?.clone();
        let name=scene.get_attribute("name")?.clone();

        let mut geometries=HashMap::new();
        let mut cameras=HashMap::new();

        for node_element in scene.children.iter(){
            if node_element.name.as_str()=="node" {
                parse_node(node_element, document, &mut geometries, &mut cameras)?;
            }
        }

        Ok(
            Scene{
                id:id,
                name:name,
                geometries:geometries,
                cameras:cameras,
            }
        )
    }

    pub fn print_tree(&self, last_scene:bool){
        use print_branch;
        use print_tab;

        print_tab(true);
        print_branch(last_scene);
        println!("Scene id:\"{}\" name:\"{}\"",self.id,self.name);

        print_tab(true);
        print_tab(last_scene);
        print_branch(false);
        println!("Geometries");

        if self.geometries.len()>1 {
            for (_,geometry) in self.geometries.iter().take(self.geometries.len()-1){
                geometry.print_tree(last_scene,false);
            }
        }

        match self.geometries.iter().last(){
            Some((_,geometry)) => geometry.print_tree(last_scene,true),
            None => {},
        }

        print_tab(true);
        print_tab(last_scene);
        print_branch(true);
        println!("Cameras");

        if self.cameras.len()>1 {
            for (_,camera) in self.cameras.iter().take(self.cameras.len()-1){
                camera.print_tree(last_scene,false);
            }
        }

        match self.cameras.iter().last(){
            Some((_,camera)) => camera.print_tree(last_scene,true),
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
