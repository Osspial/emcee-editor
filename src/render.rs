use cgmath::{Point2, Point3, Vector3, Matrix3, Matrix4, EuclideanSpace, InnerSpace, PerspectiveFov, One, Quaternion, ElementWise, vec3};
use cgmath_geometry::{OffsetBox, DimsBox, GeoBox};
use gullery::{
    ContextState,
    buffers::{Buffer, BufferUsage},
    vao::VertexArrayObj,
    program::{Program, Shader},
    framebuffer::{DrawMode, Framebuffer},
    colors::{Rgb, Rgba},
    glsl::Nu8,
    render_state::*
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
    portal_geometry_program: Program<PortalVertex, GeometryUniforms>,
    room_geometry: HashMap<RoomID, VertexArrayObj<GeometryVertex, u16>>,
    portal_geometry: VertexArrayObj<PortalVertex, u16>
}

#[derive(TypeGroup, Clone, Copy, Debug)]
struct GeometryVertex {
    pos: Point3<f32>,
    face_color: Rgb<Nu8>
}

#[derive(TypeGroup, Clone, Copy, Debug)]
struct PortalVertex {
    pos: Point3<f32>
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
        let portal_geometry_vert_shader = Shader::new(
            &load_shader_src("./assets/shaders/portal_geometry.vert")?,
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

        let (portal_geometry_program, warnings) = Program::new(
            &portal_geometry_vert_shader,
            None,
            &world_geometry_frag_shader
        ).map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))?;

        for w in warnings {
            warn!(target: "gl_program_warnings", "{}", w);
        }

        let portal_geometry = VertexArrayObj::new(
            Buffer::with_size(BufferUsage::DynamicDraw, 4, context_state.clone()),
            Buffer::with_size(BufferUsage::DynamicDraw, 6, context_state.clone())
        );

        Ok(WorldRender {
            server, context_state,
            world_geometry_program,
            portal_geometry_program,
            room_geometry: HashMap::new(),
            portal_geometry
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
        let render_state = RenderState {
            cull: Some((CullFace::Back, FrontFace::Clockwise)),
            srgb: true,
            viewport: fb_rect,
            depth_test: Some(DepthStencilFunc::LEqual),
            ..Default::default()
        };

        {
            let portal_iter = active_room.portals.iter()
                .flat_map(|room_portal| server.world.portals.get(&room_portal.id).map(|portal| (portal, room_portal)));

            let portal_render_state = RenderState {
                stencil_test: Some(StencilTest {
                    func: DepthStencilFunc::Always,
                    frag_value: 1,
                    mask: !0,
                    stencil_fail: StencilOp::Zero,
                    depth_fail: StencilOp::Zero,
                    depth_pass: StencilOp::Replace
                }),
                color_mask: ColorMask::empty(),
                depth_mask: false,
                ..render_state
            };

            for (portal, room_portal) in portal_iter {
                let portal_offset_matrix =
                    Matrix4::from_translation(room_portal.pos.to_vec()) *
                    Matrix4::from(room_portal.rot);
                let transform_matrix = perspective_matrix * active_room_matrix * portal_offset_matrix * Matrix4::from(invert_axis_matrix);

                self.portal_geometry.vertex_buffer_mut().sub_data(0, &face_triangles(room_portal.pos, portal.dims, room_portal.rot));
                self.portal_geometry.index_buffer_mut().sub_data(0, &[2, 1, 0, 0, 3, 2]);
                framebuffer.draw(
                    DrawMode::Triangles,
                    0..6,
                    &self.portal_geometry,
                    &self.portal_geometry_program,
                    GeometryUniforms {
                        transform_matrix
                    },
                    portal_render_state
                );
            }
        }

        {
            let room_vao = self.room_geometry.entry(active_room_id).or_insert_with(||
                VertexArrayObj::new(
                    Buffer::with_size(BufferUsage::DynamicDraw, 1024, context_state.clone()),
                    Buffer::with_size(BufferUsage::DynamicDraw, 1024, context_state.clone())
                )
            );
            let (room_verts, room_indices) = room_to_triangles(active_room);
            room_vao.vertex_buffer_mut().sub_data(0, &room_verts);
            room_vao.index_buffer_mut().sub_data(0, &room_indices);

            let mut room_render_state = render_state;
            room_render_state.stencil_test = Some(StencilTest {
                func: DepthStencilFunc::Equal,
                frag_value: 0,
                mask: !0,
                stencil_fail: StencilOp::Keep,
                depth_fail: StencilOp::Keep,
                depth_pass: StencilOp::Keep
            });
            let room_offset_matrix = Matrix4::one();

            let transform_matrix = perspective_matrix * active_room_matrix * room_offset_matrix * Matrix4::from(invert_axis_matrix);

            framebuffer.draw(
                DrawMode::Triangles,
                0..room_indices.len(),
                room_vao,
                &self.world_geometry_program,
                GeometryUniforms {
                    transform_matrix
                },
                room_render_state
            );
        }

        let room_iter = active_room.portals.iter()
            .flat_map(|room_portal| server.world.portals.get(&room_portal.id).map(|portal| (portal, room_portal)))
            .flat_map(|(portal, room_portal)| portal.other_room(active_room_id).map(|room_id| (room_id, room_portal)))
            .flat_map(|(room_id, room_portal)| server.world.rooms.get(&room_id).map(|room| (room_id, room, room_portal)));

        for (room_id, room, room_portal) in room_iter {
            let room_vao = self.room_geometry.entry(room_id).or_insert_with(||
                VertexArrayObj::new(
                    Buffer::with_size(BufferUsage::DynamicDraw, 1024, context_state.clone()),
                    Buffer::with_size(BufferUsage::DynamicDraw, 1024, context_state.clone())
                )
            );
            let (room_verts, room_indices) = room_to_triangles(room);
            room_vao.vertex_buffer_mut().sub_data(0, &room_verts);
            room_vao.index_buffer_mut().sub_data(0, &room_indices);

            let mut room_render_state = render_state;
            room_render_state.stencil_test = Some(StencilTest {
                func: DepthStencilFunc::Equal,
                frag_value: 1,
                mask: !0,
                stencil_fail: StencilOp::Keep,
                depth_fail: StencilOp::Keep,
                depth_pass: StencilOp::Keep
            });
            let room_offset_matrix =
                Matrix4::from_translation(room_portal.pos.to_vec()) *
                Matrix4::from(room_portal.rot);

            let transform_matrix = perspective_matrix * active_room_matrix * room_offset_matrix * Matrix4::from(invert_axis_matrix);

            framebuffer.draw(
                DrawMode::Triangles,
                0..room_indices.len(),
                room_vao,
                &self.world_geometry_program,
                GeometryUniforms {
                    transform_matrix
                },
                room_render_state
            );
        }
    }
}

fn face_triangles(pos: Point3<f32>, dims: DimsBox<Point2<f32>>, rot: Quaternion<f32>) -> [PortalVertex; 4] {
    let corner_offset = Vector3::new(dims.width(), 0., dims.height());

    [
        PortalVertex {
            pos: pos + rot * corner_offset
        },
        PortalVertex {
            pos: pos + rot * (corner_offset.mul_element_wise(vec3(1., 0., -1.)))
        },
        PortalVertex {
            pos: pos + rot * (corner_offset.mul_element_wise(vec3(-1., 0., -1.)))
        },
        PortalVertex {
            pos: pos + rot * (corner_offset.mul_element_wise(vec3(-1., 0., 1.)))
        },
    ]
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
