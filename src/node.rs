use Error;
use XMLElement;
use xmltree::Element;

use std::collections::HashMap;
use std::collections::hash_map::Entry;
use std::sync::Arc;

use Geometry;
use Camera;
use Document;
use Axis;

pub struct Position{
    pub x:f32,
    pub y:f32,
    pub z:f32,
}

impl Position{
    pub fn new(x:f32,y:f32,z:f32) -> Self{
        Position{
            x:x,
            y:y,
            z:z,
        }
    }

    pub fn parse(text:&String, up_axis:Axis) -> Result<Self,Error>{
        let mut values=[0.0;3];

        let mut count=0;

        for (i,v) in text.split(' ').filter(|v|*v!="").take(3).enumerate(){
            match v.parse::<f32>(){
                Ok ( v ) => values[i]=v,
                Err( _ ) => return Err(Error::ParseFloatError( String::from("position"), String::from(v) ) ),
            }

            count+=1;
        }

        //check
        if count!=3 {
            return Err(Error::Other( format!("Only {} elements of position have been read, expected 3", count) ));
        }

        let position = match up_axis {
            Axis::X => Position::new(values[1],values[0],values[2]),//unknown
            Axis::Y => Position::new(values[0],values[1],values[2]),//standard
            Axis::Z => Position::new(values[0],values[2],values[1]),//blender
        };

        Ok( position )
    }
}

pub struct Scale{
    pub x:f32,
    pub y:f32,
    pub z:f32,
}

impl Scale{
    pub fn new(x:f32,y:f32,z:f32) -> Self{
        Scale{
            x:x,
            y:y,
            z:z,
        }
    }

    pub fn parse(text:&String, up_axis:Axis) -> Result<Self,Error>{
        let mut values=[0.0;3];

        let mut count=0;

        for (i,v) in text.split(' ').filter(|v|*v!="").take(3).enumerate(){
            match v.parse::<f32>(){
                Ok ( v ) => values[i]=v,
                Err( _ ) => return Err(Error::ParseFloatError( String::from("scale"), String::from(v) ) ),
            }

            count+=1;
        }

        //check
        if count!=3 {
            return Err(Error::Other( format!("Only {} elements of scale have been read, expected 3", count) ));
        }

        let scale = match up_axis {
            Axis::X => Scale::new(values[1],values[0],values[2]),//unknown
            Axis::Y => Scale::new(values[0],values[1],values[2]),//standard
            Axis::Z => Scale::new(values[0],values[2],values[1]),//blender
        };

        Ok( scale )
    }
}

pub struct Euler{
    pub pitch:f32,
    pub yaw:f32,
    pub roll:f32,
}

impl Euler {
    pub fn new(x:f32,y:f32,z:f32) -> Self{
        Euler{
            pitch:x,
            yaw:y,
            roll:z,
        }
    }

    fn parse_angle(text:&String, name:&'static str) -> Result<f32,Error> {
        let value_str=match text.split(' ').filter(|v|*v!="").nth(3){
            Some( vs ) => vs,
            None => return Err(Error::Other( format!("{} does not contains angle in digress",name))),
        };

        let angle=match value_str.parse::<f32>(){
            Ok ( v ) => v,
            Err( _ ) => return Err(Error::ParseFloatError( String::from(name), String::from(value_str) ) ),
        };

        Ok(angle)
    }

    pub fn parse(node:&Element, up_axis:Axis) -> Result<Self,Error>{
        let mut rotation_x=0.0;
        let mut rotation_y=0.0;
        let mut rotation_z=0.0;

        for node_element in node.children.iter(){
            if node_element.name.as_str()=="rotate" {
                let sid=node_element.get_attribute("sid")?;

                match sid.as_str() {
                    "rotationZ" => rotation_z=Self::parse_angle(node_element.get_text()?,"rotationZ")?,
                    "rotationY" => rotation_y=Self::parse_angle(node_element.get_text()?,"rotationY")?,
                    "rotationX" => rotation_x=Self::parse_angle(node_element.get_text()?,"rotationX")?,
                    _ => return Err( Error::Other(format!("Unknown sid of rotation: \"{}\"",sid)) ),
                }
            }
        }

        let euler = match up_axis {
            Axis::X => Euler::new(rotation_y,rotation_x,rotation_z),//unknown
            Axis::Y => Euler::new(rotation_x,rotation_y,rotation_z),//standard
            Axis::Z => Euler::new(rotation_x,rotation_z,rotation_y),//blender
        };

        Ok( euler )
    }
}
/*
pub struct Matrix{
    pub values:[f32;16],
}

impl Matrix{
    pub fn parse(text:&String) -> Result<Matrix,Error>{
        let mut values=[0.0;16];

        let mut count=0;

        for (i,v) in text.split(' ').filter(|v|*v!="").take(16).enumerate(){
            match v.parse::<f32>(){
                Ok ( v ) => {values[i]=v;},
                Err( _ ) => return Err(Error::Other( format!("Can not parse value of matrix {} as float", v) )),
            }

            count+=1;
        }

        //check
        if count!=16 {
            return Err(Error::Other( format!("Only {} elements of matrix have been read", count) ));
        }

        Ok(
            Matrix{
                values:values,
            }
        )
    }
}
*/

pub struct Node<T>{
    pub id:String,
    pub name:String,
    pub position:Position,
    pub rotation:Euler,
    pub scale:Scale,
    pub joined:Arc<T>,
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
    }
}

pub fn parse_node(node:&Element, document:&mut Document, geometries:&mut HashMap<String,Node<Geometry>>, cameras:&mut HashMap<String,Node<Camera>>) -> Result<(),Error>{
    let id=node.get_attribute("id")?.clone();
    let name=node.get_attribute("name")?.clone();

    let position=match node.get_element("translate"){
        Ok ( position ) => Position::parse(position.get_text()?, document.asset.up_axis)?,
        Err ( _ ) => Position::new(0.0, 0.0, 0.0),
    };

    let scale=match node.get_element("scale"){
        Ok ( scale ) => Scale::parse(scale.get_text()?, document.asset.up_axis)?,
        Err ( _ ) => Scale::new(0.0, 0.0, 0.0),
    };

    let rotation=Euler::parse(node, document.asset.up_axis)?;

    //let matrix_str=node.get_element("matrix")?.get_text()?;
    //let matrix=Matrix::parse(matrix_str)?;

    for instance in node.children.iter(){
        if instance.name.as_str()=="instance_geometry" {
            let joined_id=instance.get_attribute("url")?.trim_left_matches('#');

            let joined=match document.geometries.get(joined_id){
                Some( geometry ) => geometry.clone(),
                None => return Err( Error::Other( format!("Geometry \"{}\" does not exists",joined_id)) ),
            };

            match geometries.entry(name.clone()){
                Entry::Occupied(_) => return Err(Error::Other( format!("Dublicate geometry node with name \"{}\"",&name) )),
                Entry::Vacant(entry) => {entry.insert(
                    Node::<Geometry>{
                        id:id,
                        name:name,
                        //matrix:matrix,
                        position:position,
                        rotation:rotation,
                        scale:scale,
                        joined:joined,
                    }
                );},
            }

            return Ok(())
        }else if instance.name.as_str()=="instance_camera" {
            let joined_id=instance.get_attribute("url")?.trim_left_matches('#');

            let joined=match document.cameras.get(joined_id){
                Some( camera ) => camera.clone(),
                None => return Err( Error::Other( format!("Camera \"{}\" does not exists",joined_id)) ),
            };

            match cameras.entry(name.clone()){
                Entry::Occupied(_) => return Err(Error::Other( format!("Dublicate camera node with name \"{}\"",&name) )),
                Entry::Vacant(entry) => {entry.insert(
                    Node::<Camera>{
                        id:id,
                        name:name,
                        position:position,
                        rotation:rotation,
                        scale:scale,
                        joined:joined,
                    }
                );},
            }

            return Ok(())
        }else if instance.name.as_str()=="instance_light" {//TODO:add light and light node
            return Ok(())
        }
    }

    Err(Error::NoElement{
        element_name:node.name.clone(),
        child_element_name:String::from("instance"),
    })
}
