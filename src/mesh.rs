use Error;
use XMLElement;
use xmltree::Element;

use std::rc::Rc;

use std::collections::HashMap;
use std::collections::hash_map::Entry;

use Source;

pub struct Polygon{
    pub first_vertex_index:usize,
    pub vertices_count:usize,
}

pub struct Mesh{
    short_semantics:String,
    full_semantics:String,
    sources:Vec<(String,Rc<Source>)>,
    polygons:Vec<Polygon>,
    vertices:Vec<Vec<usize>>,
}

//TODO полилистов несколько. у каждого свои полигоны и вершины, а select sources должен быть отдельный для каждого самантика тоже разная. по сути разные меши выйдут. и правильно, что сорсы Rc

impl Mesh{
    pub fn parse(mesh:&Element) -> Result<Mesh,Error>{
        let polylist=mesh.get_element("polylist")?;

        let (sources, short_semantics, full_semantics)=Mesh::select_sources_and_generate_semantics(&polylist,
            Mesh::read_sources(mesh)?
        )?;

        let (polygons,vertices_count)=Mesh::read_polygons(&polylist)?;
        let vertices=Mesh::read_vertices(&polylist, vertices_count, sources.len())?;

        println!("{} {} {}",polygons.len(),polygons[0].vertices_count,&full_semantics);

        Ok(
            Mesh{
                short_semantics:short_semantics,
                full_semantics:full_semantics,
                sources:sources,
                polygons:polygons,
                vertices:vertices,
            }
        )
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

    pub fn select_sources_and_generate_semantics(polylist:&Element, sources:HashMap<String,Rc<Source>>) -> Result<(Vec<(String,Rc<Source>)>,String,String),Error>{
        let poly_count=polylist.parse_attribute_as_usize("count")?;

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

                let (full_source_semantics,short_source_semantics)={
                    let mut full_source_semantics=String::new();
                    let mut short_source_semantics=String::new();

                    for source_layer in source.layers.iter().take(source.layers.len()-1){
                        full_source_semantics.push_str( &format!("{}:{},",source_layer.layer_type.print_semantics(), source_layer.data.print_semantics()) );
                        short_source_semantics.push_str( &format!("{},",source_layer.layer_type.print_semantics()) );
                    }

                    let source_layer=match source.layers.iter().last(){
                        Some(sl) => sl,
                        None => {unreachable!()},
                    };

                    full_source_semantics.push_str( &format!("{}:{}",source_layer.layer_type.print_semantics(), source_layer.data.print_semantics()) );
                    short_source_semantics.push_str( &format!("{}",source_layer.layer_type.print_semantics()) );

                    (full_source_semantics,short_source_semantics)
                };

                if full_semantics.as_str()!=""{
                    full_semantics.push(' ');
                }
                full_semantics.push_str(&format!("{}:&({})",source_semantic,full_source_semantics));

                if short_semantics.as_str()!=""{
                    short_semantics.push(' ');
                }
                short_semantics.push_str(&format!("&({})",short_source_semantics));

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
                Err( e ) => return Err(Error::Other( format!("Vertices per polygon {} as usize", vertices_per_poly_count) )),
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

    pub fn read_vertices(polylist:&Element, vertices_count:usize, sources_count:usize) -> Result<Vec<Vec<usize>>,Error>{//read vertices(<p> tag)
        let source_data_indexes_per_vertex=polylist.get_element("p")?.get_text()?;

        let mut source_data_indexes=Vec::with_capacity(sources_count);

        for i in 0..sources_count{
            source_data_indexes.push(Vec::with_capacity(vertices_count));
        }

        let mut sourceDataIndex=0;
        for source_data_index in source_data_indexes_per_vertex.split(' ').filter(|c|*c!="").take(vertices_count*sources_count) {
            let sdi=match source_data_index.parse::<usize>(){
                Ok ( c ) => c,
                Err( e ) => return Err(Error::Other( format!("source data index per vertex {} as usize", source_data_index) )),
            };

            source_data_indexes[sourceDataIndex].push(sdi);

            sourceDataIndex+=1;

            if sourceDataIndex==sources_count {
                sourceDataIndex=0;
            }
        }

        for sdi in source_data_indexes.iter(){
            if sdi.len()!=vertices_count {
                return Err(Error::Other( format!("Expected {} vertices, but {} has been read", vertices_count, sdi.len()) ));
            }
        }

        Ok(source_data_indexes)
    }
}
