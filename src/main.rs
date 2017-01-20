extern crate collada;

use std::path::Path;

fn main(){
    match collada::Document::parse(&Path::new("a1.dae")){
        Ok(_) => {},
        Err(e) => println!("{}",e),
    };
}
