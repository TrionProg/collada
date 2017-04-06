use Error;
use XMLElement;
use xmltree::Element;

use std::collections::HashMap;
use std::collections::hash_map::Entry;
use std::sync::Arc;

use Asset;
use Source;
use Matrix;
use ArrayIter;
use Bone;
use TreePrinter;

use source::read_sources;
use source::select_sources;

use std::fmt::Display;
use std::fmt;

pub struct BonesPerVertex{
    pub first_bone_index:usize,
    pub bones_count:usize,
}

pub struct Skin {
    pub id:String,
    pub geometry_id:String,
    pub sources:Vec<(String,Arc<Source>)>,
    pub additional_sources:HashMap<String,Arc<Source>>,
    pub bone_indices:HashMap<String,Arc<BoneIndices>>,
}

pub struct BoneIndices{
    pub source:Arc<Source>,
    pub indices:Vec<usize>,
}

impl Display for Skin{
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Skin id:\"{}\" for geometry with id \"{}\"", self.id, self.geometry_id)
    }
}

impl Skin {
    pub fn parse(skin_element:&Element, id:String, asset:&Asset) -> Result<Self, Error> {
        let geometry_id=skin_element.get_attribute("source")?.trim_left_matches('#').to_string();

        let matrix=Matrix::parse( skin_element.get_element("bind_shape_matrix")?.get_text()? )?;

        let all_sources=read_sources(skin_element, asset)?;

        let vertex_weight_element=skin_element.get_element("vertex_weights")?;
        let joints_element=skin_element.get_element("joints")?;

        let vertices_count=vertex_weight_element.parse_attribute_as_usize("count")?;

        let sources=select_sources(&vertex_weight_element,&all_sources)?;
        let additional_sources=Self::select_additional_sources(&joints_element,&all_sources)?;

        let (bones_count_per_vertex,bones_indices_count)=Self::read_bones_count_per_vertex(&vertex_weight_element)?;
        let bone_indices=Self::read_bone_indices(&vertex_weight_element, bones_indices_count, &sources)?;

        let skin=Skin{
            id:id,
            geometry_id:geometry_id,
            sources:sources,
            additional_sources:additional_sources,
            bone_indices:bone_indices,
        };

        Ok( skin )
    }

    fn read_bones_count_per_vertex(vertex_weight_element:&Element) -> Result<(Vec<BonesPerVertex>,usize),Error>{//read polygons(<vcount> tag)
        let vertices_count=vertex_weight_element.parse_attribute_as_usize("count")?;
        let vertices_bone_count=vertex_weight_element.get_element("vcount")?.get_text()?;

        let mut vertices=Vec::with_capacity(vertices_count);
        let mut bones_indices_count=0;

        let mut array_iter=ArrayIter::new(vertices_bone_count, vertices_count, "polygons");

        for i in 0..vertices_count {
            let bones_per_vertex=array_iter.read_usize()?;

            vertices.push(
                BonesPerVertex{
                    first_bone_index:bones_indices_count,
                    bones_count:bones_per_vertex,
                }
            );

            bones_indices_count+=bones_per_vertex;
        }

        Ok((vertices,bones_indices_count))
    }

    fn read_bone_indices(vertex_weight_element:&Element, bones_indices_count:usize, sources:&Vec<(String,Arc<Source>)>) -> Result<HashMap<String,Arc<BoneIndices>>,Error>{//read vertices(<p> tag)
        let sources_count=sources.len();

        let source_data_indices_per_bone=vertex_weight_element.get_element("v")?.get_text()?;

        let mut bone_indices_indices=Vec::with_capacity(sources_count);
        for i in 0..sources_count{
            bone_indices_indices.push(Vec::with_capacity(bones_indices_count));
        }

        let mut array_iter=ArrayIter::new(source_data_indices_per_bone, bones_indices_count*sources_count, "bone indices");

        for i in 0..bones_indices_count {
            for j in 0..sources_count {
                let data_index_per_bone=array_iter.read_usize()?;

                bone_indices_indices[j].push(data_index_per_bone);
            }
        }

        let mut bone_indices=HashMap::new();

        for &(ref source_name, ref source) in sources.iter().rev(){
            match bone_indices.entry(source_name.clone()){
                Entry::Occupied(_) => return Err(Error::Other( format!("Duplicate source with name \"{}\"",source_name) )),
                Entry::Vacant(entry) => {
                    let bi=BoneIndices{
                        source:source.clone(),
                        indices:bone_indices_indices.pop().unwrap(),
                    };

                    entry.insert( Arc::new(bi) );
                },
            }
        }

        Ok(bone_indices)
    }

    fn select_additional_sources(element:&Element, sources:&HashMap<String,Arc<Source>>) -> Result<HashMap<String,Arc<Source>>,Error>{
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
                        return Err( Error::Other(format!("Duplicate bone additional source with name \"{}\"",source_semantic)) ),
                    Entry::Vacant(entry) => {
                        entry.insert(source.clone());
                    },
                }
            }
        }

        Ok( sources_list )
    }

    pub fn print(&self, printer:TreePrinter) {
        println!("Skin id:\"{}\" for geometry with id \"{}\"", self.id, self.geometry_id);

        self.print_additional_sources( printer.new_branch(false) );
        self.print_bones_indices( printer.new_branch(true) );
    }

    fn print_additional_sources(&self, printer:TreePrinter) {
        println!("Additional sources");

        for (last,(source_name,source)) in self.additional_sources.iter().clone().enumerate().map(|i| (i.0==self.additional_sources.len()-1,i.1) ){
            printer.new_branch(last);
            println!("Source name:\"{}\" id:\"{}\"",source_name,source.id);
        }
    }

    fn print_bones_indices(&self, printer:TreePrinter) {
        println!("Bones");

        for (last,(ref name,ref bone_indices)) in self.bone_indices.iter().clone().enumerate().map(|i| (i.0==self.bone_indices.len()-1,i.1) ){
            printer.new_branch(last);
            println!("Bone indices for \"{}\" source id:\"{}\"",name,bone_indices.source.id);
        }
    }

    /*

    fn get_sources_order(sources:&Vec<(String,Arc<Source>)>) -> Vec<SourceOrder> {
        let mut sources=select_sources(&vertex_weight_element,&all_sources)?;

        let mut sources_order=Vec::with_capacity(sources.len());
        for &(ref source_name, _) in sources.iter() {
            let source_order=match source_name.as_str() {
                "JOINT" => SourceOrder::BoneIndex,
                "WEIGHT" => SourceOrder::Weight,
                _ => SourceOrder::Ignore,
            };

            sources_order.push(source_order);
        }

        sources_order
    }

    fn read_bones_per_vertex(vertex_weight_element:&Element, vertices_count:usize, sources:&Vec<(String,Arc<Source>)>) -> Result<Vec<Vec<(usize,f32)>>, Error>{//read vertices(<p> tag)
        let sources_count=sources.len();

        let data=vertex_weight_element.get_element("v")?.get_text()?;

        let mut bones_per_vertex=Vec::with_capacity(max_bones_count_per_vertex);
        for i in 0..max_bones_count_per_vertex {
            bones_per_vertex.push(Vec::with_capacity(vertices_count));
        }

        let mut array_iter=ArrayIter::new(source_data_indices_per_vertex, vertices_count*sources_count, "bones per vertexes");

        for i in 0..vertices_count {
            for j in 0..bones_count_per_vertex[i] {
                let mut bone_index=0;
                let mut bone_weight=0.0;

                for (source,source_order) in 0..sources.iter().zip(sources_order.iter()) {
                    let index=array_iter.read_usize()?;

                    match *source_order {
                        SourceOrder::BoneIndex => bone_index=i,
                        SourceOrder::BoneWeight => {
                            let weight_layer=match source.layers.get("WEIGHT") {
                                Some( ref weight_layer ) => {
                                    match *weight_layer {
                                        SourceLayer::F32 ( ref wl ) => wl,
                                        _ => return Err( Error::Other("layer WEIGHT of bone weight source must have float type") ),
                                    }
                                },
                                None => return Err( Error::Other("Bone weight source must have WEIGHT layer") ),
                            };

                            bone_weight = weight_layer[i];
                        },
                        SourceOrder::Ignore => {},
                    }
                }

                bones_per_vertex[j].push( (bone_index, bone_weight) );
            }

            for j in bones_count_per_vertex[i]..max_bones_count_per_vertex {
                bones_per_vertex[j].push( (0, 0.0) );
            }
        }

        Ok(bones_per_vertex)
    }

    fn get_bone_names_and_matrixes(sources:&HashMap<String,Arc<Source>>, additional_sources:&HashMap<String,Arc<Source>>) -> Result<Vec<(String,Matrix)>,Error> {
        let bone_names=match sourses.get("JOINT") {
            Some( ref source ) => {
                match source.layers.get("JOINT") {
                    Some( ref weight_layer ) => {
                        match *weight_layer {
                            SourceLayer::Name ( ref wl ) => wl,
                            _ => return Err( Error::Other("layer WEIGHT of bone weight source must have float type") ),
                        }
                    },
                    None => return Err( Error::Other("Bone weight source must have WEIGHT layer") ),
                }
            },
            None => return Err( Error::Other("Bone weight source must have WEIGHT layer") ),
        };

        let bone_matrixes=match source.layers.get("JOINT") {
            Some( ref weight_layer ) => {
                match *weight_layer {
                    SourceLayer::F32 ( ref wl ) => wl,
                    _ => return Err( Error::Other("layer WEIGHT of bone weight source must have float type") ),
                }
            },
            None => return Err( Error::Other("Bone weight source must have WEIGHT layer") ),
        };

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
    */
}

/*
enum SourceOrder {
    BoneIndex,
    BoneWeight,
    Ignore,
}

*/
