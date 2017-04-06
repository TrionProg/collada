use Error;
use XMLElement;
use xmltree::Element;

use std::collections::HashMap;
use std::collections::hash_map::Entry;
use std::sync::Arc;

use Mesh;
use Asset;
use TreePrinter;

pub struct Geometry{
    pub id:String,
    pub name:String,
    pub meshes:Vec<Arc<Mesh>>,
}

impl Geometry{
    pub fn parse(geometry:&Element, mesh_id:&mut usize, asset:&Asset) -> Result<Geometry,Error>{
        let id=geometry.get_attribute("id")?.clone();
        let name=geometry.get_attribute("name")?.clone();

        let mut meshes=Vec::new();

        for (mesh_index,mesh_element) in geometry.children.iter().enumerate(){
            if mesh_element.name.as_str()=="mesh" {
                Mesh::parse_meshes(&mesh_element, &name, mesh_index, mesh_id, &mut meshes, asset)?;
            }
        }

        Ok(
            Geometry{
                id:id,
                name:name,
                meshes:meshes,
            }
        )
    }

    pub fn print(&self, printer:TreePrinter){
        println!("Geometry id:\"{}\" name:\"{}\"",self.id,self.name);

        self.print_meshes( printer.new_branch(true) );
    }

    fn print_meshes(&self, printer:TreePrinter) {
        println!("Meshes");

        for (last,mesh) in self.meshes.iter().clone().enumerate().map(|i| (i.0==self.meshes.len()-1,i.1) ){
            mesh.print( printer.new_branch(last) );
        }
    }
}

pub fn parse_geometries(root:&Element, asset:&Asset) -> Result< HashMap<String,Arc<Geometry>>, Error>{
    let geometries_element=match root.get_element("library_geometries") {
        Ok( geometries_element ) => geometries_element,
        Err( _ ) => return Ok( HashMap::new() ),
    };
    
    let mut geometries=HashMap::new();

    let mut mesh_id=0;

    for geometry_element in geometries_element.children.iter(){
        if geometry_element.name.as_str()=="geometry" {
            let geometry=Geometry::parse(&geometry_element, &mut mesh_id, asset)?;

            match geometries.entry(geometry.id.clone()){
                Entry::Occupied(_) => return Err(Error::Other( format!("Dublicate geometry with id \"{}\"", &geometry.id) )),
                Entry::Vacant(entry) => { entry.insert(Arc::new(geometry)); },
            }
        }
    }

    Ok(geometries)
}
