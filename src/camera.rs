use cgmath::{
    ortho, point3, vec3, Deg, EuclideanSpace, Matrix4, Quaternion, Rotation3, SquareMatrix,
};

use crate::types::Position;

#[derive(Debug)]
pub struct Camera {
    pub pos: Position,
    pub quat: Quaternion<f32>,
    pub projection_kind: CameraProjectionKind,
    pub(crate) projection: Matrix4<f32>,
    // OPTIMIZE one view instead of two
    pub(crate) view: Matrix4<f32>,
    pub(crate) model_view: Matrix4<f32>,
}

#[allow(dead_code)]
#[derive(Debug)]
pub enum CameraProjectionKind {
    Perspective {
        aspect_ratio: f32,
        near: f32,
        far: f32,
        fov_y: f32, // OPTIMIZE use Degree instead of f32 ?
    },
    Orthographic {
        left: f32,
        right: f32,
        bottom: f32,
        top: f32,
        near: f32,
        far: f32,
    },
}

#[allow(dead_code)]
impl Camera {
    pub(crate) fn orthographic() -> Self {
        let mut camera = Self::default();
        camera.projection_kind = CameraProjectionKind::Orthographic {
            left: -1.0,
            right: 1.0,
            bottom: -1.0,
            top: 1.0,
            near: 0.1,
            far: 100.0,
        };
        camera
    }

    pub fn update(&mut self) {
        match self.projection_kind {
            CameraProjectionKind::Perspective {
                aspect_ratio,
                near,
                far,
                fov_y,
            } => {
                self.projection = cgmath::perspective(Deg(fov_y), aspect_ratio, near, far);
            }
            CameraProjectionKind::Orthographic {
                left,
                right,
                bottom,
                top,
                near,
                far,
            } => {
                self.projection = ortho(left, right, bottom, top, near, far);
            }
        }
        // OPTIMIZE check if this is necessary
        self.projection.y.y *= -1.0;
        self.view = Matrix4::from(self.quat.conjugate());
        self.model_view = self.view * Matrix4::from_translation(self.pos.to_vec() * -1.0);
        // OPTIMIZE it's possible to use the same view matrix for skybox and scene
    }

    pub fn set_aspect_ratio(&mut self, aspect_ratio: f32) {
        match self.projection_kind {
            CameraProjectionKind::Perspective {
                aspect_ratio: _,
                near,
                far,
                fov_y,
            } => {
                self.projection_kind = CameraProjectionKind::Perspective {
                    aspect_ratio,
                    near,
                    far,
                    fov_y,
                };
            }
            _ => {}
        }
        self.update();
    }
}

impl Default for Camera {
    fn default() -> Self {
        let mut camera = Self {
            pos: point3(0.0, 0.0, 0.0),
            quat: Quaternion::from_axis_angle(vec3(0.0, 1.0, 0.0), Deg(0.0)),
            projection_kind: CameraProjectionKind::Perspective {
                aspect_ratio: 16.0 / 9.0,
                fov_y: 45.0,
                near: 0.1,
                far: 100.0,
            },
            projection: Matrix4::identity(),
            model_view: Matrix4::identity(),
            view: Matrix4::identity(),
        };
        camera.update();
        camera
    }
}
