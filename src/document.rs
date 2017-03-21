use Error;
use XMLElement;
use xmltree::Element;

use Asset;
use Camera;
use Geometry;
//use Skin;
use Scene;

use std::path::Path;

use std::io::prelude::*;
use std::io::BufReader;
use std::fs::File;

use std::collections::HashMap;
use std::sync::Arc;

use camera::parse_cameras;
use geometry::parse_geometries;
//use controller::parse_controllers;
use scene::parse_scenes;

pub struct Document{
    pub asset:Asset,
    pub cameras:HashMap<String,Arc<Camera>>,
    pub geometries:HashMap<String,Arc<Geometry>>,
    //pub skins:HashMap<String,Vec<Arc<Skin>> >,
    pub scenes:HashMap<String,Arc<Scene>>,
}

impl Document{
    pub fn parse(file_name:&Path) -> Result<Document,Error>{
        let file=match File::open(file_name){
            Ok(f) => f,
            Err(e) => {
                let file_name_str=match file_name.to_str(){
                    Some( fns ) => String::from(fns),
                    None => return Err( Error::NotUnicodeFileName ),
                };

                return Err(Error::FileError(file_name_str,e));
            },
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
        let geometries=parse_geometries(&root, &asset)?;
        //let skins=parse_controllers(&root, &asset)?;

        println!("------");

        let mut document=Document{
            asset:asset,
            cameras:cameras,
            geometries:geometries,
            //skins:skins,
            scenes:HashMap::new(),
        };

        parse_scenes(&root, &mut document)?;

        Ok(document)
    }

    pub fn print_tree(&self){
        use print_branch;

        println!("Document");

        print_branch(false);
        println!("Geometries");

        if self.geometries.len()>1{
            for (_,geometry) in self.geometries.iter().take(self.geometries.len()-1){
                geometry.print_tree(false);
            }
        }

        match self.geometries.iter().last(){
            Some((_,geometry)) => geometry.print_tree(true),
            None => {},
        }

        print_branch(true);
        println!("Scenes");

        if self.scenes.len()>1{
            for (_,scene) in self.scenes.iter().take(self.scenes.len()-1){
                scene.print_tree(false);
            }
        }

        match self.scenes.iter().last(){
            Some((_,scene)) => scene.print_tree(true),
            None => {},
        }
    }

}

/*
pub enum DocumentError{
    FileError(std::io::Error),
    XMLParseError(xmltree::ParseError),
    Asset(AssetError),
    Geometry(GeometryError),
    Camera(CameraError),
}
*/
