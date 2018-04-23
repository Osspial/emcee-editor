use cgmath::{Deg, Rad, Point2, Point3, Quaternion, Euler};
use cgmath_geometry::DimsBox;

use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct World {
    pub cameras: HashMap<CameraID, Camera>,
    pub rooms: HashMap<RoomID, Room>,
    pub portals: HashMap<PortalID, Portal>,
}

id!(RoomID);
id!(PortalID);
id!(CameraID);

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

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct Camera {
    pub id: CameraID,

    pub fov: Rad<f32>,
    pub near: f32,
    pub far: f32,

    pub pos: Point3<f32>,
    pub rot: Euler<Rad<f32>>,
    pub in_room: Option<RoomID>,
}

impl Portal {
    pub fn other_room(&self, first_room: RoomID) -> Option<RoomID> {
        match () {
            _ if first_room == self.rooms_linked[0] => Some(self.rooms_linked[1]),
            _ if first_room == self.rooms_linked[1] => Some(self.rooms_linked[0]),
            _ => None
        }
    }
}

impl World {
    pub fn new_empty() -> World {
        let mut cameras = HashMap::new();
        let default_camera = Camera::new();
        cameras.insert(default_camera.id, default_camera);

        World {
            cameras,
            rooms: HashMap::new(),
            portals: HashMap::new()
        }
    }
}

impl Camera {
    pub fn new() -> Camera {
        Camera {
            id: CameraID::new(),

            fov: Deg(90.0).into(),
            near: 0.1,
            far: 2048.,

            pos: Point3::new(96., 96., 96.),
            rot: Euler::new(Rad(0.), Rad(0.), Rad(0.)),
            in_room: None
        }
    }
}
