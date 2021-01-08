use std::cmp;

use bevy::{diagnostic::{ FrameTimeDiagnosticsPlugin, PrintDiagnosticsPlugin }, prelude::*};
use rs_tiled_json::{load_map, Map};

// Can we use AppStates for transition between parts of the game?
// I.e. between maps, between map and fights etc
#[derive(Clone)]
enum AppState {
    StartMenu,
    WorldMap,
    InsideMap(usize),
    Inventory,
    Fight,
}

struct GameState {
    x: i32,
    y: i32,
    world_map: Map,
    maps: Vec<Map>,
}

impl GameState {
    fn new(world_map: Map) -> GameState {
        GameState {
            x: 0,
            y: 0,
            world_map,
            maps: Vec::new(),
        }
    }
}

struct TileMap {
    width: u32,
    height: u32,
}
struct Tile(i32, i32, usize); // x, y, depth

const TILE_SIZE: f32 = 16.0;
const TILES_X: usize = 20;
const TILES_Y: usize = 12;

fn setup(
    commands: &mut Commands,
    asset_server: Res<AssetServer>,
    state: Res<GameState>,
    mut texture_atlases: ResMut<Assets<TextureAtlas>>,
) {
    // Generic stuff
    commands.spawn(Camera2dBundle {
        transform: Transform {
            translation: Vec3::new(150.0, -100.0, 10.0),
            scale: Vec3::splat(0.2),
            ..Default::default()
        },
        ..Default::default()
    });

    // Tiles
    let tiles_handle = asset_server.load("alltiles.png");
    let tiles_texture_atlas = TextureAtlas::from_grid(tiles_handle, Vec2::new(16.0, 16.0), 3, 59);
    let tiles_atlas_handle = texture_atlases.add(tiles_texture_atlas);

    // The Idea: we have a fixed number of sprites nr_of_layers * width * height
    // Just 1 row/column larger than viewport, too large viewport zooms instead
    // We don't move the camera but instead change what image is used on the sprite
    // and move the sprites by the remainder pixels insted

    let map = &state.world_map;
    let width = map.width() as usize;

    let tile_layers = map
        .layers()
        .iter()
        .filter(|layer| layer.is_tile_layer())
        .map(|layer| layer.get_data());

    let mut depth = 0;
    for layer in tile_layers {
        if let Some(data) = layer {
            //println!("layer len {:?}", data.len());
            for x in 0..TILES_X {
                for y in 0..TILES_Y {
                    let map_index = y * width + x;
                    let tile_value = data[map_index as usize];

                    let handle = tiles_atlas_handle.clone();
                    let translation = Vec3::new(
                        x as f32 * TILE_SIZE,
                        y as f32 * TILE_SIZE * -1.0, // swap y direction
                        depth as f32,
                    );

                    commands
                        .spawn(SpriteSheetBundle {
                            texture_atlas: handle,
                            transform: Transform {
                                translation,
                                ..Default::default()
                            },
                            sprite: TextureAtlasSprite::new(if tile_value > 0 {
                                tile_value - 1
                            } else {
                                0
                            }),
                            visible: Visible {
                                is_visible: if tile_value == 0 { false } else { true },
                                is_transparent: if depth == 0 { false } else { true },
                            },
                            ..Default::default()
                        })
                        .with(Tile(x as i32, y as i32, depth));
                }
            }
            depth += 1;
        }
    }
}

fn animate_sprite_system(
    time: Res<Time>,
    texture_atlases: Res<Assets<TextureAtlas>>,
    mut query: Query<(&mut Timer, &mut TextureAtlasSprite, &Handle<TextureAtlas>)>,
) {
    for (mut timer, mut sprite, texture_atlas_handle) in query.iter_mut() {
        timer.tick(time.delta_seconds());
        if timer.finished() {
            let texture_atlas = texture_atlases.get(texture_atlas_handle).unwrap();
            sprite.index = ((sprite.index as usize + 1)
                % (texture_atlas.textures.len() - 1/* one empty frame at the end of tage-attack.png, skip it*/))
                as u32;
        }
    }
}

fn move_map(
    mut state: ResMut<GameState>,
    keyboard_input: Res<Input<KeyCode>>,
    mut query: Query<(&mut TextureAtlasSprite, &mut Visible, &Tile)>,
) {
    let mut delta_x: i32 = 0;
    let mut delta_y: i32 = 0;

    if keyboard_input.pressed(KeyCode::A) {
        delta_x = -1;
    } else if keyboard_input.pressed(KeyCode::W) {
        delta_y = -1;
    } else if keyboard_input.pressed(KeyCode::S) {
        delta_y = 1;
    } else if keyboard_input.pressed(KeyCode::D) {
        delta_x = 1;
    }

    if delta_x != 0 || delta_y != 0 {
        let width = state.world_map.width() as i32;
        let height = state.world_map.height() as i32;

        let x = cmp::min(cmp::max(state.x + delta_x, 0), width - 1);
        let y = cmp::min(cmp::max(state.y + delta_y, 0), height - 1);

        state.x = x;
        state.y = y;
        // println!("pos: x: {:?} y: {:?}", state.x, state.y);

        let tile_layers: Vec<&Vec<u32>> = state
            .world_map
            .layers()
            .iter()
            .filter(|layer| layer.is_tile_layer())
            .map(|layer| layer.get_data().expect("Could not get layer data"))
            .collect();

        for (mut sprite, mut visible, tile) in query.iter_mut() {
            let tile_x = x + tile.0;
            let tile_y = y + tile.1;

            if tile_x < width && tile_y < height {
                let map_index = tile_y * width + tile_x;

                match tile_layers[tile.2][map_index as usize] as u32 {
                    0 => visible.is_visible = false, // Zero means no tile
                    i => {
                        visible.is_visible = true;
                        sprite.index = i - 1;
                    }
                }
            } else {
                visible.is_visible = false;
            }
        }
    }
}

fn main() {
    // TODO: Move into an asset loader and into the app
    let map = load_map("assets/testbana.json").unwrap();

    App::build()
        .add_plugins(DefaultPlugins)
        .add_plugin(FrameTimeDiagnosticsPlugin::default())
        .add_plugin(PrintDiagnosticsPlugin::default())
        .add_system(bevy::input::system::exit_on_esc_system.system())
        .add_resource(GameState::new(map))
        .add_startup_system(setup.system())
        .add_system(animate_sprite_system.system())
        .add_system(move_map.system())
        .run();
}
