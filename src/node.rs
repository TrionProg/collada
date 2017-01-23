use Error;
use XMLElement;
use xmltree::Element;

use std::collections::HashMap;
use std::collections::hash_map::Entry;
use std::rc::Rc;

use Geometry;
use Camera;
use Document;

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

            count=i;
        }

        //check
        if count!=15 {
            return Err(Error::Other( format!("Only {} elements of matrix have been read", count) ));
        }

        Ok(
            Matrix{
                values:values,
            }
        )
    }
}

pub struct Node<T>{
    pub id:String,
    pub name:String,
    pub matrix:Matrix,
    pub joined:Rc<T>,
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

    let matrix_str=node.get_element("matrix")?.get_text()?;
    let matrix=Matrix::parse(matrix_str)?;

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
                        matrix:matrix,
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
                        matrix:matrix,
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
