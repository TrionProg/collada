extern crate collada;

use std::path::Path;

fn main(){
    let document=match collada::Document::parse(&Path::new("anim1.dae")){
        Ok(d) => d,
        Err(e) => panic!("{}",e),
    };

    document.print_tree();
}
