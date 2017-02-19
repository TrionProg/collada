use Error;
use XMLElement;
use xmltree::Element;

use std::collections::HashMap;
use std::collections::hash_map::Entry;
use std::rc::Rc;

use Mesh;

pub struct Geometry{
    pub id:String,
    pub name:String,
    pub meshes:Vec<Mesh>,
}

impl Geometry{
    pub fn parse(geometry:&Element) -> Result<Geometry,Error>{
        let id=geometry.get_attribute("id")?.clone();
        let name=geometry.get_attribute("name")?.clone();

        let mut meshes=Vec::new();

        for (mesh_index,mesh_element) in geometry.children.iter().enumerate(){
            if mesh_element.name.as_str()=="mesh" {
                Mesh::parse_meshes(&mesh_element, &name, mesh_index, &mut meshes)?;
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

    pub fn print_tree(&self, last_geometry:bool){
        use print_branch;
        use print_tab;

        print_tab(false);
        print_branch(last_geometry);
        println!("Geometry id:\"{}\" name:\"{}\"",self.id,self.name);

        if self.meshes.len()>1 {
            for mesh in self.meshes.iter().take(self.meshes.len()-1){
                mesh.print_tree(last_geometry,false);
            }
        }

        match self.meshes.iter().last(){
            Some(mesh) => mesh.print_tree(last_geometry,true),
            None => {},
        }
    }
}

pub fn parse_geometries(root:&Element) -> Result< HashMap<String,Rc<Geometry>>, Error>{
    let geometries_element=root.get_element("library_geometries")?;
    let mut geometries=HashMap::new();

    for geometry_element in geometries_element.children.iter(){
        if geometry_element.name.as_str()=="geometry" {
            let geometry=Geometry::parse(&geometry_element)?;

            match geometries.entry(geometry.id.clone()){
                Entry::Occupied(_) => return Err(Error::Other( format!("Dublicate geometry with id \"{}\"", &geometry.id) )),
                Entry::Vacant(entry) => { entry.insert(Rc::new(geometry)); },
            }
        }
    }

    Ok(geometries)
}
