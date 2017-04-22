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
fn example(){
    let document=match collada::Document::parse(&Path::new("scene.dae")){
        Ok(d) => d,
        Err(e) => panic!("{}",e),
    };

    document.print();

    let scene=document.scenes.get("Scene").unwrap();
    let node=scene.geometries.get("Cube").unwrap();
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
```

Document Tree
=============

```
Document
├── Geometries
│   ├── Geometry id:"Cube-mesh" name:"Cube"
│   │   └── Meshes
│   │       └── Mesh no material
│   │           ├── Short vertex_format: &(X,Y,Z) &(X,Y,Z)
│   │           └── Vertices
│   │               ├── Vertex indices for "NORMAL" source id:"Cube-mesh-normals"
│   │               └── Vertex indices for "VERTEX" source id:"Cube-mesh-positions"
│   ├── Geometry id:"Cylinder-mesh" name:"Cylinder"
│   │   └── Meshes
│   │       └── Mesh no material
│   │           ├── Short vertex_format: &(X,Y,Z) &(X,Y,Z) &(U,V)
│   │           └── Vertices
│   │               ├── Vertex indices for "VERTEX" source id:"Cylinder-mesh-positions"
│   │               ├── Vertex indices for "TEXCOORD" source id:"Cylinder-mesh-map-0"
│   │               └── Vertex indices for "NORMAL" source id:"Cylinder-mesh-normals"
│   └── Geometry id:"Cube_001-mesh" name:"Cube.001"
│       └── Meshes
│           └── Mesh no material
│               ├── Short vertex_format: &(X,Y,Z) &(X,Y,Z)
│               └── Vertices
│                   ├── Vertex indices for "NORMAL" source id:"Cube_001-mesh-normals"
│                   └── Vertex indices for "VERTEX" source id:"Cube_001-mesh-positions"
├── Skeletons
│   └── Skeleton id:"Guy"
│       └── Bone index:0 id:"Position" name:"Position"
│           └── Bone index:1 id:"Torse" name:"Torse"
│               ├── Bone index:2 id:"Hand_r" name:"Hand.r"
│               │   └── Bone index:3 id:"Hand_r_2" name:"Hand.r.2"
│               ├── Bone index:4 id:"Hand_l" name:"Hand.l"
│               │   └── Bone index:5 id:"Hand_l_2" name:"Hand.l.2"
│               └── Bone index:6 id:"Neck" name:"Neck"
│                   └── Bone index:7 id:"Head" name:"Head"
├── Animations
│   ├── Animation id:"Guy_Hand_l_pose_matrix" for bone with id "Hand_l" of skeleton with id "Guy"
│   │   ├── keyframes count: 17
│   │   └── Sources
│   │       ├── Source name:"OUTPUT" id:"Guy_Hand_l_pose_matrix-output"
│   │       ├── Source name:"INPUT" id:"Guy_Hand_l_pose_matrix-input"
│   │       └── Source name:"INTERPOLATION" id:"Guy_Hand_l_pose_matrix-interpolation"
│   ├── Animation id:"Guy_Neck_pose_matrix" for bone with id "Neck" of skeleton with id "Guy"
│   │   ├── keyframes count: 17
│   │   └── Sources
│   │       ├── Source name:"INPUT" id:"Guy_Neck_pose_matrix-input"
│   │       ├── Source name:"OUTPUT" id:"Guy_Neck_pose_matrix-output"
│   │       └── Source name:"INTERPOLATION" id:"Guy_Neck_pose_matrix-interpolation"
│   ├── Animation id:"Guy_Head_pose_matrix" for bone with id "Head" of skeleton with id "Guy"
│   │   ├── keyframes count: 17
│   │   └── Sources
│   │       ├── Source name:"INPUT" id:"Guy_Head_pose_matrix-input"
│   │       ├── Source name:"INTERPOLATION" id:"Guy_Head_pose_matrix-interpolation"
│   │       └── Source name:"OUTPUT" id:"Guy_Head_pose_matrix-output"
│   ├── Animation id:"Guy_Hand_r_2_pose_matrix" for bone with id "Hand_r_2" of skeleton with id "Guy"
│   │   ├── keyframes count: 17
│   │   └── Sources
│   │       ├── Source name:"INPUT" id:"Guy_Hand_r_2_pose_matrix-input"
│   │       ├── Source name:"OUTPUT" id:"Guy_Hand_r_2_pose_matrix-output"
│   │       └── Source name:"INTERPOLATION" id:"Guy_Hand_r_2_pose_matrix-interpolation"
│   ├── Animation id:"Guy_Position_pose_matrix" for bone with id "Position" of skeleton with id "Guy"
│   │   ├── keyframes count: 17
│   │   └── Sources
│   │       ├── Source name:"OUTPUT" id:"Guy_Position_pose_matrix-output"
│   │       ├── Source name:"INTERPOLATION" id:"Guy_Position_pose_matrix-interpolation"
│   │       └── Source name:"INPUT" id:"Guy_Position_pose_matrix-input"
│   ├── Animation id:"Guy_Hand_l_2_pose_matrix" for bone with id "Hand_l_2" of skeleton with id "Guy"
│   │   ├── keyframes count: 17
│   │   └── Sources
│   │       ├── Source name:"INTERPOLATION" id:"Guy_Hand_l_2_pose_matrix-interpolation"
│   │       ├── Source name:"INPUT" id:"Guy_Hand_l_2_pose_matrix-input"
│   │       └── Source name:"OUTPUT" id:"Guy_Hand_l_2_pose_matrix-output"
│   ├── Animation id:"Guy_Hand_r_pose_matrix" for bone with id "Hand_r" of skeleton with id "Guy"
│   │   ├── keyframes count: 17
│   │   └── Sources
│   │       ├── Source name:"INTERPOLATION" id:"Guy_Hand_r_pose_matrix-interpolation"
│   │       ├── Source name:"INPUT" id:"Guy_Hand_r_pose_matrix-input"
│   │       └── Source name:"OUTPUT" id:"Guy_Hand_r_pose_matrix-output"
│   └── Animation id:"Guy_Torse_pose_matrix" for bone with id "Torse" of skeleton with id "Guy"
│       ├── keyframes count: 17
│       └── Sources
│           ├── Source name:"INTERPOLATION" id:"Guy_Torse_pose_matrix-interpolation"
│           ├── Source name:"INPUT" id:"Guy_Torse_pose_matrix-input"
│           └── Source name:"OUTPUT" id:"Guy_Torse_pose_matrix-output"
├── Skins
│   └── Skin id:"Guy_Cube-skin" for geometry with id "Cube_001-mesh"
│       ├── Additional sources
│       │   ├── Source name:"JOINT" id:"Guy_Cube-skin-joints"
│       │   └── Source name:"INV_BIND_MATRIX" id:"Guy_Cube-skin-bind_poses"
│       └── Bones
│           ├── Bone indices for "JOINT" source id:"Guy_Cube-skin-joints"
│           └── Bone indices for "WEIGHT" source id:"Guy_Cube-skin-weights"
└── Scenes
    └── Scene id:"Scene" name:"Scene"
        ├── Geometries
        │   ├── Node id:"House" name:"House" joided to "Cube-mesh"
        │   │   └── Controller: model positions
        │   ├── Node id:"Culumn" name:"Culumn" joided to "Cylinder-mesh"
        │   │   └── Controller: model positions
        │   └── Node id:"Cube" name:"Cube" joided to "Cube_001-mesh"
        │       └── Controller: Skin id:"Guy_Cube-skin" for geometry with id "Cube_001-mesh"
        ├── Skeletons
        │   └── Node id:"Guy" name:"Guy" joided to "Guy"
        └── Cameras
            └── Node id:"Camera" name:"Camera" joided to "Camera-camera"
                └── Controller: model positions
```

License
=======
MIT
