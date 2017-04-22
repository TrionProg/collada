use Error;
use XMLElement;
use xmltree::Element;

use std::collections::HashMap;
use std::collections::hash_map::Entry;
use std::sync::Arc;

use Asset;
use Source;
use TreePrinter;

use source::read_sources;

pub struct Animation{
    pub id:String,
    pub bone_id:String,
    pub skeleton_id:String,
    pub keyframes_count:usize,
    pub sources:HashMap<String,Arc<Source>>,
}

impl Animation {
    pub fn parse(animation_element:&Element, asset:&Asset) -> Result<Animation, Error> {
        let animation_id=animation_element.get_attribute("id")?.clone();

        let all_sources=read_sources(animation_element, asset)?;

        let sampler_element=animation_element.get_element("sampler")?;
        let sources=Self::select_sources(&sampler_element,&all_sources)?;
        let keyframes_count=Self::get_keyframes_count(&sources)?;

        let channel_element=animation_element.get_element("channel")?;
        let channel_source=channel_element.get_attribute("source")?.trim_left_matches('#');
        let channel_target=channel_element.get_attribute("target")?;

        let bone_id=match channel_target.find('/') {
            Some( pos ) => {
                let (a,b)=channel_target.split_at(pos);

                String::from( a )
            },
            None => channel_target.clone(),
        };

        let skeleton_id=match channel_source.find( &format!("_{}",&bone_id) ) {
            Some( pos ) => {
                let (a,b)=channel_source.split_at(pos);

                String::from( a )
            },
            None => channel_source.to_string(),
        };

        let animation=Animation{
            id:animation_id,
            bone_id:bone_id,
            skeleton_id:skeleton_id,
            keyframes_count:keyframes_count,
            sources:sources,
        };

        Ok( animation )
    }

    fn select_sources(element:&Element, sources:&HashMap<String,Arc<Source>>) -> Result<HashMap<String,Arc<Source>>,Error>{
        let mut sources_list=HashMap::new();

        for input_element in element.children.iter(){
            if input_element.name.as_str()=="input" {
                let source_semantic=input_element.get_attribute("semantic")?;
                let source_id=input_element.get_attribute("source")?.trim_left_matches('#');

                let source=match sources.get(source_id){
                    Some(s) => s.clone(),
                    None => return Err(Error::Other( format!("Source with id \"{}\" does not exists", source_id) )),
                };

                match sources_list.entry(source_semantic.clone()) {
                    Entry::Occupied(_) =>
                        return Err( Error::Other(format!("Duplicate animation source with name \"{}\"",source_semantic)) ),
                    Entry::Vacant(entry) => {
                        entry.insert(source.clone());
                    },
                }
            }
        }

        if sources.len()==0 {
            return Err( Error::Other( String::from("No sources for animation") ));
        }

        Ok( sources_list )
    }

    fn get_keyframes_count(sources:&HashMap<String,Arc<Source>>) -> Result<usize,Error> {
        let mut keyframes_count=None;

        for (_,source) in sources.iter() {
            for (layer_name, layer) in source.layers.iter() {
                let cnt=layer.get_length();

                match keyframes_count {
                    Some( keyframes_count ) => {
                        if cnt!=keyframes_count {
                            return Err(Error::Other( format!("Layer \"{}\" of source with id \"{}\" has different number of keyframes then others", layer_name, source.id) ));
                        }
                    },
                    None =>
                        keyframes_count=Some(cnt),
                }
            }
        }

        Ok( keyframes_count.unwrap() )
    }


    pub fn print(&self, printer:TreePrinter) {
        println!("Animation id:\"{}\" for bone with id \"{}\" of skeleton with id \"{}\"",self.id, self.bone_id, self.skeleton_id);

        printer.new_branch(false);
        println!("keyframes count: {}", self.keyframes_count);

        self.print_sources( printer.new_branch(true) );
    }

    fn print_sources(&self, printer:TreePrinter) {
        println!("Sources");

        for (last,(source_name,source)) in self.sources.iter().clone().enumerate().map(|i| (i.0==self.sources.len()-1,i.1) ){
            printer.new_branch(last);
            println!("Source name:\"{}\" id:\"{}\"",source_name,source.id);
        }
    }
}

pub fn parse_animations(root:&Element, asset:&Asset) -> Result< HashMap<String,Arc<Animation> >, Error>{
    let animations_element=match root.get_element("library_animations") {
        Ok( animations_element ) => animations_element,
        Err( _ ) => return Ok( HashMap::new() ),
    };

    let mut animations:HashMap<String,Arc<Animation> >=HashMap::new();

    for animation_element in animations_element.children.iter(){
        if animation_element.name.as_str()=="animation" {
            let animation=Animation::parse(animation_element, asset)?;

            match animations.entry(animation.id.clone()){
                Entry::Occupied(_) =>
                    return Err( Error::Other(format!("Duplicate animation with id \"{}\"",animation.id)) ),
                Entry::Vacant(entry) => {
                    entry.insert(Arc::new(animation));
                },
            }
        }
    }

    Ok(animations)
}
