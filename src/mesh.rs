use Error;
use XMLElement;
use xmltree::Element;

use std::collections::HashMap;
use std::collections::hash_map::Entry;
use std::sync::Arc;

use Source;
use Asset;

pub struct Polygon{
    pub first_vertex_index:usize,
    pub vertices_count:usize,
}

pub struct Mesh{
    pub id:usize,
    pub name:String,
    pub material:Option<String>,
    pub short_vertex_format:String,
    pub vertex_format:String,
    pub sources:Vec<(String,Arc<Source>)>,
    pub polygons:Vec<Polygon>,
    pub vertex_indices:HashMap<String,Arc<VertexIndices>>,
}

pub struct VertexIndices{
    pub source:Arc<Source>,
    pub indices:Vec<usize>,
}

//TODO:Material shoild be Arc

impl Mesh{
    pub fn parse_meshes(
        mesh:&Element,
        geometry_name:&String,
        mesh_index:usize,
        mesh_id: &mut usize,
        meshes:&mut Vec<Arc<Mesh>>,
        asset:&Asset
    ) -> Result<(),Error>{
        let all_sources=Mesh::read_sources(mesh, asset)?;

        for polylist in mesh.children.iter(){
            if polylist.name.as_str()=="polylist"{
                let material=match polylist.attributes.get("material"){
                    Some(m) => Some(m.clone()),
                    None => None,
                };

                let (sources, short_vertex_format, vertex_format)=Mesh::select_sources_and_generate_vertex_format(&polylist, &all_sources)?;

                let (polygons,vertices_count)=Mesh::read_polygons(&polylist)?;
                let vertex_indices=Mesh::read_vertices(&polylist, vertices_count, &sources)?;

                let mesh=Mesh{
                    id:*mesh_id,
                    name:format!("{}#{}",geometry_name, mesh_index),
                    material:material,
                    short_vertex_format:short_vertex_format,
                    vertex_format:vertex_format,
                    sources:sources,
                    polygons:polygons,
                    vertex_indices:vertex_indices,
                };

                meshes.push( Arc::new( mesh ) );

                *mesh_id+=1;
            }
        }

        Ok(())
    }

    pub fn read_sources(mesh:&Element, asset:&Asset) -> Result<HashMap<String,Arc<Source>>,Error>{
        //read sources
        let mut sources=HashMap::new();

        for source_element in mesh.children.iter(){
            if source_element.name.as_str()=="source" {
                let source=Source::parse(&source_element, asset)?;

                match sources.entry(source.id.clone()){
                    Entry::Occupied(_) => return Err(Error::Other( format!("Dublicate source with id \"{}\"", &source.id) )),
                    Entry::Vacant(entry) => {
                        entry.insert(Arc::new(source));
                    },
                }
            }
        }

        //find source synonyms
        for source_synonym in mesh.children.iter(){
            if source_synonym.name.as_str()=="vertices" {
                let new_id=source_synonym.get_attribute("id")?;
                let existing_id=source_synonym.get_element("input")?.get_attribute("source")?.trim_left_matches('#');

                let source=match sources.get(existing_id){
                    Some(s) => s.clone(),
                    None => return Err(Error::Other( format!("Source with id \"{}\" does not exists", existing_id) )),
                };

                match sources.entry(new_id.clone()){
                    Entry::Occupied(_) => return Err(Error::Other( format!("Dublicate source synonym with id \"{}\"", new_id) )),
                    Entry::Vacant(entry) => {
                        entry.insert(source);
                    },
                }
            }
        }

        Ok(sources)
    }

    pub fn select_sources_and_generate_vertex_format(polylist:&Element, sources:&HashMap<String,Arc<Source>>) -> Result<(Vec<(String,Arc<Source>)>,String,String),Error>{
        let mut poly_sources=Vec::new();
        let mut vertex_format=String::new();
        let mut short_vertex_format=String::new();

        for input_element in polylist.children.iter(){
            if input_element.name.as_str()=="input" {
                let source_semantic=input_element.get_attribute("semantic")?;
                let source_id=input_element.get_attribute("source")?.trim_left_matches('#');
                let offset=input_element.parse_attribute_as_usize("offset")?;

                if offset!=poly_sources.len(){
                    return Err(Error::Other( format!("Expected source offset {}, but {} have been found", poly_sources.len(), offset) ));
                }

                let source=match sources.get(source_id){
                    Some(s) => s.clone(),
                    None => return Err(Error::Other( format!("Source with id \"{}\" does not exists", source_id) )),
                };

                if vertex_format.as_str()!=""{
                    vertex_format.push(' ');
                }
                vertex_format.push_str(&format!("{}:&({})",source_semantic,source.vertex_format));

                if short_vertex_format.as_str()!=""{
                    short_vertex_format.push(' ');
                }
                short_vertex_format.push_str(&format!("&({})",source.short_vertex_format));

                poly_sources.push((source_semantic.clone(),source));
            }
        }

        Ok((poly_sources, short_vertex_format, vertex_format))
    }

    pub fn read_polygons(polylist:&Element) -> Result<(Vec<Polygon>,usize),Error>{//read polygons(<vcount> tag)
        let poly_count=polylist.parse_attribute_as_usize("count")?;
        let polygons_vcount=polylist.get_element("vcount")?.get_text()?;

        let mut polygons=Vec::with_capacity(poly_count);
        let mut vertices_count=0;

        for vertices_per_poly_count in polygons_vcount.split(' ').filter(|c|*c!="").take(poly_count) {
            let vppc=match vertices_per_poly_count.parse::<usize>(){
                Ok ( c ) => c,
                Err( _ ) => return Err(Error::Other( format!("Vertices per polygon {} as usize", vertices_per_poly_count) )),
            };

            polygons.push(
                Polygon{
                    first_vertex_index:vertices_count,
                    vertices_count:vppc,
                }
            );

            vertices_count+=vppc;
        }

        if polygons.len()!=poly_count {
            return Err(Error::Other( format!("Expected {} polygons, but {} has been read", poly_count, polygons.len()) ));
        }

        Ok((polygons,vertices_count))
    }

    pub fn read_vertices(polylist:&Element, vertices_count:usize, sources:&Vec<(String,Arc<Source>)>) -> Result<HashMap<String,Arc<VertexIndices>>,Error>{//read vertices(<p> tag)
        let sources_count=sources.len();
        let source_data_indices_per_vertex=polylist.get_element("p")?.get_text()?;

        let mut vertex_indices_indices=Vec::with_capacity(sources_count);
        for i in 0..sources_count{
            vertex_indices_indices.push(Vec::with_capacity(vertices_count));
        }

        let mut source_data_index=0;
        for data_index_per_vertex in source_data_indices_per_vertex.split(' ').filter(|c|*c!="").take(vertices_count*sources_count) {
            let dipv=match data_index_per_vertex.parse::<usize>(){
                Ok ( c ) => c,
                Err( _ ) => return Err(Error::Other( format!("source data index per vertex {} as usize", data_index_per_vertex) )),
            };

            vertex_indices_indices[source_data_index].push(dipv);

            source_data_index+=1;

            if source_data_index==sources_count {
                source_data_index=0;
            }
        }

        for vertex_indices in vertex_indices_indices.iter(){
            if vertex_indices.len()!=vertices_count {
                return Err(Error::Other( format!("Expected {} indices, but {} has been read", vertices_count, vertex_indices.len()) ));
            }
        }

        let mut vertex_indices=HashMap::new();

        for &(ref vertex_layer_name, ref source) in sources.iter().rev(){
            match vertex_indices.entry(vertex_layer_name.clone()){
                Entry::Occupied(_) => return Err(Error::Other( format!("Dublicate source with vertex_format \"{}\"",vertex_layer_name) )),
                Entry::Vacant(entry) => {
                    let vi=VertexIndices{
                        source:source.clone(),
                        indices:vertex_indices_indices.pop().unwrap(),
                    };

                    entry.insert( Arc::new(vi) );
                },
            }
        }

        Ok(vertex_indices)
    }

    pub fn print_tree(&self, last_geometry:bool, last_mesh:bool){
        use print_branch;
        use print_tab;

        print_tab(false);
        print_tab(last_geometry);
        print_branch(last_mesh);

        match self.material{
            Some(ref material) => {
                println!("Mesh material:\"{}\"", material);
            },
            None => {
                println!("Mesh no material");
            },
        }

        print_tab(false);
        print_tab(last_geometry);
        print_tab(last_mesh);
        print_branch(false);
        println!("Short vertex_format: {}",self.short_vertex_format);

        print_tab(false);
        print_tab(last_geometry);
        print_tab(last_mesh);
        print_branch(true);
        println!("Vertex");

        if self.vertex_indices.len()>1 {
            for (ref name,ref vertex_indices) in self.vertex_indices.iter().take(self.vertex_indices.len()-1){
                print_tab(false);
                print_tab(last_geometry);
                print_tab(last_mesh);
                print_tab(true);
                print_branch(false);
                println!("Vertex indices for \"{}\" source id:\"{}\"",name,vertex_indices.source.id);
            }
        }

        match self.vertex_indices.iter().last(){
            Some((ref name,ref vertex_indices)) => {
                print_tab(false);
                print_tab(last_geometry);
                print_tab(last_mesh);
                print_tab(true);
                print_branch(true);
                println!("Vertex indices for \"{}\" source id:\"{}\"",name,vertex_indices.source.id);
            }
            None => {},
        }
    }
}
