use std::f32::consts::PI;

use bevy::prelude::*;
use bevy_egui::{EguiContexts, egui};

#[derive(Resource)]
pub struct WindowDims {
    pub height: f32,
    pub width: f32,
}

#[derive(Component)]
pub struct Blockman;

#[derive(Component)]
pub struct Board;

#[derive(Component)]
pub struct GameEntity;

#[derive(Component)]
pub struct Block {
    stack: usize,
    cooldown: bool,
}

#[derive(Component)]
pub struct Location {
    x: f32,
    y: f32,
}

#[derive(Component, Debug)]
pub struct Angle {
    a: f32,
    da: f32,
    need_change: bool,
}

#[derive(Component)]
pub struct HoldingBlock(bool);

#[derive(Component)]
pub struct Id(usize);

#[derive(Component)]
pub struct HoldingIcon;

#[derive(Component)]
pub struct BoardData {
    pub size: f32,
}

#[derive(Resource)]
pub struct Corners(pub [(f32, f32); 4]);

#[derive(Resource)]
pub struct Blocks(pub Vec<(f32, f32)>);

const RAD_CONST: f32 = 0.03;
const BOARD_SCALE: f32 = 0.8;
const ANGLE_STICK: f32 = 50.0;
const SISMAN_SCALE: f32 = 0.1;
const SPEED: f32 = 0.5;
const N_BLOCKS: usize = 10;

#[derive(Resource, Clone, PartialEq)]
pub struct GameConfig {
    pub n_blocks: usize,
    pub width_const: f32,
    pub sisman_scale: f32,
}

impl Default for GameConfig {
    fn default() -> Self {
        Self {
            n_blocks: N_BLOCKS,
            width_const: RAD_CONST,
            sisman_scale: SISMAN_SCALE,
        }
    }
}

#[derive(Resource)]
pub struct LiveParams {
    pub speed: f32,
    pub angle_stick: f32,
}

impl Default for LiveParams {
    fn default() -> Self {
        Self {
            speed: SPEED,
            angle_stick: ANGLE_STICK,
        }
    }
}

#[derive(Resource, Default)]
pub struct MenuState {
    pub open: bool,
    pub restart_requested: bool,
    draft: GameConfig,
}

pub fn game_running(menu: Res<MenuState>) -> bool {
    !menu.open && !menu.restart_requested
}

pub fn restart_requested(menu: Res<MenuState>) -> bool {
    menu.restart_requested
}

pub fn init(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
    mut window_dims: ResMut<WindowDims>,
    mut corners: ResMut<Corners>,
    mut block_locations: ResMut<Blocks>,
    config: Res<GameConfig>,
    window: Single<&Window>,
) {
    commands.spawn(Camera2d);

    spawn_world(
        &mut commands,
        &asset_server,
        &mut meshes,
        &mut materials,
        &window,
        &mut window_dims,
        &mut corners,
        &mut block_locations,
        &config,
    );
}

pub fn restart_game(
    mut commands: Commands,
    old_entities: Query<Entity, With<GameEntity>>,
    asset_server: Res<AssetServer>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
    mut window_dims: ResMut<WindowDims>,
    mut corners: ResMut<Corners>,
    mut block_locations: ResMut<Blocks>,
    config: Res<GameConfig>,
    mut menu: ResMut<MenuState>,
    window: Single<&Window>,
) {
    menu.restart_requested = false;

    for e in &old_entities {
        commands.entity(e).despawn();
    }

    spawn_world(
        &mut commands,
        &asset_server,
        &mut meshes,
        &mut materials,
        &window,
        &mut window_dims,
        &mut corners,
        &mut block_locations,
        &config,
    );
}

fn spawn_world(
    commands: &mut Commands,
    asset_server: &AssetServer,
    meshes: &mut Assets<Mesh>,
    materials: &mut Assets<ColorMaterial>,
    window: &Window,
    window_dims: &mut WindowDims,
    corners: &mut Corners,
    block_locations: &mut Blocks,
    config: &GameConfig,
) {
    let h = window.height();

    window_dims.height = h;
    window_dims.width = window.width();

    let board_size = BOARD_SCALE * h;

    let corners_n = [
        (0. - 0.4 * h, 0. + 0.4 * h),
        (0. + 0.4 * h, 0. + 0.4 * h),
        (0. - 0.4 * h, 0. - 0.4 * h),
        (0. + 0.4 * h, 0. - 0.4 * h),
    ];

    *corners = Corners(corners_n);

    let x_iter = rand::random_iter();
    let y_iter = rand::random_iter();

    *block_locations = Blocks(x_iter.zip(y_iter).take(config.n_blocks).collect());
    let block_tex = Color::linear_rgb(0.6, 0.6, 0.6);
    let block_mesh = Rectangle::new(config.width_const * 2. * h, config.width_const * 2. * h);

    let lbound = corners.0[0].0;
    let rbound = corners.0[0].1;
    let s = (corners.0[0].0 - corners.0[1].0).abs();

    let font = TextFont {
        //font: asset_server.load("Dynamix.ttf").into(),
        font_size: FontSize::Px(config.width_const * s),
        ..Default::default()
    };

    for (i, b) in block_locations.0.iter().enumerate() {
        let t = calc_transform(b.0, b.1, 1.5, lbound, rbound, s);
        commands.spawn((
            GameEntity,
            Block {
                stack: 1,
                cooldown: false,
            },
            Id(i),
            Location { x: b.0, y: b.1 },
            Transform::from_xyz(t.translation.x, t.translation.y, 1.5),
            MeshMaterial2d(materials.add(block_tex)),
            Mesh2d(meshes.add(block_mesh)),
        ));

        commands.spawn((
            GameEntity,
            Text2d::new("1"),
            Id(i),
            font.clone(),
            Location { x: b.0, y: b.1 },
            Transform::from_xyz(t.translation.x, t.translation.y, 1.6),
            TextColor(Color::linear_rgb(0., 0., 1.)),
        ));
    }

    let mut sisman = Sprite::from_image(asset_server.load("sisman.png"));
    sisman.custom_size = Some(Vec2::from([
        config.sisman_scale * h,
        config.sisman_scale * h,
    ]));
    sisman.flip_x = true;

    commands.spawn((
        GameEntity,
        Blockman,
        sisman,
        HoldingBlock(false),
        Transform::from_xyz(corners_n[0].0, corners_n[0].1 - 0.05 * h, 2.0),
        Location { x: 0.5, y: 0.5 },
        Angle {
            a: 90.,
            da: 0.,
            need_change: false,
        },
    ));

    commands.spawn((
        GameEntity,
        HoldingIcon,
        Transform::from_xyz(0., 0., 2.1),
        MeshMaterial2d(materials.add(block_tex)),
        Mesh2d(meshes.add(block_mesh)),
        Visibility::Hidden,
    ));

    commands.spawn((
        GameEntity,
        Board,
        Mesh2d(meshes.add(Rectangle::new(board_size * 1.1, board_size * 1.1))),
        MeshMaterial2d(materials.add(Color::linear_rgb(0.1, 0.5, 0.1))),
        Transform::from_xyz(0.0, 0.0, 1.0),
        BoardData {
            size: BOARD_SCALE * h,
        },
    ));
}

fn calc_transform(x: f32, y: f32, z: f32, lbound: f32, tbound: f32, s: f32) -> Transform {
    Transform::from_xyz(lbound + x * s, tbound - y * s, z)
}

pub fn ui_system(
    mut contexts: EguiContexts,
    mut menu: ResMut<MenuState>,
    mut config: ResMut<GameConfig>,
    mut params: ResMut<LiveParams>,
) -> Result {
    let ctx = contexts.ctx_mut()?;
    let menu = &mut *menu;
    let params = &mut *params;

    egui::Window::new("Controls")
        .anchor(egui::Align2::LEFT_TOP, [10.0, 10.0])
        .resizable(false)
        .collapsible(false)
        .show(ctx, |ui| {
            ui.add(egui::Slider::new(&mut params.speed, 0.0..=10.0).text("speed"));
            ui.add(
                egui::Slider::new(&mut params.angle_stick, 1.0..=500.0)
                    .logarithmic(true)
                    .text("angle_stick"),
            );
            ui.separator();
            if ui.button("Config").clicked() && !menu.open {
                menu.draft = (*config).clone();
                menu.open = true;
            }
        });

    if menu.open {
        let mut window_open = true;
        let mut save = false;
        let mut cancel = false;

        egui::Window::new("Config")
            .open(&mut window_open)
            .anchor(egui::Align2::CENTER_CENTER, [0.0, 0.0])
            .resizable(false)
            .collapsible(false)
            .show(ctx, |ui| {
                ui.label("Save to restart");
                ui.separator();

                ui.add(egui::Slider::new(&mut menu.draft.n_blocks, 1..=200).text("n_blocks"));
                ui.add(
                    egui::Slider::new(&mut menu.draft.width_const, 0.005..=0.1)
                        .step_by(0.001)
                        .text("rad_const"),
                );
                ui.add(
                    egui::Slider::new(&mut menu.draft.sisman_scale, 0.02..=0.4)
                        .step_by(0.005)
                        .text("sisman_scale"),
                );

                ui.separator();
                ui.horizontal(|ui| {
                    if ui.button("Save").clicked() {
                        save = true;
                    }
                    if ui.button("Cancel").clicked() {
                        cancel = true;
                    }
                    if ui.button("Defaults").clicked() {
                        menu.draft = GameConfig::default();
                    }
                });
            });

        if save {
            *config = menu.draft.clone();
            menu.restart_requested = true;
            menu.open = false;
        } else if cancel || !window_open {
            menu.open = false;
        }
    }

    Ok(())
}

pub fn check_wall_collision(query: Query<(&mut Angle, &Location), With<Blockman>>) {
    let r = 0.0..1.0;
    for (mut a, l) in query {
        if !r.contains(&l.x) || !r.contains(&l.y) {
            a.need_change = true;
        }
    }
}

pub fn update_view(
    guy_query: Query<
        (&HoldingBlock, &mut Transform, &Location),
        (With<Blockman>, Without<HoldingIcon>, Without<Block>),
    >,
    holding_icon_query: Query<
        (&mut Transform, &mut Visibility),
        (With<HoldingIcon>, Without<Blockman>, Without<Block>),
    >,
    block_query: Query<
        (&Location, &Block, &mut Transform, &Id),
        (Without<Blockman>, Without<HoldingIcon>),
    >,
    mut text_query: Query<
        (&mut Text2d, &mut Transform, &Id),
        (Without<Blockman>, Without<HoldingIcon>, Without<Block>),
    >,
    corners: Res<Corners>,
    config: Res<GameConfig>,
) {
    let mut v = false;
    let mut guy_t = Transform::from_xyz(0., 0., 0.);
    let s = (corners.0[0].0 - corners.0[1].0).abs();
    let lbound = corners.0[0].0;
    let rbound = corners.0[0].1;

    for (holding_block, mut t, l) in guy_query {
        v = holding_block.0;
        guy_t = calc_transform(l.x, l.y, 2.0, lbound, rbound, s);
        *t = guy_t;
    }

    for (mut t, mut vis) in holding_icon_query {
        *vis = if v {
            Visibility::Visible
        } else {
            Visibility::Hidden
        };
        let mut icon_t = guy_t;
        icon_t.translation.y += config.sisman_scale * s * 0.9;
        *t = icon_t;
    }

    for (l, block, mut t, id) in block_query {
        let mut block_t = calc_transform(l.x, l.y, 2.0, lbound, rbound, s);
        block_t.scale *= block.stack as f32;
        *t = block_t;

        text_query
            .iter_mut()
            .filter(|(_, _, t_id)| id.0 == t_id.0)
            .for_each(|(mut txt, mut t, _)| {
                txt.0 = format!("{}", block.stack).to_string();
                *t = block_t;
            });
    }
}

pub fn pickup_blocks(
    mut guy_query: Query<
        (&mut HoldingBlock, &mut Angle, &Location),
        (With<Blockman>, Without<Block>),
    >,
    block_query: Query<(&mut Location, &mut Block), Without<Blockman>>,
    config: Res<GameConfig>,
) {
    match guy_query.iter_mut().next() {
        Some((mut holding_block, mut angle, guy_l)) => {
            for (mut block_l, mut block_data) in block_query {
                let radius = block_data.stack as f32 * config.width_const;
                let hitbox_x = (block_l.x - radius - config.sisman_scale / 2.)
                    ..(block_l.x + radius + config.sisman_scale / 2.);
                let hitbox_y = (block_l.y - radius - config.sisman_scale / 2.)
                    ..(block_l.y + radius);

                if hitbox_x.contains(&guy_l.x) && hitbox_y.contains(&guy_l.y) {
                    if !block_data.cooldown {
                        if !holding_block.0 {
                            holding_block.0 = true;
                            block_data.stack -= 1;
                        } else {
                            holding_block.0 = false;
                            block_data.stack += 1;
                        }
                        angle.need_change = true;
                        block_data.cooldown = true;
                    }
                } else {
                    block_data.cooldown = false;
                }

                if block_data.stack < 1 {
                    block_l.x = -1.0;
                    block_l.y = -1.0;
                }
            }
        }
        None => {}
    }
}

pub fn random_walk(
    query: Query<(&mut Location, &mut Angle), With<Blockman>>,
    time: Res<Time>,
    params: Res<LiveParams>,
) {
    let speed = params.speed;
    let dt = time.delta().as_secs_f32() * speed;

    for (mut l, mut a) in query {
        if a.need_change {
            a.da = 0.;
            a.a += rand::random_range(100.0..135.0);
            a.need_change = false;
        } else {
            a.da += dt;
            let change = if a.da < 0.9 {
                rand::random_bool((a.da / params.angle_stick).clamp(0.0, 1.0) as f64)
            } else {
                true
            };

            if change {
                a.a += rand::random_range(15.0..160.0);
                a.da = 0.;
            }
        }

        l.x += a.a.to_radians().sin() * dt;
        l.y += a.a.to_radians().cos() * dt;

        l.x = l.x.clamp(-0.01, 1.01);
        l.y = l.y.clamp(-0.01, 1.01);
    }
}
