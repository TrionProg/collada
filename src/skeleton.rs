use Error;
use XMLElement;
use xmltree::Element;

use std::collections::HashMap;
use std::collections::hash_map::Entry;
use std::sync::Arc;

use node::parse_node;

use Geometry;
use Camera;
use Document;
use Asset;
use Axis;
use Editor;
use Node;
use Skin;

use Matrix;

pub struct Skeleton{
    pub bones_array:Vec<Arc<Bone>>,
    pub bones:HashMap<String,Arc<Bone>>,
}

impl Skeleton {
    pub fn parse(
        root_bone_element:&Element,
        document:&mut Document,
        skins_by_id:&HashMap<String,Arc<Skin>>,
        geometries:&mut HashMap<String,Node<Geometry>>,
        cameras:&mut HashMap<String,Node<Camera>>,
        skeletons:&mut HashMap<String,Node<Skeleton>>,
    ) -> Result<Skeleton,Error>{
        let mut bones_array=Vec::new();
        let mut bones=HashMap::new();

        Bone::parse(root_bone_element, document, skins_by_id, None, geometries, cameras, skeletons, &mut bones_array, &mut bones)?;

        let skeleton=Skeleton{
            bones_array:bones_array,
            bones:bones,
        };

        Ok( skeleton )
    }
}

pub struct Bone{
    pub id:String,
    pub sid:String,
    pub name:String,
    pub index:usize,
    pub parent:Option<usize>,

    pub matrix:Matrix,
}

impl Bone {
    pub fn parse(
        bone_element:&Element,
        document:&mut Document,
        skins_by_id:&HashMap<String,Arc<Skin>>,
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

        let matrix=Matrix::parse( bone_element.get_element("matrix")?.get_text()?, &document.asset )?;

        let bone=Arc::new( Bone{
            id:id.clone(),
            sid:sid,
            name:name,
            index:index,
            parent:parent,
            matrix:matrix,
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
                    Bone::parse(node_element, document, skins_by_id, Some(index), geometries, cameras, skeletons, bones_array, bones)?;
                }else{
                    parse_node(node_element, document, skins_by_id, Some(bone.clone()), geometries, cameras, skeletons)?;
                }
            }
        }

        Ok(())
    }
}
