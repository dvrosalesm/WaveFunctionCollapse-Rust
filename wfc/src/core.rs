use std::f32::consts::PI;

use bevy::{
    prelude::{
        default, App, AssetServer, Camera2dBundle, Commands, Entity, Plugin, Quat, Query, Res,
        ResMut, SystemSet, Transform, Vec3,
    },
    sprite::SpriteBundle,
    time::FixedTimestep,
};
use rand::Rng;

use crate::tiles::{Tile, TileRotation, WFCResource};

pub struct WFCPlugin;

enum SlotSides {
    Left = 0,
    Up = 1,
    Right = 2,
    Down = 3,
}

impl Plugin for WFCPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(WFCResource::new(20))
            .add_startup_system(weight_tiles)
            .add_system_set(
                SystemSet::new()
                    .with_run_criteria(FixedTimestep::step(0.01))
                    .with_system(collapse),
            )
            .add_system(render_images);
    }
}

fn weight_tiles(mut tiles_resource: ResMut<WFCResource>) {
    let num_images = 3;
    let num_variations = num_images * 4;

    // Load weights and create initial probabilities
    let file_contents = include_str!("../../interface/assets/rules.wfc");
    let rules: Vec<Vec<&str>> = file_contents
        .split("\n\n")
        .map(|rules| {
            rules
                .split('\n')
                .map(|rule| rule.trim())
                .collect::<Vec<&str>>()
        })
        .collect();

    // Data augmentation (Just rotate the rules)
    let tiles_with_rules: Vec<Tile> = rules
        .iter()
        .enumerate()
        .flat_map(|(ridx, sides)| {
            let mut new_tiles: Vec<Tile> = vec![];
            for idx in 0..4 {
                new_tiles.push(Tile {
                    rules: vec![
                        sides[(4 - idx) % 4].to_string(),
                        sides[(5 - idx) % 4].to_string(),
                        sides[(6 - idx) % 4].to_string(),
                        sides[(7 - idx) % 4].to_string(),
                    ],
                    rotation: match idx {
                        0 => TileRotation::Zero,
                        1 => TileRotation::Quarter,
                        2 => TileRotation::Half,
                        3 => TileRotation::ThreeQuarters,
                        _ => TileRotation::Zero,
                    },
                    file: ridx.to_string(),
                });
            }
            new_tiles
        })
        .collect::<Vec<Tile>>();

    // tiles_with_rules.push(Tile {
    //     rules: vec![
    //         "z".to_string(),
    //         "z".to_string(),
    //         "z".to_string(),
    //         "z".to_string(),
    //     ],
    //     rotation: TileRotation::Zero,
    //     file: "3".to_string(),
    // });

    tiles_resource.tiles = tiles_with_rules;
    tiles_resource.slots = vec![
        (0..(num_variations)).collect();
        tiles_resource.grid_width * tiles_resource.grid_width
    ];

    // For initial purposes collapse a random one :D
    let random_slot =
        rand::thread_rng().gen_range(0..tiles_resource.grid_width * tiles_resource.grid_width);
    let random_tile = rand::thread_rng().gen_range(0..(num_variations + 1));

    tiles_resource.slots[random_slot] = vec![random_tile];

    // Assume that the window is 500x500 px and each image will be 100x100 px
    // so the total amount of images will be 25
}

fn collapse(mut tiles_resource: ResMut<WFCResource>) {
    if !tiles_resource.slots.iter().any(|x| x.len() > 1) {
        return;
    }

    for idx in 0..tiles_resource.slots.len() {
        if tiles_resource.slots[idx].len() == 1 {
            continue;
        }

        // LEFT
        if idx > 0
            && (idx % tiles_resource.grid_width) != 0
            && tiles_resource.slots[idx - 1].len() == 1
        {
            let collapsed_slot = &tiles_resource.slots[idx - 1];
            let collapsed_tile = &tiles_resource.tiles[collapsed_slot[0]];
            let possible_tiles = tiles_resource.slots[idx]
                .clone()
                .into_iter()
                .filter(|&x| {
                    collapsed_tile.rules[SlotSides::Right as usize]
                        == tiles_resource.tiles[x].rules[SlotSides::Left as usize]
                })
                .collect::<Vec<usize>>();

            tiles_resource.slots[idx] = possible_tiles;
        }

        // TOP
        if idx >= tiles_resource.grid_width
            && tiles_resource.slots[idx - tiles_resource.grid_width].len() == 1
        {
            let collapsed_slot = &tiles_resource.slots[idx - tiles_resource.grid_width];
            let collapsed_tile = &tiles_resource.tiles[collapsed_slot[0]];
            let possible_tiles = tiles_resource.slots[idx]
                .clone()
                .into_iter()
                .filter(|&x| {
                    collapsed_tile.rules[SlotSides::Down as usize]
                        == tiles_resource.tiles[x].rules[SlotSides::Up as usize]
                })
                .collect::<Vec<usize>>();

            tiles_resource.slots[idx] = possible_tiles;
        }

        // RIGHT
        if idx < tiles_resource.grid_width * tiles_resource.grid_width
            && (idx + 1) % tiles_resource.grid_width != 0
            && tiles_resource.slots[idx + 1].len() == 1
        {
            let collapsed_slot = &tiles_resource.slots[idx + 1];
            let collapsed_tile = &tiles_resource.tiles[collapsed_slot[0]];
            let possible_tiles = tiles_resource.slots[idx]
                .clone()
                .into_iter()
                .filter(|&x| {
                    tiles_resource.tiles[x].rules[SlotSides::Right as usize]
                        == collapsed_tile.rules[SlotSides::Left as usize]
                })
                .collect::<Vec<usize>>();

            tiles_resource.slots[idx] = possible_tiles;
        }

        // BOTTOM
        if idx
            < ((tiles_resource.grid_width * tiles_resource.grid_width) - tiles_resource.grid_width)
            && tiles_resource.slots[idx + tiles_resource.grid_width].len() == 1
        {
            let collapsed_slot = &tiles_resource.slots[idx + tiles_resource.grid_width];
            let collapsed_tile = &tiles_resource.tiles[collapsed_slot[0]];
            let possible_tiles = tiles_resource.slots[idx]
                .clone()
                .into_iter()
                .filter(|&x| {
                    collapsed_tile.rules[SlotSides::Up as usize]
                        == tiles_resource.tiles[x].rules[SlotSides::Down as usize]
                })
                .collect::<Vec<usize>>();

            tiles_resource.slots[idx] = possible_tiles;
        }
    }

    if !tiles_resource.slots.is_empty() {
        // Find if there is an item that can be collapse at random since all possible tiles apply
        let collapse_candidate = tiles_resource.slots.clone().into_iter().enumerate().fold(
            (0, tiles_resource.slots[0].clone()),
            |(idx_min, val_min), (idx, val)| {
                if (val.len() > 1 && val.len() < val_min.len()) || val_min.len() < 2 {
                    (idx, val)
                } else {
                    (idx_min, val_min)
                }
            },
        );

        if collapse_candidate.1.len() > 1 {
            let random_tile = rand::thread_rng().gen_range(0..collapse_candidate.1.len());
            tiles_resource.slots[collapse_candidate.0] = vec![collapse_candidate.1[random_tile]];
        }
    }
}

fn render_images(
    tiles_resource: Res<WFCResource>,
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    query: Query<Entity>,
) {
    for entity in query.iter() {
        commands.entity(entity).despawn();
    }

    commands.spawn(Camera2dBundle::default());
    let slots = &tiles_resource.slots;

    for position in 0..slots.len() {
        if slots[position].is_empty() {
            continue;
        }
        let tile = &tiles_resource.tiles[slots[position][0]];

        let pos_x = ((position as u32 * 50) % (tiles_resource.grid_width as u32 * 50)) as f32;
        let pos_y = (position as f32 / tiles_resource.grid_width as f32).floor() * -50.0;

        let sprite1 = SpriteBundle {
            transform: Transform::from_xyz(
                pos_x - ((tiles_resource.grid_width as f32 * 50.0) / 2.0 - 25.0),
                pos_y + ((tiles_resource.grid_width as f32 * 50.0) / 2.0 - 25.0),
                0f32,
            )
            .with_scale(Vec3 {
                x: 1.0,
                y: 1.0,
                z: 0.0,
            })
            .with_rotation(Quat::from_rotation_z(match tile.rotation {
                TileRotation::Zero => 0.0,
                TileRotation::Quarter => -PI * 0.5,
                TileRotation::Half => -PI,
                TileRotation::ThreeQuarters => -PI * 1.5,
            })),
            texture: asset_server.load(format!("tiles/{}.png", tile.file)),
            ..default()
        };

        commands.spawn(sprite1);
    }
}
