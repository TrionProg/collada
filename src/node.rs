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
use TreePrinter;

use Location;
use Matrix;
use Position;
use Quaternion;
use Scale;

pub struct Node<T>{
    pub id:String,
    pub name:String,
    pub location:Location,
    pub joined:Arc<T>,
    pub controller:Controller,
}

impl Node<Geometry>{
    pub fn print(&self, printer:TreePrinter) {
        println!("Node id:\"{}\" name:\"{}\" joided to \"{}\"",self.id,self.name,self.joined.id);

        printer.new_branch(true);
        println!("{}", self.controller);
    }
}

impl Node<Camera>{
    pub fn print(&self, printer:TreePrinter) {
        println!("Node id:\"{}\" name:\"{}\" joided to \"{}\"",self.id,self.name,self.joined.id);

        printer.new_branch(true);
        println!("{}", self.controller);
    }
}

impl Node<Skeleton>{
    pub fn print(&self, printer:TreePrinter) {
        println!("Node id:\"{}\" name:\"{}\" joided to \"{}\"",self.id,self.name,self.joined.id);
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

    let location = match node.get_element("matrix") {
        Ok( matrix_element ) => Matrix::parse(matrix_element.get_text()?)?.to_location(&document.asset),
        _ => {
            let position=match node.get_element("translate"){
                Ok ( position ) => Position::parse(position.get_text()?, &document.asset)?,
                Err ( _ ) => Position::new(0.0, 0.0, 0.0),
            };

            let scale=match node.get_element("scale"){
                Ok ( scale ) => Scale::parse(scale.get_text()?, &document.asset)?,
                Err ( _ ) => Scale::new(0.0, 0.0, 0.0),
            };

            let rotation=Quaternion::parse_angles(node, &document.asset)?;

            Location::new(position, scale, rotation)
        },
    };

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
                        location:location,
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
                        location:location,
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
                        location:location,
                        joined:joined,
                        controller:Controller::Skin(skin),
                    }
                );},
            }

            return Ok(());
        }
    }

    for root_bone in node.children.iter(){
        if root_bone.name.as_str()=="node" && root_bone.get_attribute("type")?.as_str()=="JOINT" { //This is skeleton
            let skeleton=Arc::new( Skeleton::parse(node, document, skins_by_id, id.clone(), location.clone(), geometries, cameras, skeletons)? );

            let controller=match bone {
                Some( bone ) => return Err(Error::Other( format!("Skeleton with id \"{}\" can not be joined to bone (id:\"{}\")", id, bone.id) )),
                None => Controller::Model,
            };

            match document.skeletons.entry(id.clone()){
                Entry::Occupied(_) => return Err(Error::Other( format!("Duplicate skeleton node with id \"{}\"",&id) )),
                Entry::Vacant(entry) => {
                    entry.insert(skeleton.clone());
                },
            }

            match skeletons.entry(name.clone()){
                Entry::Occupied(_) => return Err(Error::Other( format!("Duplicate skeleton node with name \"{}\"",&name) )),
                Entry::Vacant(entry) => {
                    entry.insert(
                    Node::<Skeleton>{
                        id:id,
                        name:name,
                        location:location,
                        joined:skeleton,
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
