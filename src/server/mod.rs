pub mod world;

use cgmath::{Point3, Quaternion};

use std::collections::HashMap;

use self::world::{World, RoomID};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EmceeServer {
    pub cameras: HashMap<CameraID, Camera>,
    pub world: World
}

id!(CameraID);

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct Camera {
    pub id: CameraID,

    pub pos: Point3<f32>,
    pub rot: Quaternion<f32>,
    pub in_room: RoomID,
}
