use cgmath::{Point2, Point3, Quaternion};
use cgmath_geometry::DimsBox;

use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct World {
    pub rooms: HashMap<RoomID, Room>,
    pub portals: HashMap<PortalID, Portal>,
}

id!(RoomID);
id!(PortalID);

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Room {
    pub id: RoomID,
    pub dims: DimsBox<Point3<f32>>,
    pub portals: Vec<RoomPortal>
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct RoomPortal {
    pub id: PortalID,
    pub pos: Point3<f32>,
    pub rot: Quaternion<f32>
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct Portal {
    pub id: PortalID,
    pub dims: DimsBox<Point2<f32>>,
    pub rooms_linked: [RoomID; 2]
}
