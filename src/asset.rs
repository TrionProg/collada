use Error;
use XMLElement;
use xmltree::Element;

pub struct Unit{
    pub name:String,
    pub ratio:f32,
}

pub enum Axis{
    X,
    Y,
    Z,
}

pub struct Asset{
    pub created:String,
    pub modified:String,
    pub unit:Unit,
    pub up_axis:Axis,
}

impl Asset{
    pub fn parse(root:&Element) -> Result<Asset,Error>{
        let asset=root.get_element("asset")?;

        let created=asset.get_element("created")?.get_text()?.clone();
        let modified=asset.get_element("modified")?.get_text()?.clone();

        let unit_name=asset.get_element("unit")?.get_attribute("name")?.clone();

        let unit_ratio=match unit_name.as_str(){
            "meter" => {
                let ratio_str=asset.get_element("unit")?.get_attribute("meter")?;

                match ratio_str.parse::<f32>(){
                    Ok ( r ) => r,
                    Err( _ ) => return Err(Error::StringParseError( String::from("Can not parse meter ratio as f32") )),
                }
            },
            _ => return Err(Error::Other( format!("Asset/Unit: Expected meter unit, found {}", unit_name.as_str()) )),
        };

        let up_axis={
            let up_axis_str=asset.get_element("up_axis")?.get_text()?;

            match up_axis_str.as_str(){
                "X_UP" => Axis::X,
                "Y_UP" => Axis::Y,
                "Z_UP" => Axis::Z,
                _ => return Err(Error::Other( format!("Expected X_UP, Y_UP or Z_UP, but {} has been found",up_axis_str.as_str()) )),
            }
        };

        Ok(
            Asset{
                created:created,
                modified:modified,
                unit:Unit{
                    name:unit_name,
                    ratio:unit_ratio,
                },
                up_axis:up_axis,
            }
        )
    }
}
