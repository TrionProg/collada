use Error;
use XMLElement;
use xmltree::Element;

use std::sync::Arc;
use std::collections::HashMap;
use std::collections::hash_map::Entry;

use Asset;
use Axis;
use Editor;
use ArrayIter;
use Matrix;

#[derive(Clone, Eq, PartialEq)]
pub enum LayerType{
    X,
    Y,
    Z,
    U,
    V,
    R,
    G,
    B,
    BoneName,
    TransformMatrix,
    Weight,
    Other(String),
}

impl LayerType{
    pub fn print_vertex_format(&self) -> & str{
        match *self{
            LayerType::X => "X",
            LayerType::Y => "Y",
            LayerType::Z => "Z",
            LayerType::U => "U",
            LayerType::V => "V",
            LayerType::R => "R",
            LayerType::G => "G",
            LayerType::B => "B",
            LayerType::BoneName => "bone_name",
            LayerType::TransformMatrix => "transform_matrix",
            LayerType::Weight => "weight",
            LayerType::Other(ref s) => s.as_str(),
        }
    }
}

#[derive(Copy, Clone)]
pub enum DataType{
    F32,
    I32,
    Name,
    Matrix4,
}

impl DataType{
    pub fn print_data_type(&self) -> &'static str{
        match *self{
            DataType::F32 => "f32",
            DataType::I32 => "i32",
            DataType::Name => "name",
            DataType::Matrix4 => "matrix4",
        }
    }

    pub fn get_size(&self) -> usize{
        match *self{
            DataType::Matrix4 => 16,
            _ => 1,
        }
    }
}

pub enum SourceLayer{
    F32(Vec<f32>),
    I32(Vec<i32>),
    Name(Vec<String>),
    Matrix4(Vec<Matrix>),
}

impl SourceLayer{
    pub fn print_data_type(&self) -> &'static str{
        match *self{
            SourceLayer::F32(_) => "f32",
            SourceLayer::I32(_) => "i32",
            SourceLayer::Name(_) => "name",
            SourceLayer::Matrix4(_) => "matrix4",
        }
    }

    pub fn get_length(&self) -> usize {
        match *self{
            SourceLayer::F32( ref list ) => list.len(),
            SourceLayer::I32( ref list ) => list.len(),
            SourceLayer::Name( ref list ) => list.len(),
            SourceLayer::Matrix4( ref list ) => list.len(),
        }
    }
}

pub struct Source{
    pub id:String,
    pub short_vertex_format:String,
    pub vertex_format:String,
    pub layers:HashMap<String,SourceLayer>,
}

impl Source{
    pub fn parse(source:&Element, asset:&Asset) -> Result<Source,Error>{
        let id=source.get_attribute("id")?.clone();

        let accessor=source.get_element("technique_common")?.get_element("accessor")?;
        let accessor_count=accessor.parse_attribute_as_usize("count")?;
        let accessor_stride=accessor.parse_attribute_as_usize("stride")?;

        let params=Self::read_params(&accessor, &id, asset)?;
        let (vertex_format,short_vertex_format)=Self::generate_vertex_format(&params);

        let (array,array_size) = Self::get_array_and_size(source)?;

        let mut layers=Self::read_layers_data(
            accessor_stride, accessor_count, array, array_size, &params, asset
        )?;

        Ok(
            Source{
                id:id,
                short_vertex_format:short_vertex_format,
                vertex_format:vertex_format,
                layers:layers,
            }
        )
    }

    fn get_array_and_size(source:&Element) -> Result<(&String, usize),Error> {
        for data_element in source.children.iter() {
            if data_element.name.ends_with("_array") {
                let data=data_element.get_text()?;
                let data_size=data_element.parse_attribute_as_usize("count")?;

                return Ok( (data,data_size) );
            }
        }

        Err( Error::Other(String::from("Source has no data(*_array)")) )
    }

    fn read_params(accessor:&Element, id:&String, asset:&Asset) -> Result<Vec<(LayerType,LayerType,DataType)>,Error>{
        let mut params=Vec::with_capacity(4);

        for param_element in accessor.children.iter(){
            if param_element.name.as_str()=="param" {
                let param_name_str=param_element.get_attribute("name")?.as_str();
                let param_name=match param_name_str{
                    "X" => LayerType::X,
                    "Y" => LayerType::Y,
                    "Z" => LayerType::Z,
                    "S" => LayerType::U,
                    "T" => LayerType::V,
                    "R" => LayerType::R,
                    "G" => LayerType::G,
                    "B" => LayerType::B,
                    "JOINT" => LayerType::BoneName,
                    "TRANSFORM" => LayerType::TransformMatrix,
                    "WEIGHT" => LayerType::Weight,
                    _ => LayerType::Other(String::from(param_name_str)),
                };

                let standard_layer_type=Self::get_standard_layer_type(&param_name, asset);

                let param_data_type_str=param_element.get_attribute("type")?.as_str();
                let param_type=match param_data_type_str{
                    "float" => DataType::F32,
                    "name" => DataType::Name,
                    "float4x4" => DataType::Matrix4,
                    _ => return Err(Error::Other( format!("Expected float,name or float4x4 but {} has been found",param_data_type_str) )),
                };

                params.push((param_name, standard_layer_type, param_type));
            }
        }

        if params.len()==0 {
            return Err(Error::Other( format!("Source \"{}\" is empty", id) ));
        }

        Ok(params)
    }

    fn get_standard_layer_type(layer_type:&LayerType, asset:&Asset) -> LayerType {
        match *layer_type {
            LayerType::X => {
                match asset.up_axis {
                    Axis::X => LayerType::Y,
                    Axis::Y => LayerType::X,
                    Axis::Z => LayerType::X,
                }
            },
            LayerType::Y => {
                match asset.up_axis {
                    Axis::X => LayerType::X,
                    Axis::Y => LayerType::Y,
                    Axis::Z => LayerType::Z,
                }
            },
            LayerType::Z => {
                match asset.up_axis {
                    Axis::X => LayerType::Z,
                    Axis::Y => LayerType::Z,
                    Axis::Z => LayerType::Y,
                }
            },
            _ => layer_type.clone()
        }
    }

    fn generate_vertex_format(params:&Vec<(LayerType,LayerType,DataType)>) -> (String,String) {
        let mut vertex_format=String::new();
        let mut short_vertex_format=String::new();

        for &(ref param_name,_,ref param_type) in params.iter().take(params.len()-1){
            vertex_format.push_str( &format!("{}:{},",param_name.print_vertex_format(), param_type.print_data_type()) );
            short_vertex_format.push_str( &format!("{},",param_name.print_vertex_format()) );
        }

        let &(ref param_name,_,ref param_type)=match params.iter().last(){
            Some(p) => p,
            None => {unreachable!()},
        };

        vertex_format.push_str( &format!("{}:{}",param_name.print_vertex_format(), param_type.print_data_type()) );
        short_vertex_format.push_str( &format!("{}",param_name.print_vertex_format()) );

        (vertex_format,short_vertex_format)
    }

    fn read_layers_data(
        accessor_stride:usize,
        accessor_count:usize,
        array:&String,
        array_size:usize,
        params:&Vec<(LayerType,LayerType,DataType)>,
        asset:&Asset
    ) -> Result<HashMap<String,SourceLayer>,Error> {
        let mut stride=0;
        for &(_,_,ref data_type) in params.iter() {
            stride+=data_type.get_size();
        }

        if accessor_stride!=stride{
            return Err(Error::Other( format!("stride({})!=calculated stride({})", accessor_stride, stride) ));
        }

        if accessor_count*accessor_stride!=array_size{
            return Err(Error::Other( format!("count({})*stride({})!=array_size({})", accessor_count, accessor_stride, array_size) ));
        }

        let mut layers_data=Vec::with_capacity(params.len());

        for &(_,_,ref data_type) in params.iter(){
            let layer_data=match *data_type{
                DataType::F32 => SourceLayer::F32( Vec::with_capacity(accessor_count) ),
                DataType::I32 => SourceLayer::I32( Vec::with_capacity(accessor_count) ),
                DataType::Name => SourceLayer::Name( Vec::with_capacity(accessor_count) ),
                DataType::Matrix4 => SourceLayer::Matrix4(  Vec::with_capacity(accessor_count) ),
            };

            layers_data.push(layer_data);
        }

        let mut array_iter=ArrayIter::new(array, array_size, "source");

        for i in 0..accessor_count{
            for j in 0..params.len() {
                match layers_data[j] {
                    SourceLayer::F32( ref mut list) =>
                        list.push( array_iter.read_f32()? ),
                    SourceLayer::I32( ref mut list) =>
                        list.push( array_iter.read_i32()? ),
                    SourceLayer::Name( ref mut list ) =>
                        list.push( String::from(array_iter.read_str()?) ),
                    SourceLayer::Matrix4( ref mut list ) => {
                        let mut mat=[0.0;16];
                        for k in 0..16 {
                            mat[k]=array_iter.read_f32()?;
                        }

                        list.push( Matrix::from(mat, asset) );
                    },
                }
            }
        }

        if asset.editor==Editor::Blender {//layer
            //invert x axis(blender uses left-side coordination system)
            for source_layer_index in 0..params.len() {
                if params[source_layer_index].1==LayerType::X {
                    match layers_data[source_layer_index]{
                        SourceLayer::F32(ref mut list) => {
                            for x in list.iter_mut() {
                                *x=-*x;
                            }
                        },
                        SourceLayer::I32(ref mut list) => {
                            for x in list.iter_mut() {
                                *x=-*x;
                            }
                        },
                        _ => {},
                    }
                }
            }
        }

        let mut layers=HashMap::new();

        for &(_,ref layer_type,_) in params.iter().rev(){
            match layers.entry( String::from(layer_type.print_vertex_format()) ){
                Entry::Occupied(_) => return Err(Error::Other( format!("Duplicate layer with vertex_format \"{}\"",layer_type.print_vertex_format()) )),
                Entry::Vacant(entry) => {
                    entry.insert( layers_data.pop().unwrap() );
                },
            }
        }

        Ok(layers)
    }
}

pub fn read_sources(element:&Element, asset:&Asset) -> Result<HashMap<String,Arc<Source>>,Error>{
    //read sources
    let mut sources=HashMap::new();

    for source_element in element.children.iter(){
        if source_element.name.as_str()=="source" {
            let source=Source::parse(&source_element, asset)?;

            match sources.entry(source.id.clone()){
                Entry::Occupied(_) => return Err(Error::Other( format!("Duplicate source with id \"{}\"", &source.id) )),
                Entry::Vacant(entry) => {
                    entry.insert(Arc::new(source));
                },
            }
        }
    }

    //find source synonyms
    for source_synonym in element.children.iter(){
        if source_synonym.name.as_str()=="vertices" {
            let new_id=source_synonym.get_attribute("id")?;
            let existing_id=source_synonym.get_element("input")?.get_attribute("source")?.trim_left_matches('#');

            let source=match sources.get(existing_id){
                Some(s) => s.clone(),
                None => return Err(Error::Other( format!("Source with id \"{}\" does not exists", existing_id) )),
            };

            match sources.entry(new_id.clone()){
                Entry::Occupied(_) => return Err(Error::Other( format!("Duplicate source synonym with id \"{}\"", new_id) )),
                Entry::Vacant(entry) => {
                    entry.insert(source);
                },
            }
        }
    }

    Ok(sources)
}

pub fn select_sources(element:&Element, sources:&HashMap<String,Arc<Source>>) -> Result<Vec<(String,Arc<Source>)>,Error>{
    let mut sources_list=Vec::new();

    for input_element in element.children.iter(){
        if input_element.name.as_str()=="input" {
            let source_semantic=input_element.get_attribute("semantic")?;
            let source_id=input_element.get_attribute("source")?.trim_left_matches('#');
            let offset=input_element.parse_attribute_as_usize("offset")?;

            if offset!=sources_list.len(){
                return Err(Error::Other( format!("Expected source offset {}, but {} have been found", sources_list.len(), offset) ));
            }

            let source=match sources.get(source_id){
                Some(s) => s.clone(),
                None => return Err(Error::Other( format!("Source with id \"{}\" does not exists", source_id) )),
            };

            sources_list.push((source_semantic.clone(),source));
        }
    }

    Ok( sources_list )
}
