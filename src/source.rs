use Error;
use XMLElement;
use xmltree::Element;

use std::collections::HashMap;
use std::collections::hash_map::Entry;

use Asset;
use Axis;

#[derive(Copy, Clone)]
pub enum LayerType{
    X,
    Y,
    Z,
    U,
    V,
    R,
    G,
    B,
}

impl LayerType{
    pub fn print_vertex_format(&self) -> &'static str{
        match *self{
            LayerType::X => "X",
            LayerType::Y => "Y",
            LayerType::Z => "Z",
            LayerType::U => "U",
            LayerType::V => "V",
            LayerType::R => "R",
            LayerType::G => "G",
            LayerType::B => "B",
        }
    }
}

#[derive(Copy, Clone)]
pub enum DataType{
    F32,
    I32,
}

impl DataType{
    pub fn print_data_type(&self) -> &'static str{
        match *self{
            DataType::F32 => "f32",
            DataType::I32 => "i32",
        }
    }
}

pub enum SourceLayer{
    F32(Vec<f32>),
    I32(Vec<i32>),
}

impl SourceLayer{
    pub fn print_data_type(&self) -> &'static str{
        match *self{
            SourceLayer::F32(_) => "f32",
            SourceLayer::I32(_) => "i32",
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

        let float_array=source.get_element("float_array")?;
        let float_array_count=float_array.parse_attribute_as_usize("count")?;
        let float_array_data=float_array.get_text()?;

        let accessor=source.get_element("technique_common")?.get_element("accessor")?;
        let accessor_count=accessor.parse_attribute_as_usize("count")?;
        let accessor_stride=accessor.parse_attribute_as_usize("stride")?;

        let mut params=Self::get_params(&accessor, &id, asset)?;

        let (vertex_format,short_vertex_format)=Self::get_vertex_format(&params);

        let mut layers_data=Self::get_layers_data(
            accessor_stride, accessor_count, float_array_data, float_array_count, &params
        )?;

        let mut layers=HashMap::new();

        for &(_,layer_type,_) in params.iter().rev(){
            match layers.entry( String::from(layer_type.print_vertex_format()) ){
                Entry::Occupied(_) => return Err(Error::Other( format!("Dublicate layer with vertex_format \"{}\"",layer_type.print_vertex_format()) )),
                Entry::Vacant(entry) => {
                    entry.insert( layers_data.pop().unwrap() );
                },
            }
        }

        Ok(
            Source{
                id:id,
                short_vertex_format:short_vertex_format,
                vertex_format:vertex_format,
                layers:layers,
            }
        )
    }

    fn get_params(accessor:&Element, id:&String, asset:&Asset) -> Result<Vec<(LayerType,LayerType,DataType)>,Error>{
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
                    _ => return Err(Error::Other( format!("Expected X, Y, Z, S, T, R, G or B but {} has been found",param_name_str) )),
                };

                let standard_layer_type=Self::get_standard_layer_type(param_name, asset);

                let param_data_type_str=param_element.get_attribute("type")?.as_str();
                let param_type=match param_data_type_str{
                    "float" => DataType::F32,
                    _ => return Err(Error::Other( format!("Expected float, but {} has been found",param_data_type_str) )),
                };

                params.push((param_name, standard_layer_type, param_type));
            }
        }

        if params.len()==0 {
            return Err(Error::Other( format!("Source \"{}\" is empty", id) ));
        }

        Ok(params)
    }

    fn get_standard_layer_type(layer_type:LayerType, asset:&Asset) -> LayerType {
        match layer_type {
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
            _ => layer_type
        }
    }

    fn get_vertex_format(params:&Vec<(LayerType,LayerType,DataType)>) -> (String,String) {
        let mut vertex_format=String::new();
        let mut short_vertex_format=String::new();

        for &(param_name,_,param_type) in params.iter().take(params.len()-1){
            vertex_format.push_str( &format!("{}:{},",param_name.print_vertex_format(), param_type.print_data_type()) );
            short_vertex_format.push_str( &format!("{},",param_name.print_vertex_format()) );
        }

        let &(param_name,_,param_type)=match params.iter().last(){
            Some(p) => p,
            None => {unreachable!()},
        };

        vertex_format.push_str( &format!("{}:{}",param_name.print_vertex_format(), param_type.print_data_type()) );
        short_vertex_format.push_str( &format!("{}",param_name.print_vertex_format()) );

        (vertex_format,short_vertex_format)
    }

    fn get_layers_data(
        accessor_stride:usize,
        accessor_count:usize,
        float_array_data:&String,
        float_array_count:usize,
        params:&Vec<(LayerType,LayerType,DataType)>
    ) -> Result<Vec<SourceLayer>,Error> {
        if accessor_stride!=params.len(){
            return Err(Error::Other( format!("stride({})!=params.len({})", accessor_stride, params.len()) ));
        }

        if accessor_count*accessor_stride!=float_array_count{
            return Err(Error::Other( format!("count({})*stride({})!=float_array_count({})", accessor_count, accessor_stride, float_array_count) ));
        }

        let mut layers_data=Vec::with_capacity(params.len());

        for &(_,_,data_type) in params.iter(){
            let layer_data=match data_type{
                DataType::F32 => SourceLayer::F32( Vec::with_capacity(accessor_count) ),
                DataType::I32 => SourceLayer::I32( Vec::with_capacity(accessor_count) ),
            };

            layers_data.push(layer_data);
        }

        //read layer
        let mut source_data_index=0;
        for v in float_array_data.split(' ').filter(|v|*v!="").take(accessor_count*accessor_stride){
            let layer_data=&mut layers_data[source_data_index];

            match *layer_data {
                SourceLayer::F32( ref mut list) => {
                    match v.parse::<f32>(){
                        Ok ( f ) => list.push( f ),
                        Err( _ ) => return Err(Error::ParseFloatError( String::from("Mesh data"), String::from(v)) ),
                    }
                },
                SourceLayer::I32( ref mut list) => {
                    match v.parse::<i32>(){
                        Ok ( f ) => list.push( f ),
                        Err( _ ) => return Err(Error::ParseIntError( String::from("Mesh data"), String::from(v)) ),
                    }
                },
            }

            source_data_index+=1;

            if source_data_index==accessor_stride {
                source_data_index=0;
            }
        }

        //check
        for layer_data in layers_data.iter(){
            let count=match *layer_data{
                SourceLayer::F32(ref list) => list.len(),
                SourceLayer::I32(ref list) => list.len(),
            };

            if count!=accessor_count{
                return Err(Error::Other( format!("Expected count {}, but {} has been read", accessor_count, count) ));
            }
        }

        Ok(layers_data)
    }
}
