use Error;
use XMLElement;
use xmltree::Element;

use std::collections::HashMap;
use std::collections::hash_map::Entry;
use std::rc::Rc;

pub struct Perspective{
    pub z_near:f32,
    pub z_far:f32,
    pub x_fov:f32,
}

//TODO <extra><technique profile="blender">

pub struct Camera{
    pub id:String,
    pub name:String,
    pub perspective:Perspective,
}

impl Camera{
    pub fn parse(camera:&Element) -> Result<Camera,Error>{
        let id=camera.get_attribute("id")?.clone();
        let name=camera.get_attribute("name")?.clone();

        let perspective=camera.get_element("optics")?.get_element("technique_common")?.get_element("perspective")?;

        let z_near=perspective.parse_text_as_f32("znear")?;
        let z_far=perspective.parse_text_as_f32("zfar")?;
        let x_fov=perspective.parse_text_as_f32("xfov")?;

        Ok(
            Camera{
                id:id,
                name:name,
                perspective:Perspective{
                    z_near:z_near,
                    z_far:z_far,
                    x_fov:x_fov,
                }
            }
        )
    }
}

pub fn parse_cameras(root:&Element) -> Result< HashMap<String,Rc<Camera>>, Error>{
    let cameras_element=root.get_element("library_cameras")?;
    let mut cameras=HashMap::new();

    for camera_element in cameras_element.children.iter(){
        let camera=Camera::parse(&camera_element)?;

        match cameras.entry(camera.id.clone()){
            Entry::Occupied(_) => return Err(Error::Other( format!("Dublicate camera with id \"{}\"", &camera.id) )),
            Entry::Vacant(entry) => { entry.insert(Rc::new(camera)); },
        }
    }

    Ok(cameras)
}
