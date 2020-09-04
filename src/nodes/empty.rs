use super::flags::NodeBitFlags;
use super::types::{Empty, NodeVariant, NodeVariants, ZONE_DEFAULT};
use crate::{assert_that, Result};

pub fn assert_variants(node: NodeVariants, offset: u32) -> Result<NodeVariant> {
    // cannot assert name
    assert_that!("empty field 044", node.unk044 in [1, 3, 5, 7], offset + 56)?;
    assert_that!("empty zone id", node.zone_id in [1, ZONE_DEFAULT], offset + 56)?;
    assert_that!("empty data ptr", node.data_ptr == 0, offset + 56)?;
    assert_that!("empty mesh index", node.mesh_index == -1, offset + 60)?;
    assert_that!(
        "empty area partition",
        node.area_partition == None,
        offset + 76
    )?;
    assert_that!("empty has parent", node.has_parent == false, offset + 84)?;
    // parent array ptr is already asserted
    assert_that!(
        "empty children count",
        node.children_count == 0,
        offset + 92
    )?;
    // children array ptr is already asserted
    assert_that!("empty field 196", node.unk196 == 160, offset + 196)?;

    Ok(NodeVariant::Empty(Empty {
        name: node.name,
        flags: node.flags.into(),
        unk044: node.unk044,
        zone_id: node.zone_id,
        unk116: node.unk116,
        unk140: node.unk140,
        unk164: node.unk164,
        parent: 0,
    }))
}

pub fn make_variants(empty: &Empty) -> NodeVariants {
    NodeVariants {
        name: empty.name.clone(),
        flags: NodeBitFlags::from(&empty.flags),
        unk044: empty.unk044,
        zone_id: empty.zone_id,
        data_ptr: 0,
        mesh_index: -1,
        area_partition: None,
        has_parent: false,
        parent_array_ptr: 0,
        children_count: 0,
        children_array_ptr: 0,
        unk116: empty.unk116,
        unk140: empty.unk140,
        unk164: empty.unk164,
        unk196: 160,
    }
}

pub fn size() -> u32 {
    0
}
