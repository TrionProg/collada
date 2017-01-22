use Error;
use XMLElement;
use xmltree::Element;

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

//TODO:Add info about Camera/Light
//TODO joined_id shoult be pointer to Geometry/Camera/Light? Check if them are exists

pub struct Node{
    pub id:String,
    pub name:String,
    pub matrix:Matrix,
    pub joined_id:String,
}

impl Node{
    pub fn parse(node:&Element) -> Result<Node,Error>{
        let id=node.get_attribute("id")?.clone();
        let name=node.get_attribute("name")?.clone();

        let matrix_str=node.get_element("matrix")?.get_text()?;
        let matrix=Matrix::parse(matrix_str)?;

        let joined_id=Node::read_join_id(node)?;

        Ok(
            Node{
                id:id,
                name:name,
                matrix:matrix,
                joined_id:joined_id,
            }
        )
    }

    fn read_join_id(node:&Element) -> Result<String,Error>{
        for instance in node.children.iter(){
            if instance.name.starts_with("instance") {
                let joined_id=String::from( instance.get_attribute("url")?.trim_left_matches('#') );

                return Ok(joined_id);
            }
        }

        Err(Error::NoElement{
            elementName:node.name.clone(),
            childElementName:String::from("instance"),
        })
    }
}
