use Error;
use XMLElement;
use xmltree::Element;

#[derive(Copy, Clone)]
pub enum LayerType{
    X,
    Y,
    Z,
    U,
    V,
}

impl LayerType{
    pub fn print_semantics(&self) -> &'static str{
        match *self{
            LayerType::X => "X",
            LayerType::Y => "Y",
            LayerType::Z => "Z",
            LayerType::U => "U",
            LayerType::V => "V",
        }
    }
}

#[derive(Copy, Clone)]
pub enum DataType{
    Float,
    Integer,
}

pub struct SourceLayer{
    pub layer_type:LayerType,
    pub data:SourceLayerData,
}

pub enum SourceLayerData{
    Float(Vec<f32>),
    Integer(Vec<i32>),
}

impl SourceLayerData{
    pub fn print_semantics(&self) -> &'static str{
        match *self{
            SourceLayerData::Float(_) => "float",
            SourceLayerData::Integer(_) => "integer",
        }
    }
}

pub struct Source{
    pub id:String,
    pub layers:Vec<SourceLayer>,
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
                    _ => return Err(Error::Other( format!("Expected X, Y, Z, S or T, but {} has been found",param_name_str) )),
                };

                let param_data_type_str=param_element.get_attribute("type")?.as_str();
                let param_type=match param_data_type_str{
                    "float" => DataType::Float,
                    _ => return Err(Error::Other( format!("Expected float, but {} has been found",param_data_type_str) )),
                };

                params.push((param_name, param_type));
            }
        }

        if accessor_stride!=params.len(){
            return Err(Error::Other( format!("stride({})!=params.len({})", accessor_stride, params.len()) ));
        }

        if accessor_count*accessor_stride!=float_array_count{
            return Err(Error::Other( format!("count({})*stride({})!=float_array_count({})", accessor_count, accessor_stride, float_array_count) ));
        }

        let mut layers=Vec::with_capacity(params.len());

        for &(layer_type,data_type) in params.iter(){
            let data=match data_type{
                DataType::Float => SourceLayerData::Float( Vec::with_capacity(accessor_count/accessor_stride) ),
                DataType::Integer => SourceLayerData::Integer( Vec::with_capacity(accessor_count/accessor_stride) ),
            };

            let layer=SourceLayer{
                layer_type:layer_type,
                data:data,
            };

            layers.push(layer);
        }

        let mut sourceDataIndex=0;
        for v in float_array_data.split(' ').filter(|v|*v!="").take(accessor_count){
            let layer=&mut layers[sourceDataIndex];

            match layer.data {
                SourceLayerData::Float( ref mut list) => {
                    match v.parse::<f32>(){
                        Ok ( f ) => list.push( f ),
                        Err( e ) => return Err(Error::Other( format!("Can not parse mesh data {} as float", v) )),
                    }
                },
                SourceLayerData::Integer( ref mut list) => {
                    match v.parse::<i32>(){
                        Ok ( f ) => list.push( f ),
                        Err( e ) => return Err(Error::Other( format!("Can not parse mesh data {} as integer", v) )),
                    }
                },
            }

            sourceDataIndex+=1;

            if sourceDataIndex==accessor_stride {
                sourceDataIndex=0;
            }
        }

        if layers.len()==0 {
            return Err(Error::Other( format!("Source \"{}\" is empty", &id) ));
        }

        Ok(
            Source{
                id:id,
                layers:layers,
            }
        )
    }
}
