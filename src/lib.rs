extern crate xmltree;

mod string_ext;
pub use string_ext::StringExt;

mod array;
pub use array::ArrayIter;

mod xml_element;
pub use xml_element::XMLElement;

mod error;
pub use error::Error;

mod document;
pub use document::Document;

mod asset;
pub use asset::{Asset,Axis,Editor};

mod camera;
pub use camera::Camera;

mod source;
pub use source::{Source,SourceLayer};

mod mesh;
pub use mesh::{Mesh,VertexIndices};

mod geometry;
pub use geometry::Geometry;

mod controller;
pub use controller::{Controller,Skin,BoneIndices};

mod location;
pub use location::{Position,Scale,Euler,Matrix};

mod node;
pub use node::Node;

mod skeleton;
pub use skeleton::{Bone,Skeleton};

mod animation;
pub use animation::Animation;

mod scene;
pub use scene::Scene;

pub fn print_branch(last:bool) {
    if last {
        print!("└── ");
    }else{
        print!("├── ");
    }
}

pub fn print_tab(last:bool){
    if last {
        print!("    ");
    }else{
        print!("│   ");
    }
}
