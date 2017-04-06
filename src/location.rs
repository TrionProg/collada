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

    pub fn with_asset(x:f32,y:f32,z:f32,asset:&Asset) -> Self{
        let mut position = match asset.up_axis {
            Axis::X => Position::new(y,x,z),//unknown
            Axis::Y => Position::new(x,y,z),//standard
            Axis::Z => Position::new(x,z,y),//blender
        };

        if asset.editor==Editor::Blender {
            position.x=-position.x;
        }

        position
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

        let position = Position::with_asset(values[0], values[1], values[2], asset);
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

    pub fn with_asset(x:f32,y:f32,z:f32,asset:&Asset) -> Self{
        let scale = match asset.up_axis {
            Axis::X => Scale::new(y,x,z),//unknown
            Axis::Y => Scale::new(x,y,z),//standard
            Axis::Z => Scale::new(x,z,y),//blender
        };

        scale
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

        Ok( Scale::with_asset(values[0], values[1], values[2], asset) )
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

    pub fn with_asset(x:f32,y:f32,z:f32,asset:&Asset) -> Self{
        let mut euler = match asset.up_axis {
            Axis::X => Euler::new(y,x,z),//unknown
            Axis::Y => Euler::new(x,y,z),//standard
            Axis::Z => Euler::new(x,z,y),//blender
        };

        if asset.editor==Editor::Blender {
            euler.yaw=-euler.yaw;
        }

        euler
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

        Ok( Euler::with_asset(rotation_x, rotation_y, rotation_z, asset) )
    }

    //pub fn to_quat(&self) -> Quaternion {

}

pub struct Quaternion{
    pub x:f32,
    pub y:f32,
    pub z:f32,
    pub w:f32,
}

impl Quaternion {
    pub fn new(x:f32,y:f32,z:f32,w:f32) -> Self{
        Quaternion{
            x:x,
            y:y,
            z:z,
            w:w,
        }
    }

    pub fn with_asset(x:f32,y:f32,z:f32,w:f32,asset:&Asset) -> Self{
        let mut quat = match asset.up_axis {
            Axis::X => Quaternion::new(y,x,z,w),//unknown
            Axis::Y => Quaternion::new(x,y,z,w),//standard
            Axis::Z => Quaternion::new(x,z,y,w),//blender
        };

        if asset.editor==Editor::Blender {
            quat.x=-quat.x;
            quat.z=-quat.z;
        }

        quat
    }
}

pub struct Matrix{
    pub mat:[f32;16],
}

impl Matrix{
    pub fn from(mat:[f32;16]) -> Matrix{
        Matrix {
            mat:mat,
        }
    }

    pub fn parse(text:&String) -> Result<Matrix,Error>{
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

        let matrix=Matrix{
            mat:values,
        };

        Ok( matrix )
    }

    pub fn to_quat(&self, asset:&Asset) -> Quaternion {
        let mat=&self.mat;

        let t=mat[0] + mat[5] + mat[10] + 1.0;

        if t>0.0 {
            let s = 0.5 / t.sqrt();
            let w = 0.25 / s;
            let x = ( mat[9] - mat[6] ) * s;
            let y = ( mat[2] - mat[8] ) * s;
            let z = ( mat[4] - mat[1] ) * s;

            Quaternion::with_asset(x, y, z, w, asset)
        }else if mat[0]>mat[5] && mat[0]>mat[10] {
            let s = (( 1.0 + mat[0] - mat[5] - mat[10] ) * 2.0).sqrt();
            let x = 0.5 / s;
            let y = (mat[1] + mat[4] ) / s;
            let z = (mat[2] + mat[8] ) / s;
            let w = (mat[6] + mat[9] ) / s;

            Quaternion::with_asset(x, y, z, w, asset)
        }else if mat[5]>mat[0] && mat[5]>mat[10] {
            let s = (( 1.0 + mat[5] - mat[0] - mat[10] ) * 2.0).sqrt();
            let x = (mat[1] + mat[4] ) / s;
            let y = 0.5 / s;
            let z = (mat[6] + mat[9] ) / s;
            let w = (mat[2] + mat[8] ) / s;

            Quaternion::with_asset(x, y, z, w, asset)
        }else{
            let s = (( 1.0 + mat[10] - mat[0] - mat[5] ) * 2.0).sqrt();
            let x = (mat[2] + mat[8] ) / s;
            let y = (mat[6] + mat[9] ) / s;
            let z = 0.5 / s;
            let w = (mat[1] + mat[4] ) / s;

            Quaternion::with_asset(x, y, z, w, asset)
        }
    }

    pub fn to_location(&self, asset:&Asset) -> Location {
        let position = Position::with_asset(self.mat[3], self.mat[7], self.mat[11], asset);

        let scale_x = ((self.mat[0].powi(2) + self.mat[4].powi(2) + self.mat[8].powi(2)).sqrt()*100.0).round()/100.0;
        let scale_y = ((self.mat[1].powi(2) + self.mat[5].powi(2) + self.mat[9].powi(2)).sqrt()*100.0).round()/100.0;
        let scale_z = ((self.mat[2].powi(2) + self.mat[6].powi(2) + self.mat[10].powi(2)).sqrt()*100.0).round()/100.0;

        let scale = Scale::with_asset(scale_x, scale_y, scale_z, asset);

        let quat=self.to_quat(asset);

        Location::new(position, scale, quat)
    }

}


pub struct Location {
    pub position:Position,
    pub scale:Scale,
    pub rotation:Quaternion,
}

impl Location {
    pub fn new(position:Position, scale:Scale, rotation:Quaternion) -> Self {
        Location {
            position:position,
            scale:scale,
            rotation:rotation,
        }
    }
}
