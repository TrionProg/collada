About
=====
This rust library reads COLLADA file and stores all data as tree in RAM. Example of this tree see below.

How to use it
=============
This library allows you to design a converter for your own format of 3D models. You just need to call `Document::parse("model.dae")` function, think about data you are interested in and write it to output file of your format.

Details
=======
Because this library tries to be so flexible as possible, it stores data by the following way:
* The world may contain one or more **scenes**.
* Each *scene* contains **nodes** like **geometry** or **camera**.
* Each *geometry* contains **meshes**.
* Each *mesh* may have only one material(or no material) and contains next geometry data:
* **Polygons** may contain 2,3 or more **vertices**. Each Polygon stores index of first vertex and number of vertices.
* *Mesh* has **vertex_indices** field. It contains vertices, but because each vertex contains a few layers like Position,Texture Coords, vertex_indices is list of this **vertex indices**.
* Each *VertexIndices* contains **source** and **indices** of values, that source contains.
* *Source* is list of, for example, coordinates, but each coordinate has several layers like X,Y,Z. That is why each source contains several **source layers**.
* And finally each *source layer* may keep values as float or integer.

The full_semantics of mesh describes information about all layers and sources. & means that each vertex contains index. If you will keep (x,y,z) for each vertex directly in struct Vertex, you should use (X,Y,Z) semantics without &.

Example
-------

For example we want to get Y coordinate of second vertex of Polygon 3 of object named "body". Then we should write:

```rust
let document=match collada::Document::parse(&Path::new("a2.dae")){
    Ok(d) => d,
    Err(e) => panic!("{}",e),
};

let scene=document.scenes.get("Scene").unwrap();
let node=scene.geometries.get("body").unwrap();
let geometry=&node.joined;
let mesh=&geometry.meshes[0];
println!("{}",mesh.full_semantics);
let polygon=&mesh.polygons[3];
let position=mesh.vertex_layers.get("VERTEX").unwrap();
let y_source_layer=position.source.layers.get("Y").unwrap();
let source_data=match *y_source_layer {
    collada::SourceLayer::Float(ref data) => data,
    _ => panic!("we expect only float"),
};
let vertex_index=polygon.first_vertex_index+1;
println!("Y coord is {}",source_data[position.indices[vertex_index]]);

document.print_tree();//print document tree
```

Document Tree
=============

```
Document
├── Geometries
│   ├── Geometry id:"Cylinder_003-mesh" name:"Cylinder.003"
│   │   ├── Mesh material:"tex1-material"
│   │   │   ├── Short semantics: &(X,Y,Z) &(X,Y,Z) &(U,V) &(R,G,B)
│   │   │   └── Vertex
│   │   │       ├── Vertex indices for "NORMAL" source id:"Cylinder_003-mesh-normals"
│   │   │       ├── Vertex indices for "TEXCOORD" source id:"Cylinder_003-mesh-map-0"
│   │   │       ├── Vertex indices for "COLOR" source id:"Cylinder_003-mesh-colors-Col"
│   │   │       └── Vertex indices for "VERTEX" source id:"Cylinder_003-mesh-positions"
│   │   └── Mesh material:"tex2-material"
│   │       ├── Short semantics: &(X,Y,Z) &(X,Y,Z) &(U,V) &(R,G,B)
│   │       └── Vertex
│   │           ├── Vertex indices for "NORMAL" source id:"Cylinder_003-mesh-normals"
│   │           ├── Vertex indices for "TEXCOORD" source id:"Cylinder_003-mesh-map-0"
│   │           ├── Vertex indices for "COLOR" source id:"Cylinder_003-mesh-colors-Col"
│   │           └── Vertex indices for "VERTEX" source id:"Cylinder_003-mesh-positions"
│   ├── Geometry id:"Cylinder_008-mesh" name:"Cylinder.008"
│   │   └── Mesh material:"Material_001-material"
│   │       ├── Short semantics: &(X,Y,Z) &(X,Y,Z) &(U,V)
│   │       └── Vertex
│   │           ├── Vertex indices for "NORMAL" source id:"Cylinder_008-mesh-normals"
│   │           ├── Vertex indices for "TEXCOORD" source id:"Cylinder_008-mesh-map-0"
│   │           └── Vertex indices for "VERTEX" source id:"Cylinder_008-mesh-positions"
│   ├── Geometry id:"Cylinder_004-mesh" name:"Cylinder.004"
│   │   └── Mesh material:"Material-material"
│   │       ├── Short semantics: &(X,Y,Z) &(X,Y,Z) &(U,V)
│   │       └── Vertex
│   │           ├── Vertex indices for "NORMAL" source id:"Cylinder_004-mesh-normals"
│   │           ├── Vertex indices for "TEXCOORD" source id:"Cylinder_004-mesh-map-0"
│   │           └── Vertex indices for "VERTEX" source id:"Cylinder_004-mesh-positions"
│   └── Geometry id:"Cylinder_007-mesh" name:"Cylinder.007"
│       └── Mesh material:"tex2-material"
│           ├── Short semantics: &(X,Y,Z) &(X,Y,Z) &(U,V)
│           └── Vertex
│               ├── Vertex indices for "NORMAL" source id:"Cylinder_007-mesh-normals"
│               ├── Vertex indices for "TEXCOORD" source id:"Cylinder_007-mesh-map-0"
│               └── Vertex indices for "VERTEX" source id:"Cylinder_007-mesh-positions"
└── Scenes
    └── Scene id:"Scene" name:"Scene"
        ├── Geometries
        │   ├── Node id:"WFR" name:"WFR" joided to "Cylinder_007-mesh"
        │   ├── Node id:"WFR_003" name:"WFR_003" joided to "Cylinder_007-mesh"
        │   ├── Node id:"WFR_002" name:"WFR_002" joided to "Cylinder_007-mesh"
        │   ├── Node id:"body" name:"body" joided to "Cylinder_003-mesh"
        │   ├── Node id:"Cylinder_003" name:"Cylinder_003" joided to "Cylinder_004-mesh"
        │   ├── Node id:"Cylinder_000" name:"Cylinder_000" joided to "Cylinder_008-mesh"
        │   └── Node id:"WFR_001" name:"WFR_001" joided to "Cylinder_007-mesh"
        └── Cameras
            └── Node id:"Camera" name:"Camera" joided to "Camera-camera"
```

License
=======
MIT
