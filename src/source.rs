use Error;
use XMLElement;
use xmltree::Element;

use std::collections::HashMap;
use std::collections::hash_map::Entry;

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
    pub fn print_semantics(&self) -> &'static str{
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
    Float,
    Integer,
}

impl DataType{
    pub fn print_data_type(&self) -> &'static str{
        match *self{
            DataType::Float => "float",
            DataType::Integer => "integer",
        }
    }
}

pub enum SourceLayer{
    Float(Vec<f32>),
    Integer(Vec<i32>),
}

impl SourceLayer{
    pub fn print_data_type(&self) -> &'static str{
        match *self{
            SourceLayer::Float(_) => "float",
            SourceLayer::Integer(_) => "integer",
        }
    }
}

pub struct Source{
    pub id:String,
    pub short_semantics:String,
    pub full_semantics:String,
    pub layers:HashMap<String,SourceLayer>,
}

impl Source{
    pub fn parse(source:&Element) -> Result<Source,Error>{
        let id=source.get_attribute("id")?.clone();

        let float_array=source.get_element("float_array")?;
        let float_array_count=float_array.parse_attribute_as_usize("count")?;
        let float_array_data=float_array.get_text()?;

        let accessor=source.get_element("technique_common")?.get_element("accessor")?;
        let accessor_count=accessor.parse_attribute_as_usize("count")?;
        let accessor_stride=accessor.parse_attribute_as_usize("stride")?;

        //read information about layers
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

                let param_data_type_str=param_element.get_attribute("type")?.as_str();
                let param_type=match param_data_type_str{
                    "float" => DataType::Float,
                    _ => return Err(Error::Other( format!("Expected float, but {} has been found",param_data_type_str) )),
                };

                params.push((param_name, param_type));
            }
        }

        if params.len()==0 {
            return Err(Error::Other( format!("Source \"{}\" is empty", &id) ));
        }

        let (full_semantics,short_semantics)={
            let mut full_semantics=String::new();
            let mut short_semantics=String::new();

            for &(param_name,param_type) in params.iter().take(params.len()-1){
                full_semantics.push_str( &format!("{}:{},",param_name.print_semantics(), param_type.print_data_type()) );
                short_semantics.push_str( &format!("{},",param_name.print_semantics()) );
            }

            let &(param_name,param_type)=match params.iter().last(){
                Some(p) => p,
                None => {unreachable!()},
            };

            full_semantics.push_str( &format!("{}:{}",param_name.print_semantics(), param_type.print_data_type()) );
            short_semantics.push_str( &format!("{}",param_name.print_semantics()) );

            (full_semantics,short_semantics)
        };

        if accessor_stride!=params.len(){
            return Err(Error::Other( format!("stride({})!=params.len({})", accessor_stride, params.len()) ));
        }

        if accessor_count*accessor_stride!=float_array_count{
            return Err(Error::Other( format!("count({})*stride({})!=float_array_count({})", accessor_count, accessor_stride, float_array_count) ));
        }

        let mut layers_data=Vec::with_capacity(params.len());

        for &(_,data_type) in params.iter(){
            let layer_data=match data_type{
                DataType::Float => SourceLayer::Float( Vec::with_capacity(accessor_count) ),
                DataType::Integer => SourceLayer::Integer( Vec::with_capacity(accessor_count) ),
            };

            layers_data.push(layer_data);
        }

        //read layer
        let mut source_data_index=0;
        for v in float_array_data.split(' ').filter(|v|*v!="").take(accessor_count*accessor_stride){
            let layer_data=&mut layers_data[source_data_index];

            match *layer_data {
                SourceLayer::Float( ref mut list) => {
                    match v.parse::<f32>(){
                        Ok ( f ) => list.push( f ),
                        Err( _ ) => return Err(Error::Other( format!("Can not parse mesh data {} as float", v) )),
                    }
                },
                SourceLayer::Integer( ref mut list) => {
                    match v.parse::<i32>(){
                        Ok ( f ) => list.push( f ),
                        Err( _ ) => return Err(Error::Other( format!("Can not parse mesh data {} as integer", v) )),
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
                SourceLayer::Float(ref list) => list.len(),
                SourceLayer::Integer(ref list) => list.len(),
            };

            if count!=accessor_count{
                return Err(Error::Other( format!("Expected count {}, but {} has been read", accessor_count, count) ));
            }
        }

        let mut layers=HashMap::new();

        for &(layer_type,_) in params.iter().rev(){
            match layers.entry( String::from(layer_type.print_semantics()) ){
                Entry::Occupied(_) => return Err(Error::Other( format!("Dublicate layer with semantics \"{}\"",layer_type.print_semantics()) )),
                Entry::Vacant(entry) => {
                    entry.insert( layers_data.pop().unwrap() );
                },
            }
        }

        Ok(
            Source{
                id:id,
                short_semantics:short_semantics,
                full_semantics:full_semantics,
                layers:layers,
            }
        )
    }
}
