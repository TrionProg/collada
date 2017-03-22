use Error;
use XMLElement;
use xmltree::Element;

use std::collections::HashMap;
use std::collections::hash_map::Entry;
use std::sync::Arc;

use Asset;
use Source;
use Matrix;

use source::read_sources;

pub struct Animation{
    animation_id:String,
    bone_id:String,
    skeleton_id:String,
    sources:HashMap<String,Arc<Source>>,
}

impl Animation {
    pub fn parse(animation_element:&Element, asset:&Asset) -> Result<Animation, Error> {
        let animation_id=animation_element.get_attribute("id")?.clone();

        let all_sources=read_sources(animation_element, asset)?;

        let sampler_element=animation_element.get_element("sampler")?;
        let sources=Self::select_sources(&sampler_element,&all_sources)?;

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

        println!("anim: {} {}",bone_id, skeleton_id);

        let animation=Animation{
            animation_id:animation_id,
            bone_id:bone_id,
            skeleton_id:skeleton_id,
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

        Ok( sources_list )
    }
}

pub fn parse_animations(root:&Element, asset:&Asset) -> Result< HashMap<String,Arc<Animation> >, Error>{
    let animations_element=root.get_element("library_animations")?;

    let mut animations:HashMap<String,Arc<Animation> >=HashMap::new();

    for animation_element in animations_element.children.iter(){
        if animation_element.name.as_str()=="animation" {
            let animation=Animation::parse(animation_element, asset)?;

            match animations.entry(animation.animation_id.clone()){
                Entry::Occupied(_) =>
                    return Err( Error::Other(format!("Duplicate animation with id \"{}\"",animation.animation_id)) ),
                Entry::Vacant(entry) => {
                    entry.insert(Arc::new(animation));
                },
            }
        }
    }

    Ok(animations)
}
