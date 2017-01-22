use Error;
use XMLElement;
use xmltree::Element;

use Node;

pub struct Scene{
    pub id:String,
    pub name:String,
    pub nodes:Vec<Node>,
}

impl Scene{
    pub fn parse(scene:&Element) -> Result<Scene,Error>{
        let id=scene.get_attribute("id")?.clone();
        let name=scene.get_attribute("name")?.clone();

        let mut nodes=Vec::new();

        for node_element in scene.children.iter(){
            if node_element.name.as_str()=="node" {
                let node=Node::parse(node_element)?;

                nodes.push(node);
            }
        }

        Ok(
            Scene{
                id:id,
                name:name,
                nodes:nodes,
            }
        )
    }
}

pub fn parse_scenes(root:&Element) -> Result<Vec<Scene>, Error>{
    let scenes_element=root.get_element("library_visual_scenes")?;
    let mut scenes=Vec::new();

    for scene_element in scenes_element.children.iter(){
        if scene_element.name.as_str()=="visual_scene" {
            let scene=Scene::parse(scene_element)?;

            scenes.push(scene);
        }
    }

    Ok(scenes)
}
