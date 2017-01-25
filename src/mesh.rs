use Error;
use XMLElement;
use xmltree::Element;

use std::collections::HashMap;
use std::collections::hash_map::Entry;
use std::rc::Rc;

use Source;

pub struct Polygon{
    pub first_vertex_index:usize,
    pub vertices_count:usize,
}

pub struct Mesh{
    pub material:Option<String>,
    pub short_semantics:String,
    pub full_semantics:String,
    pub sources:Vec<(String,Rc<Source>)>,
    pub polygons:Vec<Polygon>,
    pub vertex_layers:HashMap<String,VertexLayer>,
}

pub struct VertexLayer{
    pub source:Rc<Source>,
    pub indexes:Vec<usize>,
}

//TODO:Material shoild be Rc

impl Mesh{
    pub fn parse_meshes(mesh:&Element, meshes:&mut Vec<Mesh>) -> Result<(),Error>{
        let all_sources=Mesh::read_sources(mesh)?;

        for polylist in mesh.children.iter(){
            if polylist.name.as_str()=="polylist"{
                let material=match polylist.attributes.get("material"){
                    Some(m) => Some(m.clone()),
                    None => None,
                };

                let (sources, short_semantics, full_semantics)=Mesh::select_sources_and_generate_semantics(&polylist, &all_sources)?;

                let (polygons,vertices_count)=Mesh::read_polygons(&polylist)?;
                let vertex_layers=Mesh::read_vertices(&polylist, vertices_count, &sources)?;

                meshes.push(
                    Mesh{
                        material:material,
                        short_semantics:short_semantics,
                        full_semantics:full_semantics,
                        sources:sources,
                        polygons:polygons,
                        vertex_layers:vertex_layers,
                    }
                );
            }
        }

        Ok(())
    }

    pub fn read_sources(mesh:&Element) -> Result<HashMap<String,Rc<Source>>,Error>{
        //read sources
        let mut sources=HashMap::new();

        for source_element in mesh.children.iter(){
            if source_element.name.as_str()=="source" {
                let source=Source::parse(&source_element)?;

                match sources.entry(source.id.clone()){
                    Entry::Occupied(_) => return Err(Error::Other( format!("Dublicate source with id \"{}\"", &source.id) )),
                    Entry::Vacant(entry) => {
                        entry.insert(Rc::new(source));
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

    pub fn select_sources_and_generate_semantics(polylist:&Element, sources:&HashMap<String,Rc<Source>>) -> Result<(Vec<(String,Rc<Source>)>,String,String),Error>{
        let mut poly_sources=Vec::new();
        let mut full_semantics=String::new();
        let mut short_semantics=String::new();

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

                if full_semantics.as_str()!=""{
                    full_semantics.push(' ');
                }
                full_semantics.push_str(&format!("{}:&({})",source_semantic,source.full_semantics));

                if short_semantics.as_str()!=""{
                    short_semantics.push(' ');
                }
                short_semantics.push_str(&format!("&({})",source.short_semantics));

                poly_sources.push((source_semantic.clone(),source));
            }
        }

        Ok((poly_sources, short_semantics, full_semantics))
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

    pub fn read_vertices(polylist:&Element, vertices_count:usize, sources:&Vec<(String,Rc<Source>)>) -> Result<HashMap<String,VertexLayer>,Error>{//read vertices(<p> tag)
        let sources_count=sources.len();
        let source_data_indexes_per_vertex=polylist.get_element("p")?.get_text()?;

        let mut vertex_layers_indexes=Vec::with_capacity(sources_count);
        for i in 0..sources_count{
            vertex_layers_indexes.push(Vec::with_capacity(vertices_count));
        }

        let mut source_data_index=0;
        for data_index_per_vertex in source_data_indexes_per_vertex.split(' ').filter(|c|*c!="").take(vertices_count*sources_count) {
            let dipv=match data_index_per_vertex.parse::<usize>(){
                Ok ( c ) => c,
                Err( _ ) => return Err(Error::Other( format!("source data index per vertex {} as usize", data_index_per_vertex) )),
            };

            vertex_layers_indexes[source_data_index].push(dipv);

            source_data_index+=1;

            if source_data_index==sources_count {
                source_data_index=0;
            }
        }

        for vertex_layer_indexes in vertex_layers_indexes.iter(){
            if vertex_layer_indexes.len()!=vertices_count {
                return Err(Error::Other( format!("Expected {} indexes, but {} has been read", vertices_count, vertex_layer_indexes.len()) ));
            }
        }

        let mut vertex_layers=HashMap::new();

        for &(ref vertex_layer_name, ref source) in sources.iter().rev(){
            match vertex_layers.entry(vertex_layer_name.clone()){
                Entry::Occupied(_) => return Err(Error::Other( format!("Dublicate source with semantic \"{}\"",vertex_layer_name) )),
                Entry::Vacant(entry) => {
                    entry.insert( VertexLayer{
                        source:source.clone(),
                        indexes:vertex_layers_indexes.pop().unwrap(),
                    });
                },
            }
        }

        Ok(vertex_layers)
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
        println!("Short semantics: {}",self.short_semantics);

        print_tab(false);
        print_tab(last_geometry);
        print_tab(last_mesh);
        print_branch(true);
        println!("Vertex");

        if self.vertex_layers.len()>1 {
            for (ref vertex_layers_name,ref vertex_layer) in self.vertex_layers.iter().take(self.vertex_layers.len()-1){
                print_tab(false);
                print_tab(last_geometry);
                print_tab(last_mesh);
                print_tab(true);
                print_branch(false);
                println!("Layer \"{}\" source id:\"{}\"",vertex_layers_name,vertex_layer.source.id);
            }
        }

        match self.vertex_layers.iter().last(){
            Some((ref vertex_layers_name,ref vertex_layer)) => {
                print_tab(false);
                print_tab(last_geometry);
                print_tab(last_mesh);
                print_tab(true);
                print_branch(true);
                println!("Layer \"{}\" source id:\"{}\"",vertex_layers_name,vertex_layer.source.id);
            }
            None => {},
        }
    }
}
