use Error;
use XMLElement;
use xmltree::Element;

use Asset;
use Camera;
use Geometry;
use Scene;

use std::path::Path;

use std::io::prelude::*;
use std::io::BufReader;
use std::fs::File;

use std::collections::HashMap;
use std::rc::Rc;

use camera::parse_cameras;
use geometry::parse_geometries;
use scene::parse_scenes;

pub struct Document{
    pub asset:Asset,
    pub cameras:HashMap<String,Rc<Camera>>,
    pub geometries:HashMap<String,Rc<Geometry>>,
    pub scenes:HashMap<String,Rc<Scene>>,
}

impl Document{
    pub fn parse(file_name:&Path) -> Result<Document,Error>{
        let file=match File::open(file_name){
            Ok(f) => f,
            Err(e) => return Err(Error::FileError(e)),
        };

        let reader = BufReader::new(file);

        let root = match Element::parse(reader){
            Ok(r) => r,
            Err(e) => return Err(Error::ParseError(e)),
        };

        for e in root.children.iter(){
            println!("{}",e.name);
        }

        //let version=root.get_attribute("version")?;
        let asset=Asset::parse(&root)?;

        let cameras=parse_cameras(&root)?;
        let geometries=parse_geometries(&root)?;

        let mut document=Document{
            asset:asset,
            cameras:cameras,
            geometries:geometries,
            scenes:HashMap::new(),
        };

        parse_scenes(&root, &mut document)?;

        Ok(document)
    }
}
