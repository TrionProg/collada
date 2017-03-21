use Error;
use XMLElement;
use xmltree::Element;

use std::collections::HashMap;
use std::collections::hash_map::Entry;
use std::sync::Arc;

use Asset;
use Matrix4;

pub struct Skin {
    geometry_id:String,
    animation_id:String,
    bone_names:Vec<String>,
    bone_matrixes:Vec<Martix4>,
    bones_per_vertex:Vec<Vec<(usize,f32)>>,
}

impl Skin {
    pub fn parse(skin_element:&Element, animation_id:String, asset:&Asset) -> Result<Self, Error> {
        let geometry_id=skin_element.get_attribute("source")?.trim_left_matches('#').to_string();

        let matrix=Matrix::parse( bone_element.get_element("bind_shape_matrix")?.get_text()?, &document.asset )?;

        let (src_names_of_bones, src_weights, src_matrixes)=Self::read_sources(skin_element, asset)?;

        let vertex_weight_element=skin_element.get_element("vertex_weights")?;
        let vertices_count=vertex_weight_element.get_attribute_as_usize("count")?;

        let mut sources_order=Self::read_sources_order(vertex_weight_element)?;
        let (bones_count_per_vertex,max_bones_count_per_vertex)=Self::read_bones_count_per_vertex(vertex_weight_element, vertices_count)?;

        let mut bones_per_vertex=Vec::with_capacity(max_bones_count_per_vertex);
        for i in 0..max_bones_count_per_vertex {
            bones_per_vertex.push(Vec::with_capacity(vertices_count));
        }

        let data=vertex_weight_element.get_element("v")?.get_text()?;
        let mut data_iter=data.split(' ').filter(|n|*n!="");

        for i in 0..vertices_count {
            for j in 0..bones_count_per_vertex[i] {
                let mut bone_index=0;
                let mut bone_weight=0.0;

                for source_order in 0..sources_order {
                    let v=data_iter.next() {
                        Some( v ) => v,
                        None => return Err(Error::Other( String::from("not all values of bone data have been read") )),
                    };

                    let index=match v.parse::<usize>(){
                        Ok ( i ) => i,
                        Err( _ ) => return Err(Error::ParseIntError( String::from("bone data value"), String::from(v)) ),
                    };

                    match *source_order {
                        SourceOrder::BoneIndex => bone_index=i,
                        SourceOrder::BoneWeight => bone_weight=src_weights[i],
                        SourceOrder::Ignore => {},
                    }
                }

                bones_per_vertex[j].push( (bone_index, bone_weight) );
            }

            for j in bones_count_per_vertex[i]..max_bones_count_per_vertex {
                bones_per_vertex[j].push( (0, 0.0) );
            }
        }

        let skin=Skin{
            geometry_id:geometry_id;
            animation_id:animation_id;
            bone_names:src_names_of_bones,
            bone_matrixes:src_matrixes,
            bones_per_vertex:bones_per_vertex,
        };

        Ok( skin )
    }

    fn read_sources_order(vertex_weight_element:&Element) -> Result<Vec<SourceOrder>,Error> {
        let mut sources_order=Vec::new();

        for input_element in vertex_weight_element.children.iter() {
            if input_element.name.as_str()=="input" {
                /*
                let offset_str=input_element.get_attribute("offset")?;

                let offset=match offset_str.parse::<usize>(){
                    Ok ( o ) => o,
                    Err( _ ) => return Err(Error::ParseIntError( String::from("Offset of skin"), String::from(offset_str)) ),
                };

                if offset>2 {
                    return Err( Error::Other(String::from("Unknown offset of skin 2")) );
                }
                */

                let source_order=match input_element.get_attribute("semantic")?.as_str() {
                    "JOINT" => SourceOrder::BoneIndex,
                    "WEIGHT" => SourceOrder::Weight,
                    _ => SourceOrder::Ignore,
                };

                sources_order.push(source_order);
            }
        };

        Ok( sources_order )
    }

    fn read_bones_count_per_vertex(vertex_weight_element:&Element, vertices_count:usize) -> Result<(Vec<usize>,usize),Error> {
        let mut bones_count_per_vertex=Vec::with_capacity(vertices_count);

        let mut count=0;

        for (i,n) in text.split(' ').filter(|n|*n!="").take(vertices_count).enumerate(){
            match v.parse::<f32>(){
                Ok ( f ) => weights.push( f ),
                Err( _ ) => return Err(Error::ParseFloatError( String::from("Bone weight"), String::from(v)) ),
            }

            count+=1;
        }

        //check
        if count!=bones_count_per_vertex {
            return Err(Error::Other( format!("Only {} bones count per vertex have been read, expected {}", count, vertices_count) ));
        }

        let mut max_bones_count_per_vertex=0;
        for bcpv in bones_count_per_vertex.iter() {
            if bcpv>max_bones_count_per_vertex {
                max_bones_count_per_vertex=bcpv;
            }
        }

        if max_bones_count_per_vertex==0 {
            return Err(Error::Other( String::from("max bones count per vertex == 0") ) );
        }

        Ok( (bones_count_per_vertex, max_bones_count_per_vertex) )
    }


    fn read_sources(skin_element:&Element, asset:&Asset) -> Result<(Vec<String>,Vec<Matrix4>,Vec<f32>),Error> {
        let mut names_of_bones=None;
        let mut weights=None;
        let mut matrixes=None;

        for source_element in skin_element.children.iter() {
            let accessor=source.get_element("technique_common")?.get_element("accessor")?;
            let accessor_count=accessor.parse_attribute_as_usize("count")?;
            let accessor_stride=accessor.parse_attribute_as_usize("stride")?;

            let param=accessor.get_element("param")?;
            let param_name=param.get_attribute("name")?;
            let param_type=param.get_attribute("type")?;

            match param_name.as_str() {
                "JOINT" => {
                    if param_type.as_str()!="name" {
                        return Err( Error::Other( String::from("JOINT param must have name type") ) );
                    }

                    names_of_bones=Some( Self::read_names_of_bones( source_element.get_element("Name_array")? )? );
                },
                "TRANSFORM" => {
                    if param_type.as_str()!="float4x4" {
                        return Err( Error::Other( String::from("TRANSFORM param must have float4x4 type") ) );
                    }

                    //TODO:read!!

                },
                "WEIGHT" => {
                    if param_type.as_str()!="float" {
                        return Err( Error::Other( String::from("WEIGHT param must have float type") ) );
                    }

                    weights=Some( Self::read_weights( source_element.get_element("float_array")? )? );
                },
                _ => {},
            }
        }

        let mut names_of_bones=match names_of_bones {
            Some( nob ) => nob,
            None => return Err( Error::Other( String::from("no names of bones") ) ),
        };

        let mut weights=match weights {
            Some( w ) => w,
            None => return Err( Error::Other( String::from("no weights of bones") ) ),
        };

        let mut matrixes=match matrixes {
            Some( m ) => m,
            None => return Err( Error::Other( String::from("no matrixes of bones") ) ),
        };

        if names_of_bones.len() != matrixes.len() {
            return Err( Error::Other( format!("count of bones ({}) and count of matrixes ({}) mismatch", names_of_bones.len(), matrixes.len()) ) );
        }

        Ok(names_of_bones,weights,matrixes)
    }

    fn read_names_of_bones(name_array_element:&Element) -> Result<Vec<String>,Error> {
        let names_of_bones_count=name_array_element.parse_attribute_as_usize("count")?;
        let text=name_array_element.get_text()?;

        let mut names_of_bones=Vec::with_capacity(names_of_bones_count);

        let mut count=0;

        for (i,n) in text.split(' ').filter(|n|*n!="").take(names_of_bones_count).enumerate(){
            names_of_bones.push(String::from(n));

            count+=1;
        }

        //check
        if count!=names_of_bones_count {
            return Err(Error::Other( format!("Only {} names of bones have been read, expected {}", count, names_of_bones_count) ));
        }

        Ok( names_of_bones )
    }

    fn read_weights(float_array_element:&Element) -> Result<Vec<String>,Error> {
        let weights_count=float_array_element.parse_attribute_as_usize("count")?;
        let text=float_array_element.get_text()?;

        let mut weights=Vec::with_capacity(weights_count);

        let mut count=0;

        for (i,n) in text.split(' ').filter(|n|*n!="").take(weights_count).enumerate(){
            match v.parse::<f32>(){
                Ok ( f ) => weights.push( f ),
                Err( _ ) => return Err(Error::ParseFloatError( String::from("Bone weight"), String::from(v)) ),
            }

            count+=1;
        }

        //check
        if count!=weights_count {
            return Err(Error::Other( format!("Only {} weights of bones have been read, expected {}", count, weights_count) ));
        }

        Ok( weights )
    }
}

pub fn parse_controllers(root:&Element, asset:&Asset) -> Result< HashMap<String,Vec<Arc<Skin>> >, Error>{
    let controllers_element=root.get_element("library_controllers")?;

    let mut skins=HashMap::new();

    for controller_element in controllers_element.children.iter(){
        if controller_element.name.as_str()=="controller" {
            let animation_id=controller_element.get_attribute("id")?;

            for skin_element in controller_element.children.iter() {
                let skin=Skin::parse(skin_element, animation_id.clone(), asset)?;

                match skins.entry(skin.geometry_id.clone()){
                    Entry::Occupied(entry) =>
                        entry.get_mut().push(Arc::new(skin)),
                    Entry::Vacant(entry) => {
                        let mut skins=Vec::with_capacity(1);
                        skins.push(Arc::new(skin));

                        entry.insert(skins);
                    },
                }
            }
        }
    }

    Ok(skins)
}

enum SourceOrder {
    BoneIndex,
    BoneWeight,
    Ignore,
}
