use xmltree::Element;
use Error;

pub trait XMLElement{
    fn get_attribute(&self,name:&str) -> Result<&String,Error>;
    fn get_element(&self,name:&str) -> Result<&Element,Error>;
    fn get_text(&self) -> Result<&String,Error>;
    fn parse_text_as_f32(&self,name:&str) -> Result<f32,Error>;
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
        match self.get_child(name){ //TODO:multiple??
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
        match self.get_element(name)?.get_text()?.parse::<f32>(){
            Ok(v) => Ok(v),
            Err(_) => Err(Error::StringParseError( format!("Can not parse {} as f32",name) )),
        }
    }

    fn parse_attribute_as_usize(&self,name:&str) -> Result<usize,Error>{
        match self.get_attribute(name)?.parse::<usize>(){
            Ok(v) => Ok(v),
            Err(_) => Err(Error::StringParseError( format!("Can not parse {} as usize",name) )),
        }
    }
}
