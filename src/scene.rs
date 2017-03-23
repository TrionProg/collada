use Error;
use XMLElement;
use xmltree::Element;

use std::collections::HashMap;
use std::collections::hash_map::Entry;
use std::sync::Arc;

use node::parse_node;

use Node;
use Document;

use Camera;
use Geometry;
use Skeleton;
use Skin;
use TreePrinter;

pub struct Scene{
    pub id:String,
    pub name:String,
    pub geometries:HashMap<String,Node<Geometry>>,
    pub cameras:HashMap<String,Node<Camera>>,
    pub skeletons:HashMap<String,Node<Skeleton>>,
}

impl Scene{
    pub fn parse(scene:&Element, document:&mut Document, skins_by_id:&HashMap<String,Arc<Skin>>) -> Result<Scene,Error>{
        let id=scene.get_attribute("id")?.clone();
        let name=scene.get_attribute("name")?.clone();

        let mut geometries=HashMap::new();
        let mut cameras=HashMap::new();
        let mut skeletons=HashMap::new();

        for node_element in scene.children.iter(){
            if node_element.name.as_str()=="node" {
                parse_node(node_element, document, skins_by_id, None, &mut geometries, &mut cameras, &mut skeletons)?;
            }
        }

        Ok(
            Scene{
                id:id,
                name:name,
                geometries:geometries,
                cameras:cameras,
                skeletons:skeletons,
            }
        )
    }

    pub fn print(&self, printer:TreePrinter) {
        println!("Scene id:\"{}\" name:\"{}\"",self.id,self.name);

        self.print_geometries( printer.new_branch(false) );
        self.print_skeletons( printer.new_branch(false) );
        self.print_cameras( printer.new_branch(true) );
    }

    fn print_geometries(&self, printer:TreePrinter) {
        println!("Geometries");

        for (last,(_,geometry)) in self.geometries.iter().clone().enumerate().map(|i| (i.0==self.geometries.len()-1,i.1) ){
            geometry.print( printer.new_branch(last) );
        }
    }

    fn print_skeletons(&self, printer:TreePrinter) {
        println!("Skeletons");

        for (last,(_,skeleton)) in self.skeletons.iter().clone().enumerate().map(|i| (i.0==self.skeletons.len()-1,i.1) ){
            skeleton.print( printer.new_branch(last) );
        }
    }

    fn print_cameras(&self, printer:TreePrinter) {
        println!("Cameras");

        for (last,(_,camera)) in self.cameras.iter().clone().enumerate().map(|i| (i.0==self.cameras.len()-1,i.1) ){
            camera.print( printer.new_branch(last) );
        }
    }

    /*
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
        print_branch(false);
        println!("Skeletons");

        if self.skeletons.len()>1 {
            for (_,skeleton) in self.skeletons.iter().take(self.skeletons.len()-1){
                skeleton.print_tree(last_scene,false);
            }
        }

        match self.skeletons.iter().last(){
            Some((_,skeleton)) => skeleton.print_tree(last_scene,true),
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
    */
}

pub fn parse_scenes(root:&Element, document:&mut Document, skins_by_id:HashMap<String,Arc<Skin>>) -> Result<(), Error>{
    let scenes_element=root.get_element("library_visual_scenes")?;

    for scene_element in scenes_element.children.iter(){
        if scene_element.name.as_str()=="visual_scene" {
            let scene=Scene::parse(scene_element, document, &skins_by_id)?;

            match document.scenes.entry(scene.id.clone()){
                Entry::Occupied(_) => return Err(Error::Other( format!("Dublicate scene with id \"{}\"", &scene.id) )),
                Entry::Vacant(entry) => { entry.insert(Arc::new(scene)); },
            }
        }
    }

    Ok(())
}
