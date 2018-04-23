#![feature(nll)]

extern crate cgmath;
extern crate cgmath_geometry;
extern crate uuid;

#[macro_use]
extern crate serde_derive;
extern crate serde;

#[macro_use]
extern crate derin_macros;
extern crate derin;

#[macro_use]
extern crate gullery_macros;
extern crate gullery;

#[macro_use]
extern crate log;
extern crate env_logger;

#[macro_use]
mod macros;

mod config;
mod gui;
mod server;
mod render;

use cgmath::{Angle, Quaternion, Euler, Rad, Vector3, Matrix3};
use server::{
    Server,
    world::{World, CameraID}
};
use std::{
    rc::Rc,
    cell::RefCell
};

#[derive(Debug, Clone, Copy, PartialEq)]
enum EmceeAction {
    CameraMove(CameraID, Vector3<f32>),
    CameraRotate(CameraID, Euler<Rad<f32>>)
}

fn main() {
    env_logger::init();

    let window_config = derin::WindowConfig {
        title: "Emcee Editor".to_string(),
        depth_bits: Some(24),
        stencil_bits: Some(8),
        ..Default::default()
    };

    let config = Rc::new(config::Config::default());

    let server = Rc::new(RefCell::new(Server {
        world: World::new_empty()
    }));

    let default_camera_id;
    {
        let mut server = server.borrow_mut();

        let base_room = server::world::Room {
            id: server::world::RoomID::new(),
            dims: cgmath_geometry::DimsBox::new3(128., 128., 128.),
            portals: Vec::new()
        };
        let (dcid, default_camera) = server.world.cameras.iter_mut().next().unwrap();
        default_camera_id = *dcid;
        default_camera.in_room = Some(base_room.id);

        server.world.rooms.insert(base_room.id, base_room);
    }

    let mut window = unsafe{ derin::Window::new(
        window_config,
        gui::RootGUI::new(server.clone(), config, default_camera_id),
        derin::theme::Theme::default()
    ).expect("Failed to create Emcee window") };
    let _: Option<()> = window.run_forever(
        |action, gui_root, _| {
            let mut server = server.borrow_mut();
            match action {
                EmceeAction::CameraMove(camera_id, move_dir) => {
                    if let Some(camera) = server.world.cameras.get_mut(&camera_id) {
                        println!("{:?}", move_dir);
                        let move_dir_rot = Vector3 {
                            x: camera.rot.y.sin() * (move_dir.z + move_dir.y) + camera.rot.y.cos() * move_dir.x,
                            y: camera.rot.y.cos() * (move_dir.z + move_dir.y) - camera.rot.y.sin() * move_dir.x,
                            z: -camera.rot.x.sin() * move_dir.z + camera.rot.x.cos() * move_dir.y
                        };
                        camera.pos += move_dir_rot;
                    }
                },
                EmceeAction::CameraRotate(camera_id, rotate) => {
                    if let Some(camera) = server.world.cameras.get_mut(&camera_id) {
                        camera.rot.x += rotate.x;
                        camera.rot.y += rotate.y;
                        camera.rot.z += rotate.z;

                        if camera.rot.x < -Rad::turn_div_4() {
                            camera.rot.x = -Rad::turn_div_4();
                        } else if Rad::turn_div_4() < camera.rot.x {
                            camera.rot.x = Rad::turn_div_4();
                        }
                        camera.rot.y %= Rad::full_turn();
                        camera.rot.z %= Rad::full_turn();
                    }
                },
            }
            derin::LoopFlow::Continue
        },
        |_, _| None
    );
}
