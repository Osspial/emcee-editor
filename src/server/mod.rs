pub mod world;

use self::world::World;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Server {
    pub world: World
}
