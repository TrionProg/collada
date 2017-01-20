/*
#[macro_export]
macro_rules! get_attribute{
    ($element:expr, $name:expr) => (
        match $element.attributes.get($name){
            Some(attr) => attr,
            None => return Err(
                Error::NoAttribute{
                    elementName:$element.name.clone(),
                    attribName:String::from($name),
                }
            ),
        }
    )
}

#[macro_export]
macro_rules! get_child{
    ($element:expr, $name:expr) => (
        for child in $element.children.iter(){
            if child.
        match $element.attributes.get($name){
            Some(attr) => attr,
            None => return Err(
                Error::NoAttribute{
                    elementName:$element.name.clone(),
                    attribName:String::from($name),
                }
            ),
        }
    )
}
*/
