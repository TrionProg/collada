use Error;
use XMLElement;
use xmltree::Element;

use std::collections::HashMap;
use std::collections::hash_map::Entry;
use std::sync::Arc;

use Source;
use Asset;
use ArrayIter;
use TreePrinter;

use source::read_sources;
use source::select_sources;

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
        let all_sources=read_sources(mesh, asset)?;

        for polylist in mesh.children.iter(){
            if polylist.name.as_str()=="polylist"{
                let material=match polylist.attributes.get("material"){
                    Some(m) => Some(m.clone()),
                    None => None,
                };

                let sources=select_sources(&polylist,&all_sources)?;
                let (short_vertex_format, vertex_format)=Self::generate_vertex_format(&polylist,&sources)?;

                let (polygons,vertex_indices_count)=Self::read_polygons(&polylist)?;
                let vertex_indices=Self::read_vertex_indices(&polylist, vertex_indices_count, &sources)?;

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

    pub fn generate_vertex_format(polylist:&Element, sources_list:&Vec<(String,Arc<Source>)>) -> Result<(String,String),Error>{
        let mut vertex_format=String::new();
        let mut short_vertex_format=String::new();

        for &(ref name, ref source) in sources_list.iter(){
            if vertex_format.as_str()!=""{
                vertex_format.push(' ');
            }
            vertex_format.push_str(&format!("{}:&({})",name,source.vertex_format));

            if short_vertex_format.as_str()!=""{
                short_vertex_format.push(' ');
            }
            short_vertex_format.push_str(&format!("&({})",source.short_vertex_format));
        }

        Ok( (short_vertex_format, vertex_format) )
    }

    pub fn read_polygons(polylist:&Element) -> Result<(Vec<Polygon>,usize),Error>{//read polygons(<vcount> tag)
        let polygons_count=polylist.parse_attribute_as_usize("count")?;
        let polygons_vcount=polylist.get_element("vcount")?.get_text()?;

        let mut polygons=Vec::with_capacity(polygons_count);
        let mut vertex_indices_count=0;

        let mut array_iter=ArrayIter::new(polygons_vcount, polygons_count, "polygons");

        for i in 0..polygons_count {
            let vertices_per_polygon=array_iter.read_usize()?;

            polygons.push(
                Polygon{
                    first_vertex_index:vertex_indices_count,
                    vertices_count:vertices_per_polygon,
                }
            );

            vertex_indices_count+=vertices_per_polygon;
        }

        Ok((polygons,vertex_indices_count))
    }

    pub fn read_vertex_indices(polylist:&Element, vertex_indices_count:usize, sources:&Vec<(String,Arc<Source>)>) -> Result<HashMap<String,Arc<VertexIndices>>,Error>{//read vertices(<p> tag)
        let sources_count=sources.len();

        let source_data_indices_per_vertex=polylist.get_element("p")?.get_text()?;

        let mut vertex_indices_indices=Vec::with_capacity(sources_count);
        for i in 0..sources_count{
            vertex_indices_indices.push(Vec::with_capacity(vertex_indices_count));
        }

        let mut array_iter=ArrayIter::new(source_data_indices_per_vertex, vertex_indices_count*sources_count, "vertex indices");

        for i in 0..vertex_indices_count {
            for j in 0..sources_count {
                let data_index_per_vertex=array_iter.read_usize()?;

                vertex_indices_indices[j].push(data_index_per_vertex);
            }
        }

        let mut vertex_indices=HashMap::new();

        for &(ref source_name, ref source) in sources.iter().rev(){
            match vertex_indices.entry(source_name.clone()){
                Entry::Occupied(_) => return Err(Error::Other( format!("Duplicate source with name \"{}\"",source_name) )),
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

    pub fn print(&self, printer:TreePrinter) {
        match self.material{
            Some(ref material) => {
                println!("Mesh material:\"{}\"", material);
            },
            None => {
                println!("Mesh no material");
            },
        }

        printer.new_branch(false);
        println!("Short vertex_format: {}",self.short_vertex_format);

        self.print_vertex_indices( printer.new_branch(true) );
    }

    fn print_vertex_indices(&self, printer:TreePrinter){
        println!("Vertices");

        for (last,(ref name,ref vertex_indices)) in self.vertex_indices.iter().clone().enumerate().map(|i| (i.0==self.vertex_indices.len()-1,i.1) ){
            printer.new_branch(last);
            println!("Vertex indices for \"{}\" source id:\"{}\"",name,vertex_indices.source.id);
        }
    }
}
