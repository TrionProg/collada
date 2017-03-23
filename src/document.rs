use Error;
use XMLElement;
use xmltree::Element;

use Asset;
use Camera;
use Geometry;
use Animation;
use Skin;
use Skeleton;
use Scene;
use TreePrinter;

use std::path::Path;

use std::io::prelude::*;
use std::io::BufReader;
use std::fs::File;

use std::collections::HashMap;
use std::sync::Arc;

use camera::parse_cameras;
use geometry::parse_geometries;
use animation::parse_animations;
use controller::parse_controllers;
use scene::parse_scenes;

pub struct Document{
    pub asset:Asset,
    pub cameras:HashMap<String,Arc<Camera>>,
    pub geometries:HashMap<String,Arc<Geometry>>,
    pub skins:HashMap<String,Arc<Skin>>,
    pub animations:HashMap<String,Arc<Animation>>,
    pub skeletons:HashMap<String,Arc<Skeleton>>,
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

        let version=root.get_attribute("version")?;
        let asset=Asset::parse(&root)?;

        let cameras=parse_cameras(&root)?;
        let geometries=parse_geometries(&root, &asset)?;
        let animations=parse_animations(&root, &asset)?;
        let (skins, skins_by_id)=parse_controllers(&root, &asset)?;

        let mut document=Document{
            asset:asset,
            cameras:cameras,
            geometries:geometries,
            animations:animations,
            skins:skins,
            skeletons:HashMap::new(),
            scenes:HashMap::new(),
        };

        parse_scenes(&root, &mut document, skins_by_id)?;

        Ok(document)
    }

    pub fn print(&self){
        let mut printer=TreePrinter::new();
        println!("Document");

        self.print_geometries( printer.new_branch(false) );
        self.print_skeletons( printer.new_branch(false) );
        self.print_animations( printer.new_branch(false) );
        self.print_skins( printer.new_branch(false) );
        self.print_scenes( printer.new_branch(true) );
    }

    fn print_geometries(&self, printer:TreePrinter) {
        println!("Geometries");

        for (last,(_,geometry)) in self.geometries.iter().clone().enumerate().map(|i| (i.0==self.geometries.len()-1,i.1) ){
            geometry.print( printer.new_branch(last) );
        }
    }

    fn print_skeletons(&self, printer:TreePrinter) {
        println!("Skeletons");

        for (last,(_,skeleton)) in self.skeletons.iter().clone().enumerate().map(|i| (i.0==self.skeletons.len()-1,i.1) ){
            skeleton.print( printer.new_branch(last) );
        }
    }

    fn print_animations(&self, printer:TreePrinter) {
        println!("Animations");

        for (last,(_,animation)) in self.animations.iter().clone().enumerate().map(|i| (i.0==self.animations.len()-1,i.1) ){
            animation.print( printer.new_branch(last) );
        }
    }

    fn print_skins(&self, printer:TreePrinter) {
        println!("Skins");

        for (last,(_,skin)) in self.skins.iter().clone().enumerate().map(|i| (i.0==self.skins.len()-1,i.1) ){
            skin.print( printer.new_branch(last) );
        }
    }

    fn print_scenes(&self, printer:TreePrinter) {
        println!("Scenes");

        for (last,(_,scene)) in self.scenes.iter().clone().enumerate().map(|i| (i.0==self.scenes.len()-1,i.1) ){
            scene.print( printer.new_branch(last) );
        }
    }

}
