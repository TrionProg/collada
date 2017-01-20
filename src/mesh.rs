use Error;
use XMLElement;
use xmltree::Element;

use Source;

pub struct Mesh{
    //sources:Vec<Source>,
}

impl Mesh{
    pub fn parse(mesh:&Element) -> Result<Mesh,Error>{
        Ok(Mesh{})
    }
}
