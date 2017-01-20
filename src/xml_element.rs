use xmltree::Element;
use Error;

pub trait XMLElement{
    fn get_attribute(&self,name:&str) -> Result<&String,Error>;
    fn get_element(&self,name:&str) -> Result<&Element,Error>;
    fn get_text(&self) -> Result<&String,Error>;
    fn parse_text_as_f32(&self,name:&str) -> Result<f32,Error>;
}

impl XMLElement for Element{
    fn get_attribute(&self,name:&str) -> Result<&String,Error>{
        match self.attributes.get(name){
            Some(attr) => Ok(attr),
            None => Err(
                Error::NoAttribute{
                    elementName:self.name.clone(),
                    attribName:String::from(name),
                }
            ),
        }
    }

    fn get_element(&self,name:&str) -> Result<&Element,Error>{
        match self.get_child(name){
            Some(element) => Ok(element),
            None => Err(
                Error::NoElement{
                    elementName:self.name.clone(),
                    childElementName:String::from(name),
                }
            ),
        }
    }

    fn get_text(&self) -> Result<&String,Error>{
        match self.text{
            Some(ref text) => Ok(text),
            None => Err(
                Error::NoText{
                    elementName:self.name.clone(),
                }
            ),
        }
    }

    fn parse_text_as_f32(&self,name:&str) -> Result<f32,Error>{
        match self.get_element(name)?.get_text()?.parse::<f32>(){
            Ok(r) => Ok(r),
            Err(e) => Err(Error::StringParseError( format!("Can not parse {} as f32",name) )),
        }
    }
}
