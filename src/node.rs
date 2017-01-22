use Error;
use XMLElement;
use xmltree::Element;

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

        for (i,v) in text.split(' ').filter(|v|*v!="").take(16).enumerate(){
            match v.parse::<f32>(){
                Ok ( v ) => {values[i]=v;},
                Err( _ ) => return Err(Error::Other( format!("Can not parse value of matrix {} as float", v) )),
            }
        }

        //check

        Ok(
            Matrix{
                values:values,
            }
        )
    }
}

pub enum JoinedTo{
    Geometry(Rc<Geometry>),
    Camera(Rc<Camera>),
}

pub struct Node{
    pub id:String,
    pub name:String,
    pub matrix:Matrix,
    pub joined:JoinedTo,
}

impl Node{
    pub fn parse(node:&Element, document:&mut Document) -> Result<Node,Error>{
        let id=node.get_attribute("id")?.clone();
        let name=node.get_attribute("name")?.clone();

        let matrix_str=node.get_element("matrix")?.get_text()?;
        let matrix=Matrix::parse(matrix_str)?;

        let joined=Node::read_join_id(node, document)?;

        Ok(
            Node{
                id:id,
                name:name,
                matrix:matrix,
                joined:joined,
            }
        )
    }

    fn read_join_id(node:&Element, document:&mut Document) -> Result<JoinedTo,Error>{
        for instance in node.children.iter(){
            if instance.name.as_str()=="instance_geometry" {
                let joined_id=instance.get_attribute("url")?.trim_left_matches('#');

                match document.geometries.get(joined_id){
                    Some( geometry ) => return Ok( JoinedTo::Geometry(geometry.clone()) ),
                    None => return Err( Error::Other( format!("Geometry \"{}\" does not exists",joined_id)) ),
                }
            }else if instance.name.as_str()=="instance_camera" {
                let joined_id=instance.get_attribute("url")?.trim_left_matches('#');

                match document.cameras.get(joined_id){
                    Some( camera ) => return Ok( JoinedTo::Camera(camera.clone()) ),
                    None => return Err( Error::Other( format!("Camera \"{}\" does not exists",joined_id)) ),
                }
            }
        }

        Err(Error::NoElement{
            element_name:node.name.clone(),
            child_element_name:String::from("instance"),
        })
    }

    pub fn print_tree(&self, last_scene:bool, last_node:bool){
        use print_branch;
        use print_tab;

        print_tab(true);
        print_tab(last_scene);
        print_branch(last_node);

        let joined=match self.joined{
            JoinedTo::Geometry( ref geometry ) => format!("Geometry with id:\"{}\"", geometry.id),
            JoinedTo::Camera( ref camera ) => format!("Camera with id:\"{}\"", camera.id),
        };

        println!("Source id:\"{}\" name:\"{}\" joided to {}",self.id,self.name,joined);
    }
}
