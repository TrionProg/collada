use std;
use xmltree;
use std::fmt::Display;
use std::fmt;

pub enum Error{
    FileError(std::io::Error),
    ParseError(xmltree::ParseError),
    NoAttribute{elementName:String, attribName:String},
    NoElement{elementName:String, childElementName:String},
    NoText{elementName:String},
    StringParseError(String),
    Other(String),
}


impl Display for Error{
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self{
            Error::FileError(ref e) => write!(f, "File error:{}", e),
            Error::ParseError(ref e) => write!(f, "Parse error:{}", e),
            Error::NoAttribute{ref elementName, ref attribName} => write!(f, "Element \"{}\" has not attrib \"{}\"", elementName, attribName),
            Error::NoElement{ref elementName, ref childElementName} => write!(f, "Element \"{}\" does not contains element \"{}\"", elementName, childElementName),
            Error::NoText{ref elementName} => write!(f, "Element \"{}\" does not contains text between <x> and </x>", elementName),
            Error::StringParseError(ref message) => write!(f, "String parse error: {}", message),
            Error::Other(ref message) => write!(f, "{}", message),
        }
    }
}
