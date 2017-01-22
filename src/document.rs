use Error;
use XMLElement;
use xmltree::Element;

use Asset;
use Camera;

use std::path::Path;

use std::io::prelude::*;
use std::io::BufReader;
use std::fs::File;

use std::collections::HashMap;

use camera::parse_cameras;
use geometry::parse_geometries;
use scene::parse_scenes;

pub struct Document{

}

impl Document{
    pub fn parse(file_name:&Path) -> Result<Document,Error>{
        let file=match File::open(file_name){
            Ok(f) => f,
            Err(e) => return Err(Error::FileError(e)),
        };

        let mut reader = BufReader::new(file);

        let mut root = match Element::parse(reader){
            Ok(r) => r,
            Err(e) => return Err(Error::ParseError(e)),
        };

        for e in root.children.iter(){
            println!("{}",e.name);
        }

        for (n,a) in root.attributes.iter(){
            println!("{} {}",n,a);
        }

        //let version=root.get_attribute("version")?;
        let asset=Asset::parse(&root)?;

        let cameras=parse_cameras(&root)?;
        let geometries=parse_geometries(&root)?;
        let scenes=parse_scenes(&root)?;

        Ok(Document{})
    }
}
