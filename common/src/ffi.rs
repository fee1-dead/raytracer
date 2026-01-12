use crate::aabb::AxisAlignedBoundingBox;


pub struct BvhNode {
    pub left: &'static BvhNode,
    pub right: &'static BvhNode,
    pub bbox: AxisAlignedBoundingBox,
}
