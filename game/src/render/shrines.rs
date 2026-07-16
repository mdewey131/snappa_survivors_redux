use avian2d::prelude::*;
use bevy::{
    prelude::*,
    render::render_resource::AsBindGroup,
    sprite_render::{AlphaMode2d, Material2d, Material2dPlugin},
};

use crate::{render::RenderYtoZ, shared::interactables::BeerShrine};

const BEER_SHADER: &str = "shrines/beer_shader.wgsl";
const BEER_COLOR: [f32; 3] = [252.0 / 255.0, 194.0 / 255.0, 93.0 / 255.0];

pub struct SharedShrinesRenderPlugin;

impl Plugin for SharedShrinesRenderPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(Material2dPlugin::<BeerShrineMaterial>::default());
        app.add_systems(Update, (rendering_on_shrine_add, animate_beer_shrine));
        app.add_systems(Update, animate_beer_shrine);
    }
}

#[derive(Component)]
pub struct BeerShrineFront;
#[derive(Component)]
pub struct BeerShrineBeer;
#[derive(Component)]
pub struct BeerShrineBack;

#[derive(Asset, TypePath, AsBindGroup, Debug, Clone)]
struct BeerShrineMaterial {
    #[uniform(0)]
    color: LinearRgba,
    #[texture(1)]
    #[sampler(2)]
    texture: Handle<Image>,
    #[uniform(3)]
    percent: f32,
}

impl Material2d for BeerShrineMaterial {
    fn fragment_shader() -> bevy::shader::ShaderRef {
        BEER_SHADER.into()
    }
    fn alpha_mode(&self) -> bevy::sprite_render::AlphaMode2d {
        AlphaMode2d::Blend
    }
}

fn rendering_on_shrine_add(
    mut commands: Commands,
    asset: Res<AssetServer>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<BeerShrineMaterial>>,
    q_shrine: Query<(Entity, &Position), (Added<BeerShrine>, Without<Sprite>)>,
) {
    for (shrine, _pos) in &q_shrine {
        let _layout = TextureAtlasLayout::from_grid(UVec2::splat(96), 8, 1, None, None);
        let front_image: Handle<Image> = asset.load("shrines/beer_shrine_front.png");
        let back_image: Handle<Image> = asset.load("shrines/beer_shrine_back.png");

        let image_size = Vec2::new(96.0, 96.0);
        commands.entity(shrine).with_children(|parent| {
            parent.spawn((
                BeerShrineFront,
                Sprite::from(front_image),
                RenderYtoZ::new(1.0),
            ));
            parent.spawn((
                BeerShrineBeer,
                Mesh2d(meshes.add(Rectangle::from_size(image_size))),
                MeshMaterial2d(materials.add(BeerShrineMaterial {
                    color: LinearRgba {
                        red: BEER_COLOR[0],
                        green: BEER_COLOR[1],
                        blue: BEER_COLOR[2],
                        alpha: 1.0,
                    },
                    texture: asset.load("shrines/beer_shrine_beer.png"),
                    percent: 1.0,
                })),
                RenderYtoZ::new(0.0),
            ));
            parent.spawn((
                BeerShrineBack,
                Sprite::from(back_image),
                RenderYtoZ::new(-1.0),
            ));
        });
    }
}

fn animate_beer_shrine(
    mut materials: ResMut<Assets<BeerShrineMaterial>>,
    q_shrine: Query<(&BeerShrine, &Children)>,
    q_beer_mesh: Query<&mut MeshMaterial2d<BeerShrineMaterial>>,
) {
    for (shrine, children) in &q_shrine {
        let pct_charge_remaining = shrine.current_charge / shrine.max_charge;

        for child in children {
            if let Ok(material) = q_beer_mesh.get(*child) {
                let mut asset = materials
                    .get_mut(material.id())
                    .expect("Material not found!");
                asset.percent = pct_charge_remaining;
            }
        }
    }
}
