extern crate collada;

use std::path::Path;

#[test]
fn example(){
    let document=match collada::Document::parse(&Path::new("a2.dae")){
        Ok(d) => d,
        Err(e) => panic!("{}",e),
    };

    document.print_tree();

    let scene=document.scenes.get("Scene").unwrap();
    let node=scene.geometries.get("body").unwrap();
    let geometry=&node.joined;
    let mesh=&geometry.meshes[0];
    println!("{}",mesh.vertex_format);
    let polygon=&mesh.polygons[3];
    let position=mesh.vertex_indices.get("VERTEX").unwrap();
    let y_source_layer=position.source.layers.get("Y").unwrap();
    let source_data=match *y_source_layer {
        collada::SourceLayer::F32(ref data) => data,
        _ => panic!("we expect only f32"),
    };
    let vertex_index=polygon.first_vertex_index+1;
    println!("Y coord is {}",source_data[position.indices[vertex_index]]);

}
