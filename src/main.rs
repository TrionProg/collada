extern crate collada;

use std::path::Path;

fn main(){
    let document=match collada::Document::parse(&Path::new("anim2.dae")){
        Ok(d) => d,
        Err(e) => panic!("{}",e),
    };

    document.print();
}
