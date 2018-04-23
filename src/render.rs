use cgmath::{Point2, Point3, Vector3, Matrix3, Matrix4, EuclideanSpace, InnerSpace, PerspectiveFov};
use cgmath_geometry::{OffsetBox, GeoBox};
use gullery::{
    ContextState,
    buffers::{Buffer, BufferUsage},
    vao::VertexArrayObj,
    program::{Program, Shader},
    framebuffer::{DrawMode, Framebuffer},
    colors::{Rgb, Rgba},
    glsl::Nu8,
    render_state
};
use server::{
    Server,
    world::{RoomID, CameraID, Room, Camera}
};
use std::{
    rc::Rc,
    cell::RefCell,
    collections::HashMap,

    fs::File,
    io::{self, prelude::*}
};


pub struct WorldRender {
    server: Rc<RefCell<Server>>,
    context_state: Rc<ContextState>,
    world_geometry_program: Program<GeometryVertex, GeometryUniforms>,
    room_geometry: HashMap<RoomID, VertexArrayObj<GeometryVertex, u16>>
}

#[derive(TypeGroup, Clone, Copy, Debug)]
struct GeometryVertex {
    pos: Point3<f32>,
    face_color: Rgb<Nu8>
}

#[derive(Uniforms, Clone, Copy)]
struct GeometryUniforms {
    transform_matrix: Matrix4<f32>
}

impl WorldRender {
    pub fn new(server: Rc<RefCell<Server>>, context_state: Rc<ContextState>) -> Result<WorldRender, io::Error> {
        let load_shader_src: fn(&str) -> Result<String, io::Error> = |path| {
            let mut file = File::open(path)?;
            let mut src = String::new();
            file.read_to_string(&mut src)?;
            Ok(src)
        };

        // Load up the shaders and compile them into the world render program.
        let world_geometry_vert_shader = Shader::new(
            &load_shader_src("./assets/shaders/world_geometry.vert")?,
            context_state.clone()
        ).map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))?;

        let world_geometry_frag_shader = Shader::new(
            &load_shader_src("./assets/shaders/world_geometry.frag")?,
            context_state.clone())
        .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))?;

        let (world_geometry_program, warnings) = Program::new(
            &world_geometry_vert_shader,
            None,
            &world_geometry_frag_shader
        ).map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))?;

        for w in warnings {
            warn!(target: "gl_program_warnings", "{}", w);
        }

        Ok(WorldRender {
            server, context_state,
            world_geometry_program,
            room_geometry: HashMap::new()
        })
    }

    pub fn draw_world<F: Framebuffer>(&mut self, camera_id: CameraID, fb_rect: OffsetBox<Point2<u32>>, framebuffer: &mut F) {
        let server = self.server.borrow();
        let context_state = self.context_state.clone();

        let camera = match server.world.cameras.get(&camera_id) {
            Some(c) => c,
            None => return
        };
        let active_room_id = match camera.in_room.or(server.world.rooms.keys().next().cloned()) {
            Some(id) => id,
            None => return
        };
        let active_room = match server.world.rooms.get(&active_room_id) {
            Some(room) => room,
            None => return
        };

        let invert_axis_matrix = Matrix3::new(
            1., 0., 0.,
            0., 0., -1.,
            0., 1., 0.,
        );
        let active_room_matrix =
            Matrix4::from(camera.rot) *
            Matrix4::from_translation(invert_axis_matrix * -camera.pos.to_vec());
        let perspective_matrix = Matrix4::from(PerspectiveFov {
            fovy: camera.fov,
            aspect: fb_rect.width() as f32 / fb_rect.height() as f32,
            near: camera.near,
            far: camera.far
        });
        let render_state = render_state::RenderState {
            cull: Some((render_state::CullFace::Back, render_state::FrontFace::Clockwise)),
            srgb: true,
            viewport: fb_rect,
            ..Default::default()
        };

        let room_iter = Some((active_room_id, active_room)).into_iter().chain(
            active_room.portals.iter()
                .flat_map(|room_portal| server.world.portals.get(&room_portal.id))
                .flat_map(|portal| portal.other_room(active_room_id))
                .flat_map(|room_id| server.world.rooms.get(&room_id).map(|room| (room_id, room)))
        );
        for (room_id, room) in room_iter {
            let room_vao = self.room_geometry.entry(room_id).or_insert_with(||
                VertexArrayObj::new(
                    Buffer::with_size(BufferUsage::DynamicDraw, 1024, context_state.clone()),
                    Buffer::with_size(BufferUsage::DynamicDraw, 1024, context_state.clone())
                )
            );
            let (room_verts, room_indices) = room_to_triangles(room);
            room_vao.vertex_buffer_mut().sub_data(0, &room_verts);
            room_vao.index_buffer_mut().sub_data(0, &room_indices);

            framebuffer.draw(
                DrawMode::Triangles,
                0..room_indices.len(),
                room_vao,
                &self.world_geometry_program,
                GeometryUniforms {
                    transform_matrix: perspective_matrix * active_room_matrix * Matrix4::from(invert_axis_matrix)
                },
                render_state
            );
        }
    }
}

// Clockwise winding order,
// Z is up
fn room_to_triangles(room: &Room) -> ([GeometryVertex; 8], [u16; 36]) {
    let (x, y, z) = (room.dims.dims.x, room.dims.dims.y, room.dims.dims.z);

    (
        [
            GeometryVertex {
                pos: Point3::new(0., 0., 0.),
                face_color: Rgb::new(Nu8(0), Nu8(0), Nu8(0))
            },
            GeometryVertex {
                pos: Point3::new(x , 0., 0.),
                face_color: Rgb::new(Nu8(255), Nu8(0), Nu8(0))
            },
            GeometryVertex {
                pos: Point3::new(x , y, 0.),
                face_color: Rgb::new(Nu8(255), Nu8(255), Nu8(0))
            },
            GeometryVertex {
                pos: Point3::new(0., y , 0.),
                face_color: Rgb::new(Nu8(0), Nu8(255), Nu8(0))
            },
            GeometryVertex {
                pos: Point3::new(0., 0., z ),
                face_color: Rgb::new(Nu8(0), Nu8(0), Nu8(255))
            },
            GeometryVertex {
                pos: Point3::new(x , 0., z ),
                face_color: Rgb::new(Nu8(255), Nu8(0), Nu8(255))
            },
            GeometryVertex {
                pos: Point3::new(x , y , z ),
                face_color: Rgb::new(Nu8(255), Nu8(255), Nu8(255))
            },
            GeometryVertex {
                pos: Point3::new(0., y , z ),
                face_color: Rgb::new(Nu8(0), Nu8(255), Nu8(255))
            },
        ],
        [
            0, 3, 2,
                2, 1, 0,
            0, 1, 5,
                5, 4, 0,
            0, 4, 7,
                7, 3, 0,

            6, 7, 4,
                4, 5, 6,
            6, 5, 1,
                1, 2, 6,
            6, 2, 3,
                3, 7, 6,
        ]
    )
}
