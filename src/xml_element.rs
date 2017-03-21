use xmltree::Element;
use Error;
use StringExt;

pub trait XMLElement{
    fn get_attribute(&self,name:&str) -> Result<&String,Error>;
    fn get_element(&self,name:&str) -> Result<&Element,Error>;
    fn get_text(&self) -> Result<&String,Error>;
    fn parse_text_as_f32(&self,name:&str) -> Result<f32,Error>;
    fn parse_text_as_usize(&self,name:&str) -> Result<usize,Error>;
    fn parse_attribute_as_f32(&self,name:&str) -> Result<f32,Error>;
    fn parse_attribute_as_usize(&self,name:&str) -> Result<usize,Error>;
}

impl XMLElement for Element{
    fn get_attribute(&self,name:&str) -> Result<&String,Error>{
        match self.attributes.get(name){
            Some(attr) => Ok(attr),
            None => Err(
                Error::NoAttribute{
                    element_name:self.name.clone(),
                    attrib_name:String::from(name),
                }
            ),
        }
    }

    fn get_element(&self,name:&str) -> Result<&Element,Error>{
        let mut found_element=None;

        for element in self.children.iter(){
            if element.name.as_str()==name {
                if found_element.is_some() {
                    return Err( Error::Other(format!("Element \"{}\" contains several elements \"{}\" but just one has been expected", &self.name, name)) );
                }

                found_element=Some(element);
            }
        }

        match found_element{
            Some(element) => Ok(element),
            None => Err(
                Error::NoElement{
                    element_name:self.name.clone(),
                    child_element_name:String::from(name),
                }
            ),
        }
    }

    fn get_text(&self) -> Result<&String,Error>{
        match self.text{
            Some(ref text) => Ok(text),
            None => Err(
                Error::NoText{
                    element_name:self.name.clone(),
                }
            ),
        }
    }

    fn parse_text_as_f32(&self,name:&str) -> Result<f32,Error>{
        self.get_element(name)?.get_text()?.as_str().parse_as_f32(name)
    }

    fn parse_text_as_usize(&self,name:&str) -> Result<usize,Error>{
        self.get_element(name)?.get_text()?.as_str().parse_as_usize(name)
    }

    fn parse_attribute_as_f32(&self,name:&str) -> Result<f32,Error>{
        self.get_attribute(name)?.as_str().parse_as_f32(name)
    }

    fn parse_attribute_as_usize(&self,name:&str) -> Result<usize,Error>{
        self.get_attribute(name)?.as_str().parse_as_usize(name)
    }
}
