use super::flags::NodeBitFlags;
use super::math::{apply_zero_signs, euler_to_matrix, extract_zero_signs, IDENTITY_MATRIX, PI};
use super::node::NodeC;
use super::types::{NodeVariants, Object3d, Transformation, ZONE_DEFAULT};
use super::wrappers::Object3dWrapper;
use crate::assert::assert_all_zero;
use crate::io_ext::{ReadHelper, WriteHelper};
use crate::size::ReprSize;
use crate::types::{Matrix, Vec3};
use crate::{assert_that, static_assert_size, Result};
use std::io::{Read, Write};

#[repr(C)]
struct Object3dC {
    flags: u32,
    opacity: f32,
    zero008: f32,
    zero012: f32,
    zero016: f32,
    zero020: f32,
    rotation: Vec3,    // 024
    scale: Vec3,       // 032
    matrix: Matrix,    // 048
    translation: Vec3, // 084
    zero096: [u8; 48],
}
static_assert_size!(Object3dC, 144);

fn assert_node_variants(node: &NodeVariants, offset: u32) -> Result<()> {
    // cannot assert name
    let contains_base = node.flags.contains(NodeBitFlags::BASE);
    assert_that!("flags", contains_base == true, offset + 36)?;
    assert_that!("field 044", node.unk044 == 1, offset + 44)?;
    if node.zone_id != ZONE_DEFAULT {
        assert_that!("zone id", 1 <= node.zone_id <= 80, offset + 48)?;
    }
    assert_that!("data ptr", node.data_ptr != 0, offset + 56)?;
    if node.flags.contains(NodeBitFlags::HAS_MESH) {
        assert_that!("mesh index", node.mesh_index > 0, offset + 60)?;
    } else {
        //assert_that!("mesh index", node.mesh_index == -1, offset + 60)?;
    }
    // can have area partition, parent, children
    assert_that!("field 196", node.unk196 == 160, offset + 196)?;
    Ok(())
}

fn assert_object3d(object3d: Object3dC, offset: u32) -> Result<Option<Transformation>> {
    assert_that!("flags", object3d.flags in [32u32, 40u32], offset + 0)?;
    assert_that!("opacity", object3d.opacity == 0.0, offset + 4)?;
    assert_that!("field 008", object3d.zero008 == 0.0, offset + 8)?;
    assert_that!("field 012", object3d.zero012 == 0.0, offset + 12)?;
    assert_that!("field 016", object3d.zero016 == 0.0, offset + 16)?;
    assert_that!("field 020", object3d.zero020 == 0.0, offset + 20)?;
    assert_that!("scale", object3d.scale == Vec3(1.0, 1.0, 1.0), offset + 36)?;
    assert_all_zero("field 096", offset + 96, &object3d.zero096)?;

    let transformation = if object3d.flags == 40 {
        assert_that!(
            "rotation",
            object3d.rotation == Vec3(0.0, 0.0, 0.0),
            offset + 24
        )?;
        assert_that!(
            "translation",
            object3d.translation == Vec3(0.0, 0.0, 0.0),
            offset + 84
        )?;
        assert_that!("matrix", object3d.matrix == IDENTITY_MATRIX, offset + 48)?;
        None
    } else {
        let rotation = object3d.rotation;
        assert_that!("rotation x", -PI <= rotation.0 <= PI, offset + 24)?;
        assert_that!("rotation y", -PI <= rotation.1 <= PI, offset + 28)?;
        assert_that!("rotation z", -PI <= rotation.2 <= PI, offset + 32)?;
        let translation = object3d.translation;

        let expected_matrix = euler_to_matrix(&rotation);
        // in most cases, the calculated matrix is correct :/ for 2%, this fails
        let matrix = if expected_matrix == object3d.matrix {
            None
        } else {
            Some(object3d.matrix)
        };

        Some(Transformation {
            rotation,
            translation,
            matrix,
        })
    };
    Ok(transformation)
}

pub fn read_object3d<R>(
    read: &mut R,
    node: NodeVariants,
    offset: &mut u32,
) -> Result<Object3dWrapper>
where
    R: Read,
{
    assert_node_variants(&node, *offset)?;
    *offset += NodeC::SIZE;

    let object3dc: Object3dC = read.read_struct()?;
    let matrix_signs = extract_zero_signs(&object3dc.matrix);
    let transformation = assert_object3d(object3dc, *offset)?;
    *offset += Object3dC::SIZE;

    let object3d = Object3d {
        name: node.name,
        flags: node.flags.into(),
        zone_id: node.zone_id,
        mesh_index: node.mesh_index,
        area_partition: node.area_partition,
        transformation,
        matrix_signs,
        parent: None,
        children: None,
        data_ptr: node.data_ptr,
        parent_array_ptr: node.parent_array_ptr,
        children_array_ptr: node.children_array_ptr,
        unk116: node.unk116,
        unk140: node.unk140,
        unk164: node.unk164,
    };

    Ok(Object3dWrapper {
        wrapped: object3d,
        has_parent: node.has_parent,
        children_count: node.children_count,
    })
}

pub fn node_object3d(object3d: &Object3d) -> NodeVariants {
    let flags = NodeBitFlags::from(&object3d.flags);
    let children_count = object3d
        .children
        .as_ref()
        .map(|children| children.len() as u32)
        .unwrap_or(0);
    NodeVariants {
        name: object3d.name.clone(),
        flags,
        unk044: 1,
        zone_id: object3d.zone_id,
        data_ptr: object3d.data_ptr,
        mesh_index: object3d.mesh_index,
        area_partition: object3d.area_partition,
        has_parent: object3d.parent.is_some(),
        parent_array_ptr: object3d.parent_array_ptr,
        children_count,
        children_array_ptr: object3d.children_array_ptr,
        unk116: object3d.unk116,
        unk140: object3d.unk140,
        unk164: object3d.unk164,
        unk196: 160,
    }
}

pub fn write_object3d<W>(write: &mut W, object3d: &Object3d) -> Result<()>
where
    W: Write,
{
    let (flags, rotation, translation, matrix) = object3d
        .transformation
        .as_ref()
        .map(|tr| {
            let matrix = tr
                .matrix
                .as_ref()
                .map(|matrix| matrix.clone())
                .unwrap_or_else(|| euler_to_matrix(&tr.rotation));
            (32, tr.rotation.clone(), tr.translation.clone(), matrix)
        })
        .unwrap_or((
            40,
            Vec3(0.0, 0.0, 0.0),
            Vec3(0.0, 0.0, 0.0),
            IDENTITY_MATRIX,
        ));

    let matrix = apply_zero_signs(&matrix, object3d.matrix_signs);

    write.write_struct(&Object3dC {
        flags,
        opacity: 0.0,
        zero008: 0.0,
        zero012: 0.0,
        zero016: 0.0,
        zero020: 0.0,
        rotation,
        scale: Vec3(1.0, 1.0, 1.0),
        matrix,
        translation,
        zero096: [0u8; 48],
    })?;
    Ok(())
}
