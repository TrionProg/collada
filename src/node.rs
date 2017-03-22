use Error;
use XMLElement;
use xmltree::Element;

use std::collections::HashMap;
use std::collections::hash_map::Entry;
use std::sync::Arc;

use Geometry;
use Camera;
use Document;
use Asset;
use Axis;
use Editor;
use Bone;
use Skeleton;
use Skin;
use Controller;

use Position;
use Euler;
use Scale;

pub struct Node<T>{
    pub id:String,
    pub name:String,
    pub position:Position,
    pub rotation:Euler,
    pub scale:Scale,
    pub joined:Arc<T>,
    pub controller:Controller,
}

impl Node<Geometry>{
    pub fn print_tree(&self, last_scene:bool, last_node:bool){
        use print_branch;
        use print_tab;

        print_tab(true);
        print_tab(last_scene);
        print_tab(false);
        print_branch(last_node);
        println!("Node id:\"{}\" name:\"{}\" joided to \"{}\"",self.id,self.name,self.joined.id);
        self.controller.print();
    }
}

impl Node<Camera>{
    pub fn print_tree(&self, last_scene:bool, last_node:bool){
        use print_branch;
        use print_tab;

        print_tab(true);
        print_tab(last_scene);
        print_tab(true);
        print_branch(last_node);
        println!("Node id:\"{}\" name:\"{}\" joided to \"{}\"",self.id,self.name,self.joined.id);
        self.controller.print();
    }
}

impl Node<Skeleton>{
    pub fn print_tree(&self, last_scene:bool, last_node:bool){
        use print_branch;
        use print_tab;

        print_tab(true);
        print_tab(last_scene);
        print_tab(true);
        print_branch(last_node);
        println!("Skeleton id:\"{}\" name:\"{}\"",self.id,self.name);
        self.controller.print();
    }
}

pub fn parse_node(
    node:&Element,
    document:&mut Document,
    skins_by_id:&HashMap<String,Arc<Skin>>,
    bone:Option<Arc<Bone>>,
    geometries:&mut HashMap<String,Node<Geometry>>,
    cameras:&mut HashMap<String,Node<Camera>>,
    skeletons:&mut HashMap<String,Node<Skeleton>>
) -> Result<(),Error>{
    let id=node.get_attribute("id")?.clone();
    let name=node.get_attribute("name")?.clone();

    let position=match node.get_element("translate"){
        Ok ( position ) => Position::parse(position.get_text()?, &document.asset)?,
        Err ( _ ) => Position::new(0.0, 0.0, 0.0),
    };

    let scale=match node.get_element("scale"){
        Ok ( scale ) => Scale::parse(scale.get_text()?, &document.asset)?,
        Err ( _ ) => Scale::new(0.0, 0.0, 0.0),
    };

    let rotation=Euler::parse(node, &document.asset)?;

    //let matrix_str=node.get_element("matrix")?.get_text()?;
    //let matrix=Matrix::parse(matrix_str)?;

    for instance in node.children.iter(){
        if instance.name.as_str()=="instance_geometry" {
            let geometry_id=instance.get_attribute("url")?.trim_left_matches('#');

            let joined=match document.geometries.get(geometry_id){
                Some( geometry ) => geometry.clone(),
                None => return Err( Error::Other( format!("Geometry \"{}\" does not exists",geometry_id)) ),
            };

            match geometries.entry(name.clone()){
                Entry::Occupied(_) => return Err(Error::Other( format!("Duplicate geometry node with name \"{}\"",&name) )),
                Entry::Vacant(entry) => {entry.insert(
                    Node::<Geometry>{
                        id:id,
                        name:name,
                        //matrix:matrix,
                        position:position,
                        rotation:rotation,
                        scale:scale,
                        joined:joined,
                        controller:match bone {
                            Some( bone ) => Controller::Bone( bone ),
                            None => Controller::Model,
                        }
                    }
                );},
            }

            return Ok(());
        }else if instance.name.as_str()=="instance_camera" {
            let camera_id=instance.get_attribute("url")?.trim_left_matches('#');

            let joined=match document.cameras.get(camera_id){
                Some( camera ) => camera.clone(),
                None => return Err( Error::Other( format!("Camera \"{}\" does not exists",camera_id)) ),
            };

            match cameras.entry(name.clone()){
                Entry::Occupied(_) => return Err(Error::Other( format!("Duplicate camera node with name \"{}\"",&name) )),
                Entry::Vacant(entry) => {entry.insert(
                    Node::<Camera>{
                        id:id,
                        name:name,
                        position:position,
                        rotation:rotation,
                        scale:scale,
                        joined:joined,
                        controller:match bone {
                            Some( bone ) => Controller::Bone( bone ),
                            None => Controller::Model,
                        }
                    }
                );},
            }

            return Ok(());
        }else if instance.name.as_str()=="instance_light" {//TODO:add light and light node
            return Ok(());
        }else if instance.name.as_str()=="instance_controller" {
            let skin_id=instance.get_attribute("url")?.trim_left_matches('#');

            let skin=match skins_by_id.get(skin_id) {
                Some( skin ) =>
                    skin.clone(),
                None => return Err(Error::Other( format!("Skin with id \"{}\" does not exists",skin_id) )),
            };

            let joined=match document.geometries.get(&skin.geometry_id){
                Some( geometry ) => geometry.clone(),
                None => return Err(Error::Other( format!("Geometry with id \"{}\" does not exists",&skin.geometry_id) )),
            };

            match geometries.entry(name.clone()){
                Entry::Occupied(_) => return Err(Error::Other( format!("Duplicate geometry node with name \"{}\"",&name) )),
                Entry::Vacant(entry) => {entry.insert(
                    Node::<Geometry>{
                        id:id,
                        name:name,
                        //matrix:matrix,
                        position:position,
                        rotation:rotation,
                        scale:scale,
                        joined:joined,
                        controller:Controller::Skin(skin),
                    }
                );},
            }

            return Ok(());
        }
    }

    for root_bone in node.children.iter(){
        if root_bone.name.as_str()=="node" && root_bone.get_attribute("type")?.as_str()=="JOINT" {
            let skeleton=Skeleton::parse(root_bone, document, skins_by_id, geometries, cameras, skeletons)?;

            let controller=match bone {
                Some( bone ) => return Err(Error::Other( format!("Skeleton with id \"{}\" can not be joined to bone (id:\"{}\")", id, bone.id) )),
                None => Controller::Model,
            };

            match skeletons.entry(name.clone()){
                Entry::Occupied(_) => return Err(Error::Other( format!("Duplicate skeleton node with name \"{}\"",&name) )),
                Entry::Vacant(entry) => {
                    entry.insert(
                    Node::<Skeleton>{
                        id:id,
                        name:name,
                        position:position,
                        rotation:rotation,
                        scale:scale,
                        joined:Arc::new(skeleton),
                        controller:controller,
                    }
                );},
            }

            return Ok(());
        }
    }

    Err(Error::NoElement{
        element_name:node.name.clone(),
        child_element_name:String::from("instance"),
    })
}
