use Error;
use XMLElement;
use xmltree::Element;

use Asset;
use Axis;
use Editor;

pub struct Position{
    pub x:f32,
    pub y:f32,
    pub z:f32,
}

impl Position{
    pub fn new(x:f32,y:f32,z:f32) -> Self{
        Position{
            x:x,
            y:y,
            z:z,
        }
    }

    pub fn parse(text:&String, asset:&Asset) -> Result<Self,Error>{
        let mut values=[0.0;3];

        let mut count=0;

        for (i,v) in text.split(' ').filter(|v|*v!="").take(3).enumerate(){
            match v.parse::<f32>(){
                Ok ( v ) => values[i]=v,
                Err( _ ) => return Err(Error::ParseFloatError( String::from("position"), String::from(v) ) ),
            }

            count+=1;
        }

        //check
        if count!=3 {
            return Err(Error::Other( format!("Only {} elements of position have been read, expected 3", count) ));
        }

        let mut position = match asset.up_axis {
            Axis::X => Position::new(values[1],values[0],values[2]),//unknown
            Axis::Y => Position::new(values[0],values[1],values[2]),//standard
            Axis::Z => Position::new(values[0],values[2],values[1]),//blender
        };

        if asset.editor==Editor::Blender {
            position.x=-position.x;
        }

        Ok( position )
    }
}

pub struct Scale{
    pub x:f32,
    pub y:f32,
    pub z:f32,
}

impl Scale{
    pub fn new(x:f32,y:f32,z:f32) -> Self{
        Scale{
            x:x,
            y:y,
            z:z,
        }
    }

    pub fn parse(text:&String, asset:&Asset) -> Result<Self,Error>{
        let mut values=[0.0;3];

        let mut count=0;

        for (i,v) in text.split(' ').filter(|v|*v!="").take(3).enumerate(){
            match v.parse::<f32>(){
                Ok ( v ) => values[i]=v,
                Err( _ ) => return Err(Error::ParseFloatError( String::from("scale"), String::from(v) ) ),
            }

            count+=1;
        }

        //check
        if count!=3 {
            return Err(Error::Other( format!("Only {} elements of scale have been read, expected 3", count) ));
        }

        let scale = match asset.up_axis {
            Axis::X => Scale::new(values[1],values[0],values[2]),//unknown
            Axis::Y => Scale::new(values[0],values[1],values[2]),//standard
            Axis::Z => Scale::new(values[0],values[2],values[1]),//blender
        };

        Ok( scale )
    }
}

pub struct Euler{
    pub pitch:f32,
    pub yaw:f32,
    pub roll:f32,
}

impl Euler {
    pub fn new(x:f32,y:f32,z:f32) -> Self{
        Euler{
            pitch:x,
            yaw:y,
            roll:z,
        }
    }

    fn parse_angle(text:&String, name:&'static str) -> Result<f32,Error> {
        let value_str=match text.split(' ').filter(|v|*v!="").nth(3){
            Some( vs ) => vs,
            None => return Err(Error::Other( format!("{} does not contains angle in digress",name))),
        };

        let angle=match value_str.parse::<f32>(){
            Ok ( v ) => v,
            Err( _ ) => return Err(Error::ParseFloatError( String::from(name), String::from(value_str) ) ),
        };

        Ok(angle)
    }

    pub fn parse(node:&Element, asset:&Asset) -> Result<Self,Error>{
        let mut rotation_x=0.0;
        let mut rotation_y=0.0;
        let mut rotation_z=0.0;

        for node_element in node.children.iter(){
            if node_element.name.as_str()=="rotate" {
                let sid=node_element.get_attribute("sid")?;

                match sid.as_str() {
                    "rotationZ" => rotation_z=Self::parse_angle(node_element.get_text()?,"rotationZ")?,
                    "rotationY" => rotation_y=Self::parse_angle(node_element.get_text()?,"rotationY")?,
                    "rotationX" => rotation_x=Self::parse_angle(node_element.get_text()?,"rotationX")?,
                    _ => return Err( Error::Other(format!("Unknown sid of rotation: \"{}\"",sid)) ),
                }
            }
        }

        let mut euler = match asset.up_axis {
            Axis::X => Euler::new(rotation_y,rotation_x,rotation_z),//unknown
            Axis::Y => Euler::new(rotation_x,rotation_y,rotation_z),//standard
            Axis::Z => Euler::new(rotation_x,rotation_z,rotation_y),//blender
        };

        if asset.editor==Editor::Blender {
            euler.yaw=-euler.yaw;
        }

        Ok( euler )
    }
}

pub struct Matrix{
    pub mat:[f32;16],
}

impl Matrix{
    pub fn from(mat:[f32;16], asset:&Asset) -> Matrix{
        Matrix {
            mat:mat,
        }
    }
    
    pub fn parse(text:&String, asset:&Asset) -> Result<Matrix,Error>{
        let mut values=[0.0;16];

        let mut count=0;

        for (i,v) in text.split(' ').filter(|v|*v!="").take(16).enumerate(){
            match v.parse::<f32>(){
                Ok ( v ) => values[i]=v,
                Err( _ ) => return Err(Error::ParseFloatError( String::from("matrix"), String::from(v) ) ),
            }

            count+=1;
        }

        //check
        if count!=16 {
            return Err(Error::Other( format!("Only {} elements of matrix have been read, expected 16", count) ));
        }
        /*
        {
            let x=&mut values[3];
            let y=&mut values[7];
            let z=&mut values[11];

            match asset.up_axis {
                Axis::X => {
                    let tmp=*x;
                    *x=*y;
                    *y=tmp;
                },
                Axis::Y => {},
                Axis::Z => {
                    let tmp=*y;
                    *y=*z;
                    *z=tmp;
                },
            }


            if asset.editor==Editor::Blender {
                *x=-*x;
            }
        }
        */

        //TODO:Angle,scale(I hope, not xyz)

        let mut matrix=Matrix{
            mat:values,
        };

        Ok( matrix )
    }
}
