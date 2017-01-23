extern crate collada;

use std::path::Path;

fn main(){
    let document=match collada::Document::parse(&Path::new("a2.dae")){
        Ok(d) => d,
        Err(e) => {println!("{}",e); return; },
    };

    document.print_tree();

    let scene=document.scenes.get("Scene").unwrap();
    let node=scene.nodes.get("body").unwrap();
    let geometry=match node.joined{
        collada::JoinedTo::Geometry(ref geometry) => geometry,
        _ => panic!("we expect only geometry"),
    };
    let mesh=&geometry.meshes[0];
    println!("{}",mesh.full_semantics);
    let polygon=&mesh.polygons[3];
    let position=mesh.vertex_layers.get("VERTEX").unwrap();
    let y_source_layer=&position.source.layers[1];
    let source_data=match y_source_layer.data {
        collada::SourceLayerData::Float(ref data) => data,
        _ => panic!("we expect only float"),
    };
    let vertex_index=polygon.first_vertex_index+1;
    println!("Y coord is {}",source_data[position.indexes[vertex_index]]);
}
