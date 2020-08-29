use crate::io_ext::{ReadHelper, WriteHelper};
use crate::mesh::{read_mesh_data, read_mesh_info, write_mesh_data, write_mesh_info, Mesh};
use crate::nodes::{
    read_node as read_node_wrapped, write_node as write_node_wrapped, Node, WrappedNode,
};
use crate::{assert_that, Result};
use ::serde::{Deserialize, Serialize};
use std::io::{Read, Write};

const VERSION: u32 = 27;
const FORMAT: u32 = 1;

#[derive(Debug, Serialize, Deserialize)]
pub struct Model {
    root: Node,
    meshes: Vec<Mesh>,
}

pub fn read_version<R>(read: &mut R) -> Result<()>
where
    R: Read,
{
    let version = read.read_u32()?;
    assert_that!("version", version == VERSION, 0)?;
    read.assert_end()
}

pub fn read_format<R>(read: &mut R) -> Result<()>
where
    R: Read,
{
    let format = read.read_u32()?;
    assert_that!("format", format == FORMAT, 0)?;
    read.assert_end()
}

pub fn write_version<W>(write: &mut W) -> Result<()>
where
    W: Write,
{
    write.write_u32(VERSION)?;
    Ok(())
}

pub fn write_format<W>(write: &mut W) -> Result<()>
where
    W: Write,
{
    write.write_u32(FORMAT)?;
    Ok(())
}

fn read_node<R>(read: &mut R, offset: &mut u32, meshes: &mut Vec<Mesh>) -> Result<Node>
where
    R: Read,
{
    match read_node_wrapped(read, offset)? {
        WrappedNode::Object3d(wrapped) => {
            let mut object3d = wrapped.wrapped;
            if object3d.mesh_index != 0 {
                let wrapped_mesh = read_mesh_info(read, offset)?;
                let mesh = read_mesh_data(read, offset, wrapped_mesh)?;
                meshes.push(mesh);
            }

            // we have to apply this, so data is written out correctly again, even if
            // the mechlib data doesn't read/write parents
            object3d.parent = if wrapped.has_parent { Some(0) } else { None };

            object3d.children = if wrapped.children_count > 0 {
                let children = (0..wrapped.children_count)
                    .into_iter()
                    .map(|_| read_node(read, offset, meshes))
                    .collect::<Result<Vec<_>>>()?;
                Some(children)
            } else {
                None
            };

            Ok(Node::Object3d(object3d))
        }
    }
}

pub fn read_model<R>(read: &mut R) -> Result<Model>
where
    R: Read,
{
    let mut offset = 0;
    let mut meshes = vec![];
    let root = read_node(read, &mut offset, &mut meshes)?;
    read.assert_end()?;
    Ok(Model { root, meshes })
}

fn write_node<W>(write: &mut W, node: &Node, meshes: &[Mesh], mesh_index: &mut usize) -> Result<()>
where
    W: Write,
{
    write_node_wrapped(write, node)?;
    match node {
        Node::Object3d(object3d) => {
            if object3d.mesh_index != 0 {
                let mesh = &meshes[*mesh_index];
                write_mesh_info(write, mesh)?;
                write_mesh_data(write, mesh)?;
                *mesh_index += 1;
            }

            if let Some(children) = &object3d.children {
                for child in children {
                    write_node(write, child, meshes, mesh_index)?;
                }
            }
        }
    }
    Ok(())
}

pub fn write_model<W>(write: &mut W, model: &Model) -> Result<()>
where
    W: Write,
{
    let mut mesh_index = 0;
    write_node(write, &model.root, &model.meshes, &mut mesh_index)
}
