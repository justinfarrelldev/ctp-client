//! Pixel contracts for the generated-map/tileset/SDL rendering pipeline.
//!
//! No screenshots or color goldens: check geometric ground coverage and compare
//! fogged pixels against the same unfogged frame. Installation art is only read;
//! every framebuffer and deliberate corruption stays in memory. Missing art is
//! an explicit test failure, never a silently skipped rendering regression.

extern crate alloc;

use alloc::string::String;
use alloc::vec::Vec;
use std::path::Path;

use sdl2::pixels::{Color, PixelFormatEnum};
use sdl2::render::{Canvas, TextureCreator};
use sdl2::video::{Window, WindowContext};

use super::draw_map;
use crate::gameplay::model::{FogState, GameState, MapPoint, TerrainKind};
use crate::gameplay::runtime::textures::GameplayTextures;
use crate::gameplay::runtime::{
    Camera, DEFAULT_WINDOW_HEIGHT, DEFAULT_WINDOW_WIDTH, HeadlessSdlEnvironment, MovementInputState,
};

/// Impossible RGB565 terrain color, exposing even one uncovered pixel.
const BACKGROUND: [u8; 4] = [255, 0, 255, 255];
/// Ground families whose edges must meet, unlike overhanging mountain art.
const FLAT_TERRAINS: [TerrainKind; 3] = [
    TerrainKind::Grassland,
    TerrainKind::Plains,
    TerrainKind::Water,
];

/// One analytically selected pixel in the ground diamond, not its headroom.
#[derive(Clone, Copy, Debug)]
struct Probe {
    /// Generated map cell owning the sample.
    point: MapPoint,
    /// Index into the three flat terrain families.
    terrain: usize,
    /// NW, NE, SW or SE quadrant.
    edge: usize,
    /// Whether this pixel is within eight native pixels of a ground edge.
    on_edge: bool,
    /// RGBA byte offset in the SDL framebuffer.
    offset: usize,
}

#[test]
fn generated_flat_terrain_has_no_pixel_gaps_at_any_zoom() -> Result<(), String> {
    with_canvas(|canvas| {
        let creator = canvas.texture_creator();
        let mut textures = load_textures(&creator)?;
        for seed in [42_u64, 88_u64] {
            let (mut state, centered) = terrain_only_map(seed);
            state.filters.fog_of_war = false;
            for zoom in 1_i32..=3_i32 {
                let mut checked = Vec::new();
                for camera in cameras_at_zoom(centered, zoom) {
                    let pixels = render_pixels(canvas, &mut textures, &state, &camera)?;
                    let probes = flat_ground_probes(&state, &camera, 0_i32);
                    check_pixels(&probes, &pixels)
                        .map_err(|error| format!("seed {seed}, zoom {zoom}: {error}"))?;
                    checked.extend(probes);
                }
                check_coverage(&checked)?;
            }
        }
        Ok(())
    })
}

#[test]
fn generated_flat_edges_receive_only_their_own_fog_shading() -> Result<(), String> {
    with_canvas(|canvas| {
        let creator = canvas.texture_creator();
        let mut textures = load_textures(&creator)?;
        let (mut state, centered) = terrain_only_map(42_u64);
        // Keep generated terrain unchanged. Alternating visibility puts both
        // kinds of fog next to visible tiles, including repeated texture reuse.
        for y in 0_u16..state.height {
            for x in 0_u16..state.width {
                let index = usize::from(y) * usize::from(state.width) + usize::from(x);
                state.tiles[index].fog = match (x + y) % 3_u16 {
                    0_u16 => FogState::Visible,
                    1_u16 => FogState::Explored,
                    _ => FogState::Hidden,
                };
            }
        }
        for zoom in 1_i32..=3_i32 {
            let mut checked = Vec::new();
            for camera in cameras_at_zoom(centered, zoom) {
                state.filters.fog_of_war = false;
                let reference = render_pixels(canvas, &mut textures, &state, &camera)?;
                state.filters.fog_of_war = true;
                let pixels = render_pixels(canvas, &mut textures, &state, &camera)?;
                // Exclude the shared raster boundary, which can round to either
                // neighbor; still inspect the band immediately inside each edge.
                let probes = flat_ground_probes(&state, &camera, zoom);
                check_fog_pixels(&state, &probes, &pixels, &reference)
                    .map_err(|error| format!("fog enabled, zoom {zoom}: {error}"))?;
                checked.extend(probes);
            }
            check_coverage(&checked)?;
        }
        Ok(())
    })
}

#[test]
fn launch_map_flat_edges_keep_their_generated_visibility() -> Result<(), String> {
    with_canvas(|canvas| {
        let creator = canvas.texture_creator();
        let mut textures = load_textures(&creator)?;
        let mut checked = Vec::new();
        for seed in [42_u64, 88_u64] {
            let (mut state, camera) = terrain_only_map(seed);
            state.filters.fog_of_war = false;
            let reference = render_pixels(canvas, &mut textures, &state, &camera)?;
            state.filters.fog_of_war = true;
            let pixels = render_pixels(canvas, &mut textures, &state, &camera)?;
            let probes = flat_ground_probes(&state, &camera, 1_i32);
            check_fog_pixels(&state, &probes, &pixels, &reference)?;
            checked.extend(probes);
        }
        check_coverage(&checked)
    })
}

#[test]
fn fallback_flat_ground_also_fills_generated_map_tiles() -> Result<(), String> {
    with_canvas(|canvas| {
        let mut textures = GameplayTextures::without_assets_for_test();
        let (mut state, centered) = terrain_only_map(42_u64);
        for zoom in 1_i32..=3_i32 {
            let mut checked = Vec::new();
            for camera in cameras_at_zoom(centered, zoom) {
                state.filters.fog_of_war = false;
                let pixels = render_pixels(canvas, &mut textures, &state, &camera)?;
                let probes = flat_ground_probes(&state, &camera, 0_i32);
                check_pixels(&probes, &pixels)
                    .map_err(|error| format!("fallback, zoom {zoom}: {error}"))?;
                checked.extend(probes);
                state.filters.fog_of_war = true;
                let fogged = render_pixels(canvas, &mut textures, &state, &camera)?;
                let inset_probes = flat_ground_probes(&state, &camera, zoom);
                check_fog_pixels(&state, &inset_probes, &fogged, &pixels)?;
            }
            check_coverage(&checked)?;
        }
        Ok(())
    })
}

#[test]
fn pixel_oracle_detects_a_missing_pixel_on_each_flat_terrain_edge() -> Result<(), String> {
    with_canvas(|canvas| {
        let creator = canvas.texture_creator();
        let mut textures = load_textures(&creator)?;
        let (mut state, camera) = terrain_only_map(42_u64);
        state.filters.fog_of_war = false;
        let mut pixels = render_pixels(canvas, &mut textures, &state, &camera)?;
        let probes = flat_ground_probes(&state, &camera, 0_i32);
        check_coverage(&probes)?;
        check_pixels(&probes, &pixels)?;
        for terrain in 0_usize..FLAT_TERRAINS.len() {
            for edge in 0_usize..4_usize {
                let probe = probes
                    .iter()
                    .find(|probe| probe.on_edge && probe.terrain == terrain && probe.edge == edge)
                    .ok_or_else(|| {
                        format!("missing oracle control for terrain {terrain}, edge {edge}")
                    })?;
                let original = pixels[probe.offset..probe.offset + 4_usize].to_vec();
                pixels[probe.offset..probe.offset + 4_usize].copy_from_slice(&BACKGROUND);
                let detected = check_pixels(core::slice::from_ref(probe), &pixels).is_err();
                pixels[probe.offset..probe.offset + 4_usize].copy_from_slice(&original);
                if !detected {
                    return Err(format!("oracle accepted a missing pixel: {probe:?}"));
                }
            }
        }
        Ok(())
    })
}

/// Uses the same decoder, terrain selection, RGBA upload and blend mode as gameplay.
fn load_textures(creator: &TextureCreator<WindowContext>) -> Result<GameplayTextures<'_>, String> {
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("ctp2");
    GameplayTextures::load(&root, creator, true)
        .map_err(|error| format!("pixel regression requires the CTP2 installation: {error}"))
}

/// Generates application terrain; suppresses actors/ruins that could conceal gaps.
fn terrain_only_map(seed: u64) -> (GameState, Camera) {
    let mut state = GameState::new(48, 36, 5, seed);
    let camera = Camera::centered_on_selected(&state);
    state.units.clear();
    state.cities.clear();
    state.selected_unit = None;
    for tile in &mut state.tiles {
        tile.feature = None;
    }
    (state, camera)
}

/// Exercises a launch-centered view plus clipped map borders at each supported zoom.
fn cameras_at_zoom(centered: Camera, zoom: i32) -> [Camera; 2] {
    let mut cameras = [centered, Camera::for_test_identity()];
    for camera in &mut cameras {
        for _ in 1_i32..zoom {
            camera.zoom_in();
        }
    }
    cameras
}

/// Reads the production SDL map renderer back into memory without presenting UI.
fn render_pixels(
    canvas: &mut Canvas<Window>,
    textures: &mut GameplayTextures<'_>,
    state: &GameState,
    camera: &Camera,
) -> Result<Vec<u8>, String> {
    canvas.set_draw_color(Color::RGBA(
        BACKGROUND[0],
        BACKGROUND[1],
        BACKGROUND[2],
        BACKGROUND[3],
    ));
    canvas.clear();
    draw_map(
        canvas,
        textures,
        state,
        camera,
        &MovementInputState::default(),
    )
    .map_err(|error| error.to_string())?;
    canvas.read_pixels(None, PixelFormatEnum::RGBA32)
}

/// Samples ideal diamonds using pixel centers and integer rational arithmetic.
///
/// Deliberately independent of decoder scanline bounds: tileset.h specifies a
/// 94x48 ground diamond with 24px headroom. Coverage samples include the entire
/// interior AND its four edge bands. Overhanging terrain silhouettes are excluded.
fn flat_ground_probes(state: &GameState, camera: &Camera, inset: i32) -> Vec<Probe> {
    let width = i32::try_from(camera.tile_width()).expect("tile width fits i32");
    let grid_height = i32::try_from(camera.tile_height()).expect("tile height fits i32");
    let zoom = grid_height.div_euclid(72_i32);
    let height = 48_i32 * zoom;
    let headroom = 24_i32 * zoom;
    let limit = width * height;
    let mut probes = Vec::new();
    for y in 1_u16..state.height - 1_u16 {
        for x in 1_u16..state.width - 1_u16 {
            let point = MapPoint::new(x, y);
            let tile = state.tile(point).expect("generated cell exists");
            let Some(terrain) = FLAT_TERRAINS.iter().position(|kind| *kind == tile.terrain) else {
                continue;
            };
            let (screen_x, screen_y) = camera.tile_to_screen(point);
            for row in 0_i32..height {
                let py = screen_y + headroom + row;
                if !(0_i32..i32::try_from(DEFAULT_WINDOW_HEIGHT).expect("viewport fits"))
                    .contains(&py)
                {
                    continue;
                }
                for col in 0_i32..width {
                    let px = screen_x + col;
                    if !(0_i32..i32::try_from(DEFAULT_WINDOW_WIDTH).expect("viewport fits"))
                        .contains(&px)
                    {
                        continue;
                    }
                    let dx = 2_i32 * col + 1_i32 - width;
                    let dy = 2_i32 * row + 1_i32 - height;
                    let distance = dx.abs() * height + dy.abs() * width;
                    let depth = limit - distance;
                    if depth < 2_i32 * width * inset {
                        continue;
                    }
                    let edge = usize::from(dy >= 0_i32) * 2_usize + usize::from(dx >= 0_i32);
                    let on_edge = depth <= 2_i32 * width * 8_i32 * zoom;
                    let offset = (usize::try_from(py).expect("pixel is nonnegative")
                        * usize::try_from(DEFAULT_WINDOW_WIDTH).expect("viewport fits")
                        + usize::try_from(px).expect("pixel is nonnegative"))
                        * 4_usize;
                    probes.push(Probe {
                        point,
                        terrain,
                        edge,
                        on_edge,
                        offset,
                    });
                }
            }
        }
    }
    probes
}

/// Ensures each run really inspected all four sides of each required terrain.
fn check_coverage(probes: &[Probe]) -> Result<(), String> {
    for (terrain, kind) in FLAT_TERRAINS.iter().enumerate() {
        for edge in 0_usize..4_usize {
            if !probes
                .iter()
                .any(|probe| probe.on_edge && probe.terrain == terrain && probe.edge == edge)
            {
                return Err(format!("no pixels checked for {kind:?}, edge {edge}"));
            }
        }
    }
    Ok(())
}

/// Reports the first uncovered pixel with actionable map and screen context.
fn check_pixels(probes: &[Probe], pixels: &[u8]) -> Result<(), String> {
    for probe in probes {
        let pixel = &pixels[probe.offset..probe.offset + 4_usize];
        if pixel == BACKGROUND {
            return Err(format!(
                "uncovered {:?} at {:?}, edge {}, screen {:?}",
                FLAT_TERRAINS[probe.terrain],
                probe.point,
                probe.edge,
                screen_pixel(*probe)
            ));
        }
    }
    Ok(())
}

/// Verifies attenuation without freezing any source art or palette colors.
fn check_fog_pixels(
    state: &GameState,
    probes: &[Probe],
    pixels: &[u8],
    reference: &[u8],
) -> Result<(), String> {
    for probe in probes {
        let fog = state.tile(probe.point).expect("probe cell exists").fog;
        // Preserve the former black-overlay alpha strengths: 0, 120 and 220.
        let brightness = match fog {
            FogState::Visible => 255_u16,
            FogState::Explored => 135_u16,
            FogState::Hidden => 35_u16,
        };
        for channel in 0_usize..3_usize {
            let original = u16::from(reference[probe.offset + channel]);
            let expected = original.saturating_mul(brightness).div_euclid(255_u16);
            let actual = u16::from(pixels[probe.offset + channel]);
            if actual.abs_diff(expected) > 1_u16 {
                return Err(format!(
                    "{fog:?} {:?} at {:?}, edge {}, screen {:?}: channel {channel} was {actual}, expected {expected}",
                    FLAT_TERRAINS[probe.terrain],
                    probe.point,
                    probe.edge,
                    screen_pixel(*probe)
                ));
            }
        }
    }
    Ok(())
}

/// Converts a byte offset into useful framebuffer coordinates for failures.
fn screen_pixel(probe: Probe) -> (usize, usize) {
    let pixel = probe.offset.div_euclid(4_usize);
    let width = usize::try_from(DEFAULT_WINDOW_WIDTH).expect("viewport width fits");
    (pixel % width, pixel.div_euclid(width))
}

/// Serializes SDL access and restores dummy drivers even when a check fails.
fn with_canvas(
    check: impl FnOnce(&mut Canvas<Window>) -> Result<(), String>,
) -> Result<(), String> {
    let guard = crate::sdl_test_lock::lock_sdl_for_test()?;
    let environment = HeadlessSdlEnvironment::apply();
    let result = (|| {
        let sdl = sdl2::init()?;
        let video = sdl.video()?;
        let window = video
            .window(
                "terrain pixel regression",
                DEFAULT_WINDOW_WIDTH,
                DEFAULT_WINDOW_HEIGHT,
            )
            .hidden()
            .build()
            .map_err(|error| error.to_string())?;
        let mut canvas = window
            .into_canvas()
            .software()
            .build()
            .map_err(|error| error.to_string())?;
        check(&mut canvas)
    })();
    environment.restore();
    drop(guard);
    result
}
