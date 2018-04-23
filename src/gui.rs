use cgmath::{Point2, Vector3, Rad, Euler};
use cgmath_geometry::OffsetBox;
use derin::{
    gl_render::PrimFrame,
    event::{InputState, WidgetEvent, EventOps, MouseButton, FocusChange},
    widgets::{
        Group, DirectRender, DirectRenderState,
        custom::{WidgetIdent, popup::ChildPopupsMut}
    },
    layout::LayoutHorizontal
};
use gullery::{
    ContextState,
    framebuffer::DefaultFramebuffer
};
use server::{
    Server,
    world::CameraID
};
use EmceeAction;
use config::Config;
use render::WorldRender;
use std::{
    rc::Rc,
    cell::RefCell,
    time::Duration
};

#[derive(WidgetContainer)]
#[derin(action = "EmceeAction")]
pub struct RootGUI {
    pub world_render: DirectRender<WorldDirectRender>
}

pub struct WorldDirectRender {
    server: Rc<RefCell<Server>>,
    config: Rc<Config>,
    camera_id: CameraID,
    world_render: Option<WorldRender>,
}

impl RootGUI {
    pub fn new(server: Rc<RefCell<Server>>, config: Rc<Config>, default_camera: CameraID) -> Group<RootGUI, LayoutHorizontal> {
        Group::new(
            RootGUI {
                world_render: DirectRender::new(WorldDirectRender {
                    server,
                    config: config.clone(),
                    camera_id: default_camera,
                    world_render: None
                }, Some(config.refresh_rate))
            },
            LayoutHorizontal::default()
        )
    }
}

impl DirectRenderState<EmceeAction> for WorldDirectRender {
    type RenderType = (DefaultFramebuffer, OffsetBox<Point2<u32>>, Rc<ContextState>);
    fn render(&mut self, &mut (ref mut fb, viewport_rect, ref context_state): &mut Self::RenderType) {
        // If the world renderer hasn't been initialized, try to initialize it, and throw an error
        // if initialization failed for whatever reason.
        if let None = self.world_render {
            match WorldRender::new(self.server.clone(), context_state.clone()) {
                Ok(world_render) => self.world_render = Some(world_render),
                Err(create_error) => error!("{}", create_error)
            }
        }

        if let Some(ref mut world_render) = self.world_render {
            world_render.draw_world(self.camera_id, viewport_rect, fb);
        }
    }

    fn on_widget_event<F>(
        &mut self,
        event: WidgetEvent,
        input_state: InputState,
        _: Option<ChildPopupsMut<EmceeAction, F>>,
        _: &[WidgetIdent],
        refresh_rate: &mut Option<Duration>
    ) -> EventOps<EmceeAction, F>
        where F: PrimFrame<DirectRender=Self::RenderType>
    {
        let WorldDirectRender {
            ref config,
            ..
        } = *self;

        let (mut action, mut focus) = (None, None);
        let mut move_dir = Vector3::new(0., 0., 0.);

        match event {
            WidgetEvent::MouseDown{button, in_widget: true, ..}
                if Some(button) == self.config.camera_move_button
            => {
                focus = Some(FocusChange::Take);
            },
            WidgetEvent::MouseUp{button, in_widget: true, ..}
                if Some(button) == self.config.camera_move_button
            => {
                focus = Some(FocusChange::Remove);
            },
            _ => ()
        }
        if input_state.mouse_buttons_down_in_widget.iter().find(|d| Some(d.button) == self.config.camera_move_button).is_some() {
            match event {
                WidgetEvent::MouseMove{in_widget: true, old_pos, new_pos} => {
                    let sensitivity_rad = Rad::from(config.mouse_sensitivity);
                    let rotate_delta_px = new_pos - old_pos;
                    let rotate_euler = Euler {
                        y: sensitivity_rad * rotate_delta_px.x as f32,
                        x: sensitivity_rad * rotate_delta_px.y as f32,
                        z: Rad(0.0)
                    };
                    action = Some(EmceeAction::CameraRotate(self.camera_id, rotate_euler));
                },
                WidgetEvent::Timer{..} => {
                    for key in input_state.keys_down {
                        match () {
                            _ if Some(*key) == self.config.keybindings.move_forward => move_dir.z = 1.,
                            _ if Some(*key) == self.config.keybindings.move_backward => move_dir.z = -1.,
                            _ if Some(*key) == self.config.keybindings.move_left => move_dir.x = -1.,
                            _ if Some(*key) == self.config.keybindings.move_right => move_dir.x = 1.,
                            _ if Some(*key) == self.config.keybindings.move_up => move_dir.y = 1.,
                            _ if Some(*key) == self.config.keybindings.move_down => move_dir.y = -1.,
                            _ => ()
                        }
                    }
                    if move_dir != Vector3::new(0., 0., 0.) {
                        action = Some(EmceeAction::CameraMove(self.camera_id, move_dir));
                    }
                }
                _ => ()
            }

        }

        EventOps {
            action,
            focus,
            bubble: event.default_bubble(),
            cursor_pos: None,
            cursor_icon: None,
            popup: None
        }
    }
}
