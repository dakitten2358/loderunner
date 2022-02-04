use bevy::{
    prelude::*,
    reflect::Reflect,
    render::{
        camera::{camera_system, CameraPlugin, CameraProjection, DepthCalculation},
        primitives::Frustum,
        view::{update_frusta, VisibleEntities},
    },
    transform::TransformSystem,
};

#[derive(Bundle)]
pub struct ScalableOrthographicCameraBundle {
    pub camera: Camera,
    pub orthographic_projection: ScalableOrthographicProjection,
    pub visible_entities: VisibleEntities,
    pub frustum: Frustum,
    pub transform: Transform,
    pub global_transform: GlobalTransform,
}

impl ScalableOrthographicCameraBundle {
    pub fn new(width: f32, height: f32) -> Self {
        // we want 0 to be "closest" and +far to be "farthest" in 2d, so we offset
        // the camera's translation by far and use a right handed coordinate system
        let far = 1000.0;
        let orthographic_projection = ScalableOrthographicProjection {
            far,
            depth_calculation: DepthCalculation::ZDifference,
            virtual_width: width,
            virtual_height: height,
            ..Default::default()
        };
        let transform = Transform::from_xyz(0.0, 0.0, far - 0.1);
        let view_projection = orthographic_projection.get_projection_matrix() * transform.compute_matrix().inverse();
        let frustum = Frustum::from_view_projection(
            &view_projection,
            &transform.translation,
            &transform.back(),
            orthographic_projection.far(),
        );
        ScalableOrthographicCameraBundle {
            camera: Camera {
                name: Some(CameraPlugin::CAMERA_2D.to_string()),
                near: orthographic_projection.near,
                far: orthographic_projection.far,
                ..Default::default()
            },
            orthographic_projection,
            visible_entities: VisibleEntities::default(),
            frustum,
            transform,
            global_transform: Default::default(),
        }
    }
}

#[derive(Component, Debug, Clone, Reflect)]
#[reflect(Component)]
pub struct ScalableOrthographicProjection {
    pub left: f32,
    pub right: f32,
    pub bottom: f32,
    pub top: f32,
    pub near: f32,
    pub far: f32,
    pub scale: f32,
    pub depth_calculation: DepthCalculation,

    pub virtual_width: f32,
    pub virtual_height: f32,
}

impl CameraProjection for ScalableOrthographicProjection {
    fn get_projection_matrix(&self) -> Mat4 {
        Mat4::orthographic_rh(
            self.left * self.scale,
            self.right * self.scale,
            self.bottom * self.scale,
            self.top * self.scale,
            // NOTE: near and far are swapped to invert the depth range from [0,1] to [1,0]
            // This is for interoperability with pipelines using infinite reverse perspective projections.
            self.far,
            self.near,
        )
    }

    fn update(&mut self, width: f32, height: f32) {
        let desired_aspect_ratio = self.virtual_width / self.virtual_height;
        let actual_aspect_ratio = width / height;

        if actual_aspect_ratio >= desired_aspect_ratio {
            let full_width = self.virtual_width * (actual_aspect_ratio / desired_aspect_ratio);
            let half_width = full_width / 2.0;

            self.left = -half_width;
            self.right = half_width;
            self.top = self.virtual_height;
            self.bottom = 0.0;
        } else {
            let full_height = self.virtual_height * (desired_aspect_ratio / actual_aspect_ratio);
            let desired_height = self.virtual_width / desired_aspect_ratio;
            let extra_height = full_height - desired_height;

            self.left = self.virtual_width / -2.0;
            self.right = self.virtual_width / 2.0;
            self.top = full_height - (extra_height / 2.0);
            self.bottom = 0.0 - (extra_height / 2.0);
        }
    }

    fn depth_calculation(&self) -> DepthCalculation {
        self.depth_calculation
    }

    fn far(&self) -> f32 {
        self.far
    }
}

impl Default for ScalableOrthographicProjection {
    fn default() -> Self {
        ScalableOrthographicProjection {
            left: -1.0,
            right: 1.0,
            bottom: -1.0,
            top: 1.0,
            near: 0.0,
            far: 1000.0,
            scale: 1.0,
            depth_calculation: DepthCalculation::Distance,
            virtual_width: 640.0,
            virtual_height: 360.0,
        }
    }
}

#[derive(Debug, Hash, PartialEq, Eq, Clone, SystemLabel)]
pub enum ScalableOrthographicSystems {
    UpdateScalableOrthographicFrusta,
}

pub struct ScalableOrthographicCameraPlugin;

impl Plugin for ScalableOrthographicCameraPlugin {
    fn build(&self, app: &mut App) {
        use ScalableOrthographicSystems::*;

        app.register_type::<ScalableOrthographicProjection>();
        app.add_system_to_stage(
            CoreStage::PostUpdate,
            update_frusta::<ScalableOrthographicProjection>
                .label(UpdateScalableOrthographicFrusta)
                .after(TransformSystem::TransformPropagate),
        );

        app.add_system_to_stage(CoreStage::PostUpdate, camera_system::<ScalableOrthographicProjection>);
    }
}
