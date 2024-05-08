use bevy::app::Plugin;
use bevy::core_pipeline::core_2d::graph::{Core2d, Node2d};
use bevy::prelude::*;
use bevy::render::render_graph::{RenderGraphApp, ViewNodeRunner};
use bevy::render::render_resource::{ShaderType, StorageBuffer};
use bevy::render::{Extract, Render, RenderApp, RenderSet};
use render::lighting::{LightingNode, LightingPass, LightingPipeline};

mod component;
mod render;

pub use component::{PointLight2d, PointLight2dBundle};

pub struct Light2dPlugin;

impl Plugin for Light2dPlugin {
    fn build(&self, app: &mut App) {
        let Ok(render_app) = app.get_sub_app_mut(RenderApp) else {
            return;
        };

        render_app.add_systems(ExtractSchedule, (extract_camera, extract_point_lights));

        render_app.add_systems(Render, (prepare_lights).in_set(RenderSet::Prepare));

        render_app.add_render_graph_node::<ViewNodeRunner<LightingNode>>(Core2d, LightingPass);

        render_app.add_render_graph_edge(Core2d, Node2d::MainPass, LightingPass);
    }

    fn finish(&self, app: &mut App) {
        let Ok(render_app) = app.get_sub_app_mut(RenderApp) else {
            return;
        };

        render_app
            .init_resource::<LightingPipeline>()
            .init_resource::<LightingPassAssets>();
    }
}

fn extract_point_lights(
    mut commands: Commands,
    point_light_query: Extract<Query<(Entity, &PointLight2d, &GlobalTransform)>>,
) {
    for (entity, point_light, global_transform) in &point_light_query {
        commands
            .get_or_spawn(entity)
            .insert(*point_light)
            .insert(*global_transform);
    }
}

fn extract_camera(
    mut commands: Commands,
    camera_query: Extract<Query<(Entity, &Camera, &GlobalTransform)>>,
) {
    for (entity, camera, global_transform) in &camera_query {
        commands
            .get_or_spawn(entity)
            .insert(camera.clone())
            .insert(*global_transform);
    }
}

fn prepare_lights(
    camera_query: Query<(&Camera, &GlobalTransform)>,
    point_light_query: Query<(&PointLight2d, &GlobalTransform)>,
    mut lighting_pass_assets: ResMut<LightingPassAssets>,
) {
    let point_light_buffer = lighting_pass_assets.point_lights.get_mut();

    // Resources are global state, so we need to clear the data from the previous frame.
    point_light_buffer.data.clear();

    // TODO: Smarter camera query.
    // TODO: Better error when camera count is not equal to one.
    let (camera, camera_global_transform) = camera_query.single();

    for (point_light, point_light_global_transform) in &point_light_query {
        // TODO: Something smarter than unwrap.
        let point_light_position = camera
            .world_to_viewport(
                camera_global_transform,
                point_light_global_transform.translation(),
            )
            .unwrap();

        point_light_buffer.data.push(GpuPointLight2d {
            center: point_light_position,
            radius: point_light.radius,
            color: point_light.color.rgb_to_vec3(),
            energy: point_light.energy,
        });
    }
}

#[derive(Default, Resource)]
struct LightingPassAssets {
    pub point_lights: StorageBuffer<GpuPointLight2dBuffer>,
}

#[derive(Default, Clone, ShaderType)]
struct GpuPointLight2dBuffer {
    #[size(runtime)]
    pub data: Vec<GpuPointLight2d>,
}

#[derive(Default, Clone, ShaderType)]
struct GpuPointLight2d {
    pub center: Vec2,
    pub radius: f32,
    pub color: Vec3,
    pub energy: f32,
}
