extern crate collada;

use std::path::Path;

fn main(){
    let document=match collada::Document::parse(&Path::new("a2.dae")){
        Ok(d) => d,
        Err(e) => {println!("{}",e); return; },
    };

    document.print_tree();
}
