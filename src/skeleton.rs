use Error;
use XMLElement;
use xmltree::Element;

use std::collections::HashMap;
use std::collections::hash_map::Entry;
use std::sync::Arc;

use node::parse_node;

use std::fmt::Display;
use std::fmt;

use Geometry;
use Camera;
use Document;
use Asset;
use Axis;
use Editor;
use Node;
use Skin;
use TreePrinter;

use Location;
use Matrix;
use Position;
use Quaternion;
use Scale;

pub struct Skeleton{
    pub id:String,
    pub location:Location,
    pub bones_array:Vec<Arc<Bone>>,
    pub bones:HashMap<String,Arc<Bone>>,
}

impl Skeleton {
    pub fn parse(
        skeleton_element:&Element,
        document:&mut Document,
        skins_by_id:&HashMap<String,Arc<Skin>>,
        id:String,
        location:Location,
        geometries:&mut HashMap<String,Node<Geometry>>,
        cameras:&mut HashMap<String,Node<Camera>>,
        skeletons:&mut HashMap<String,Node<Skeleton>>,
    ) -> Result<Skeleton,Error> {
        let mut bones_array=Vec::new();
        let mut bones=HashMap::new();

        for node_element in skeleton_element.children.iter(){
            if node_element.name.as_str()=="node" {
                let node_type=node_element.get_attribute("type")?;

                if node_type.as_str()=="JOINT" {
                    Bone::parse(node_element, document, skins_by_id, id.clone(), None, geometries, cameras, skeletons, &mut bones_array, &mut bones)?;
                }
            }
        }

        //Bone::parse(root_bone_element, document, skins_by_id, id.clone(), None, geometries, cameras, skeletons, &mut bones_array, &mut bones)?;

        let skeleton=Skeleton{
            id:id,
            location:location,
            bones_array:bones_array,
            bones:bones,
        };

        Ok( skeleton )
    }

    pub fn print(&self, printer:TreePrinter) {
        println!("Skeleton id:\"{}\"", self.id);

        if self.bones_array.len()>0 {
            self.bones_array[0].print( printer.new_branch(true), &self.bones_array );
        }
    }
}

pub struct Bone{
    pub id:String,
    pub sid:String,
    pub name:String,
    pub skeleton_id:String,
    pub index:usize,
    pub parent:Option<usize>,

    pub location:Location,
}

impl Display for Bone{
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Bone name:\"{}\" of skeleton with id \"{}\"", self.name, self.skeleton_id)
    }
}

impl Bone {
    pub fn parse(
        bone_element:&Element,
        document:&mut Document,
        skins_by_id:&HashMap<String,Arc<Skin>>,
        skeleton_id:String,
        parent:Option<usize>,
        geometries:&mut HashMap<String,Node<Geometry>>,
        cameras:&mut HashMap<String,Node<Camera>>,
        skeletons:&mut HashMap<String,Node<Skeleton>>,
        bones_array:&mut Vec<Arc<Bone>>,
        bones:&mut HashMap<String,Arc<Bone>>,
    ) -> Result<(),Error> {
        let id=bone_element.get_attribute("id")?.clone();
        let sid=bone_element.get_attribute("sid")?.clone();
        let name=bone_element.get_attribute("name")?.clone();
        let index=bones_array.len();

        let location=Matrix::parse( bone_element.get_element("matrix")?.get_text()? )?.to_location(&document.asset);

        let bone=Arc::new( Bone{
            id:id.clone(),
            sid:sid,
            name:name,
            skeleton_id:skeleton_id.clone(),
            index:index,
            parent:parent,

            location:location,
        } );

        bones_array.push(bone.clone());

        match bones.entry(id.clone()){
            Entry::Occupied(_) => return Err(Error::Other( format!("Duplicate bone node with id \"{}\"",&id) )),
            Entry::Vacant(entry) => {entry.insert( bone.clone() );},
        }

        for node_element in bone_element.children.iter(){
            if node_element.name.as_str()=="node" {
                let node_type=node_element.get_attribute("type")?;

                if node_type.as_str()=="JOINT" {
                    Bone::parse(node_element, document, skins_by_id, skeleton_id.clone(), Some(index), geometries, cameras, skeletons, bones_array, bones)?;
                }else{
                    parse_node(node_element, document, skins_by_id, Some(bone.clone()), geometries, cameras, skeletons)?;
                }
            }
        }

        Ok(())
    }

    pub fn print(&self, printer:TreePrinter, bones_array:&Vec<Arc<Bone>>) {
        println!("Bone index:{} id:\"{}\" name:\"{}\"", self.index, self.id, self.name);

        let mut children:Vec<&Arc<Bone>>=Vec::new();

        for bone in bones_array.iter() {
            match bone.parent {
                Some( parent_index ) => {
                    if parent_index==self.index {
                        children.push(bone);
                    }
                },
                None => {},
            }
        }

        for (last,bone) in children.iter().clone().enumerate().map(|i| (i.0==children.len()-1,i.1) ){
            bone.print( printer.new_branch(last), bones_array );
        }
    }
}
