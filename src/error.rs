use std;
use xmltree;
use std::fmt::Display;
use std::fmt;

pub enum Error{
    FileError(std::io::Error),
    ParseError(xmltree::ParseError),
    NoAttribute{element_name:String, attrib_name:String},
    NoElement{element_name:String, child_element_name:String},
    NoText{element_name:String},
    StringParseError(String),
    Other(String),
}


impl Display for Error{
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self{
            Error::FileError(ref e) => write!(f, "File error:{}", e),
            Error::ParseError(ref e) => write!(f, "Parse error:{}", e),
            Error::NoAttribute{ref element_name, ref attrib_name} => write!(f, "Element \"{}\" has not attrib \"{}\"", element_name, attrib_name),
            Error::NoElement{ref element_name, ref child_element_name} => write!(f, "Element \"{}\" does not contains element \"{}\"", element_name, child_element_name),
            Error::NoText{ref element_name} => write!(f, "Element \"{}\" does not contains text between <x> and </x>", element_name),
            Error::StringParseError(ref message) => write!(f, "String parse error: {}", message),
            Error::Other(ref message) => write!(f, "{}", message),
        }
    }
}
