use bevy::DefaultPlugins;
use bevy::asset::AssetMetaCheck;
use bevy::prelude::*;
use bevy_egui::prelude::*;

use blockman::*;

fn main() {
    App::new()
        .insert_resource(ClearColor(Color::linear_rgb(1., 1., 1.)))
        .add_plugins(
            DefaultPlugins
                .set(WindowPlugin {
                    primary_window: Some(Window {
                        title: "Bevy game".to_string(), // ToDo
                        // Bind to canvas included in `index.html`
                        canvas: Some("#bevy".to_owned()),
                        fit_canvas_to_parent: true,
                        // Tells wasm not to override default event handling, like F5 and Ctrl+R
                        prevent_default_event_handling: false,
                        ..default()
                    }),
                    ..default()
                })
                .set(AssetPlugin {
                    meta_check: AssetMetaCheck::Never,
                    ..default()
                }),
        )
        .add_plugins(EguiPlugin::default())
        .insert_resource(WindowDims { height: 0.0, width: 0.0 })
        .insert_resource(Corners([(0.0, 0.0); 4]))
        .insert_resource(Blocks(vec![]))
        .init_resource::<GameConfig>()
        .init_resource::<LiveParams>()
        .init_resource::<MenuState>()
        .add_systems(Startup, init)
        .add_systems(
            Update,
            (
                restart_game.run_if(restart_requested),
                (check_wall_collision, random_walk, pickup_blocks)
                    .chain()
                    .run_if(game_running),
                update_view,
            )
                .chain(),
        )
        .add_systems(EguiPrimaryContextPass, ui_system)
        .run();
}
