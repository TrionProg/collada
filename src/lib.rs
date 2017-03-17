extern crate xmltree;

mod xml_element;
pub use xml_element::XMLElement;

mod error;
pub use error::Error;

mod document;
pub use document::Document;

mod asset;
pub use asset::{Asset,Axis};

mod camera;
pub use camera::Camera;

mod source;
pub use source::{Source,SourceLayer};

mod mesh;
pub use mesh::{Mesh,VertexIndices};

mod geometry;
pub use geometry::Geometry;

mod node;
pub use node::Node;

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
