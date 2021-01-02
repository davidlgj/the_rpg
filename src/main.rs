use bevy::prelude::*;


// Can we use AppStates for transition between parts of the game?
// I.e. between maps, between map and fights etc
// Is it possible to access the AppState in the system? Then we could even send along some state :thinking_face:
#[derive(Clone)]
enum AppState {
    StartMenu,
    WorldMap,
    InsideMap(usize),
    Inventory,
    Fight,
}
type Map = Vec<i32>; // TODO: maybe u8 to save space?

struct GameState {
    world_map: Map,
    maps: Vec<Map>,
}

struct Tile;

const TILE_SIZE: f32 = 16.0;

fn _setup_old(
    commands: &mut Commands,
    asset_server: Res<AssetServer>,
    mut materials: ResMut<Assets<ColorMaterial>>,
) {
    let texture_handle = asset_server.load("tage-attack.png");
    commands
        .spawn(Camera2dBundle::default())
        .spawn(SpriteBundle {
            material: materials.add(texture_handle.into()),
            ..Default::default()
        });
}

fn setup(
    commands: &mut Commands,
    asset_server: Res<AssetServer>,
    mut texture_atlases: ResMut<Assets<TextureAtlas>>,
) {
    // Generic stuff
    commands
        .spawn(Camera2dBundle::default());

    // Sprites
     // Hero
     let texture_handle = asset_server.load("tage-attack.png");
     let texture_atlas = TextureAtlas::from_grid(texture_handle, Vec2::new(TILE_SIZE, TILE_SIZE), 2, 4);
     let texture_atlas_handle = texture_atlases.add(texture_atlas);

     commands.spawn(SpriteSheetBundle {
             texture_atlas: texture_atlas_handle,
             transform: Transform {
                translation: Vec3::new(0.0, 0.0, 1.0), // z 1 so we are on top of the tiles
                scale: Vec3::splat(6.0),
                ..Default::default()
            },
             ..Default::default()
         })
         .with(Timer::from_seconds(0.01, true));

    // Tiles
    let tiles_handle = asset_server.load("alltiles.png");
    let tiles_texture_atlas = TextureAtlas::from_grid(tiles_handle, Vec2::new(16.0, 16.0), 3, 59);
    let tiles_atlas_handle = texture_atlases.add(tiles_texture_atlas);

    // Create 9x9 tiles
    for x in -5..6 {
        for y in -5..6 {
            let handle = tiles_atlas_handle.clone();

            commands.spawn(SpriteSheetBundle {
                texture_atlas: handle,
                transform: Transform {
                    translation: Vec3::new(x as f32 * TILE_SIZE*6.0, y as f32 * TILE_SIZE * 6.0, 0.0),
                    scale: Vec3::splat(6.0),
                    ..Default::default()
                },
                sprite: TextureAtlasSprite::new(40),

                ..Default::default()
            })
            .with(Tile)
            .with(Timer::from_seconds(0.3, true));
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
            sprite.index = ((sprite.index as usize + 1) % (texture_atlas.textures.len() - 1 /* one empty frame at the end of tage-attack.png, skip it*/)) as u32;
        }
    }
}


fn main() {
    App::build()
        .add_plugins(DefaultPlugins)
        .add_system(bevy::input::system::exit_on_esc_system.system())
        .add_startup_system(setup.system())
        .add_system(animate_sprite_system.system())
        .run();
}
