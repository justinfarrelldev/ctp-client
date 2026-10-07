//! Main-menu model and headless runtime tests.

use super::*;
use core::cell::RefCell;
use sdl2::event::Event;
use sdl2::mouse::MouseButton;
use tempfile::TempDir;

/// Recorded music autoplay request.
#[derive(Debug, Clone, PartialEq, Eq)]
struct MusicCall {
    /// Policy supplied by the session.
    policy: MenuMusicPolicy,
    /// Absolute path supplied by the session.
    absolute_path: PathBuf,
}

/// Test audio backend that records calls.
struct RecordingMusicBackend {
    /// Recorded play requests.
    calls: RefCell<Vec<MusicCall>>,
}

impl RecordingMusicBackend {
    /// Creates an empty recording backend.
    fn new() -> Self {
        Self {
            calls: RefCell::new(Vec::new()),
        }
    }
}

impl MainMenuMusicBackend for RecordingMusicBackend {
    fn play_music(
        &mut self,
        policy: &MenuMusicPolicy,
        absolute_path: PathBuf,
    ) -> Result<(), MainMenuError> {
        self.calls.borrow_mut().push(MusicCall {
            policy: policy.clone(),
            absolute_path,
        });
        Ok(())
    }
}

/// Creates a deterministic SDL mouse-down event for headless runtime tests.
const fn mouse_down(x: i32, y: i32) -> Event {
    Event::MouseButtonDown {
        timestamp: 0,
        window_id: 0,
        which: 0,
        mouse_btn: MouseButton::Left,
        clicks: 1,
        x,
        y,
    }
}

#[test]
fn landing_page_exposes_accessible_buttons() {
    let model = MainMenuModel::ctp2_landing(PathBuf::from("/ctp2"));

    assert_eq!(
        model.activate_accessible("Single Player"),
        Ok(MainMenuAction::SinglePlayer)
    );
    assert_eq!(
        model.activate_accessible("multiplayer"),
        Ok(MainMenuAction::Multiplayer),
        "accessible lookup should be case-insensitive"
    );
    assert_eq!(
        model.activate_accessible("Credits"),
        Ok(MainMenuAction::Credits)
    );
    assert_eq!(model.activate_accessible("Quit"), Ok(MainMenuAction::Quit));
}

#[test]
fn landing_page_button_layout_matches_original_ldl() {
    let model = MainMenuModel::ctp2_landing(PathBuf::from("/ctp2"));

    let multiplayer = model
        .button_by_id(MainMenuButtonId::Multiplayer)
        .expect("multiplayer button should exist");
    let single_player = model
        .button_by_id(MainMenuButtonId::SinglePlayer)
        .expect("single-player button should exist");
    let credits = model
        .button_by_id(MainMenuButtonId::Credits)
        .expect("credits button should exist");
    let quit = model
        .button_by_id(MainMenuButtonId::Quit)
        .expect("quit button should exist");

    assert_eq!(multiplayer.rect, MenuRect::new(36, 455, 290, 32));
    assert_eq!(single_player.rect, MenuRect::new(478, 455, 290, 32));
    assert_eq!(credits.rect, MenuRect::new(36, 558, 140, 32));
    assert_eq!(quit.rect, MenuRect::new(624, 558, 140, 32));
}

#[test]
fn landing_page_button_styles_match_original_assets() {
    let model = MainMenuModel::ctp2_landing(PathBuf::from("/ctp2"));

    let multiplayer = model
        .button_by_id(MainMenuButtonId::Multiplayer)
        .expect("multiplayer button should exist");
    let quit = model
        .button_by_id(MainMenuButtonId::Quit)
        .expect("quit button should exist");

    assert_eq!(multiplayer.style.id, ButtonStyleId::BlueLarge);
    assert_eq!(multiplayer.style.assets.up.left, "upbt07aU.tga");
    assert_eq!(multiplayer.style.assets.hover.middle, "upbt07bH.tga");
    assert_eq!(quit.style.id, ButtonStyleId::StoneLarge);
    assert_eq!(quit.style.assets.up.left, "upbt06aU.tga");
    assert_eq!(quit.style.assets.down.right, "upbt06cD.tga");
    assert!(
        model.original_style_mismatches().is_empty(),
        "landing page style should validate against original LDL values"
    );
}

#[test]
fn landing_page_uses_original_background_and_version_metadata() {
    let model = MainMenuModel::ctp2_landing(PathBuf::from("/ctp2"));

    assert_eq!(model.width, MAIN_MENU_WIDTH);
    assert_eq!(model.height, MAIN_MENU_HEIGHT);
    assert_eq!(
        model.background,
        MenuAssetReference {
            archive_relative_path: "ctp2_data/default/graphics/pictures/pic565.zfs",
            entry_name: "upsg001.rim",
        }
    );
    assert_eq!(model.version_text, "1.1");
    assert_eq!(model.version_rect, MenuRect::new(720, 515, 60, 24));
}

#[test]
fn menu_music_autoplays_expected_ctp2_track_once() {
    let temp_dir = TempDir::new().expect("temp directory should be created");
    let model = MainMenuModel::ctp2_landing(temp_dir.path().to_path_buf());
    let backend = RecordingMusicBackend::new();
    let mut session = MainMenuSession::new(model, backend);

    session.start_music().expect("music should start");
    session
        .start_music()
        .expect("second start should be ignored");

    assert!(session.music_started());
    let calls = session.audio_backend.calls.borrow();
    assert_eq!(calls.len(), 1, "autoplay should happen exactly once");
    assert!(calls[0].policy.autoplay);
    assert_eq!(calls[0].policy.loops, -1_i32);
    assert!(
        calls[0]
            .absolute_path
            .ends_with("ctp2_program/ctp/music/Track02.ogg"),
        "main menu should use the expected CTP2 music track"
    );
}

#[test]
fn original_button_art_assets_are_resolved_from_immutable_game_copy() {
    assert_eq!(
        runtime::button_asset_entry_name_for_test("upbt07aU.tga"),
        "upbt07au.rim"
    );
    assert_eq!(
        runtime::menu_texture_format_for_test(),
        sdl2::pixels::PixelFormatEnum::RGBA32,
        "RGBA byte data must be uploaded with SDL's endian-correct RGBA32 format"
    );

    for (style, style_name) in [
        (ButtonVisualStyle::blue_large(), "blue"),
        (ButtonVisualStyle::stone_large(), "stone"),
        (ButtonVisualStyle::ctp2_small(), "small popup"),
    ] {
        let summaries =
            runtime::button_style_asset_summaries_for_test(std::path::Path::new("ctp2"), style)
                .expect("button assets should load from original immutable CTP2 archives");

        assert_eq!(
            summaries.len(),
            12,
            "all {style_name} button state pieces should load"
        );
        let mut total_transparent_pixels = 0_usize;
        for (entry_name, width, height, transparent_pixels) in summaries {
            let has_rim_extension = std::path::Path::new(&entry_name)
                .extension()
                .is_some_and(|extension| extension.eq_ignore_ascii_case("rim"));
            assert!(
                has_rim_extension,
                "button asset should use packed RIM entry"
            );
            assert!(width > 0, "button asset width should be non-zero");
            assert!(height > 0, "button asset height should be non-zero");
            total_transparent_pixels = total_transparent_pixels.saturating_add(transparent_pixels);
        }
        assert!(
            total_transparent_pixels > 0,
            "{style_name} button style should apply chroma-key transparency somewhere in its original art"
        );
    }
}

#[test]
fn original_button_art_is_rendered_as_native_tiled_pieces() {
    let render_plan = runtime::button_triplet_render_plan_for_test(
        std::path::Path::new("ctp2"),
        ButtonVisualStyle::blue_large().assets.up,
        290,
    )
    .expect("original button triplet should produce a render plan");

    assert_eq!(
        render_plan,
        vec![
            ("upbt07au.rim".to_owned(), 35, 39, false),
            ("upbt07bu.rim".to_owned(), 220, 39, true),
            ("upbt07cu.rim".to_owned(), 35, 39, false),
        ],
        "button rendering should keep original endcaps native and tile only the middle piece"
    );
}

#[test]
fn hover_button_art_layers_match_original_ctp2_sizable_button_flags() {
    let render_plan = runtime::button_layer_render_plan_for_test(
        std::path::Path::new("ctp2"),
        ButtonVisualStyle::blue_large(),
        true,
        290,
    )
    .expect("hovered button layers should load from original art");

    assert_eq!(
        render_plan,
        vec![
            (
                "always".to_owned(),
                "upbt07ax.rim".to_owned(),
                35,
                39,
                false
            ),
            (
                "always".to_owned(),
                "upbt07bx.rim".to_owned(),
                220,
                39,
                true
            ),
            (
                "always".to_owned(),
                "upbt07cx.rim".to_owned(),
                35,
                39,
                false
            ),
            ("up".to_owned(), "upbt07au.rim".to_owned(), 35, 39, false),
            ("up".to_owned(), "upbt07bu.rim".to_owned(), 220, 39, true),
            ("up".to_owned(), "upbt07cu.rim".to_owned(), 35, 39, false),
            (
                "highlight".to_owned(),
                "upbt07ah.rim".to_owned(),
                35,
                39,
                false,
            ),
            (
                "highlight".to_owned(),
                "upbt07bh.rim".to_owned(),
                220,
                39,
                true,
            ),
            (
                "highlight".to_owned(),
                "upbt07ch.rim".to_owned(),
                35,
                39,
                false,
            ),
        ],
        "hover should draw the original always and up layers before the highlight overlay"
    );
}

#[test]
fn disabled_button_art_uses_only_original_inactive_layer() {
    let render_plan = runtime::disabled_button_layer_render_plan_for_test(
        std::path::Path::new("ctp2"),
        ButtonVisualStyle::blue_large(),
        146,
    )
    .expect("disabled button layers should load from original art");

    assert_eq!(
        render_plan,
        vec![
            (
                "disabled".to_owned(),
                "upbt07ax.rim".to_owned(),
                35,
                39,
                false,
            ),
            (
                "disabled".to_owned(),
                "upbt07bx.rim".to_owned(),
                76,
                39,
                true,
            ),
            (
                "disabled".to_owned(),
                "upbt07cx.rim".to_owned(),
                35,
                39,
                false,
            ),
        ],
        "disabled buttons should draw the original X-state triplet only, without U/H overlays"
    );
}

#[test]
fn button_labels_use_original_dark_text_color() {
    assert_eq!(
        runtime::button_label_text_color_for_test(),
        (0, 0, 0),
        "button labels should be dark over the original CTP2 button art"
    );
}

#[test]
fn shell_runtime_effect_helpers_match_original_option_scales() {
    assert_eq!(
        runtime::parse_resolution_label_for_test("1024 x 768"),
        Some((1024_u32, 768_u32)),
        "resolution picker labels should be parsed into SDL window dimensions"
    );
    assert_eq!(
        runtime::option_volume_to_mixer_volume_for_test(5_i32),
        64_i32,
        "midpoint shell volume should map to midpoint SDL_mixer volume"
    );
    assert_eq!(
        runtime::option_volume_to_mixer_volume_for_test(99_i32),
        128_i32,
        "shell volume should clamp to SDL_mixer maximum"
    );
    assert!(
        runtime::music_track_relative_path_for_test(3_usize)
            .ends_with("ctp2_program/ctp/music/Track03.ogg"),
        "music picker track 3 should map to original Track03.ogg"
    );
}

#[test]
fn headless_sdl_smoke_renders_menu_and_processes_quit_event() {
    let temp_dir = TempDir::new().expect("temp directory should be created");

    let result = run_headless_sdl_smoke_for_test(temp_dir.path().to_path_buf())
        .expect("headless SDL menu smoke test should render and exit");

    assert_eq!(result, MainMenuAction::Quit);
}

#[test]
fn headless_sdl_shell_options_back_and_quit_flow_exits_cleanly() {
    let temp_dir = TempDir::new().expect("temp directory should be created");
    let model = MainMenuModel::ctp2_landing(temp_dir.path().to_path_buf());

    let result = runtime::run_headless_events_for_test(
        &model,
        vec![
            mouse_down(623_i32, 471_i32),
            mouse_down(156_i32, 276_i32),
            mouse_down(311_i32, 240_i32),
            mouse_down(688_i32, 416_i32),
            mouse_down(538_i32, 454_i32),
            mouse_down(694_i32, 574_i32),
        ],
    )
    .expect("headless SDL options/back/quit flow should exit");

    assert_eq!(
        result,
        MainMenuAction::Quit,
        "options back flow should return to the hub and quit cleanly"
    );
}

#[test]
fn headless_sdl_shell_nation_picker_selects_from_original_grid() {
    let temp_dir = TempDir::new().expect("temp directory should be created");
    let model = MainMenuModel::ctp2_landing(temp_dir.path().to_path_buf());

    let result = runtime::run_headless_events_for_test(
        &model,
        vec![
            mouse_down(623_i32, 471_i32),
            mouse_down(644_i32, 246_i32),
            mouse_down(400_i32, 100_i32),
            mouse_down(260_i32, 120_i32),
            mouse_down(660_i32, 503_i32),
            mouse_down(400_i32, 574_i32),
        ],
    )
    .expect("headless SDL nation picker grid flow should reach launch");

    assert_eq!(
        result,
        MainMenuAction::LaunchSinglePlayer,
        "selecting a nation from the original popup grid should return to New Game and launch"
    );
}

#[test]
fn headless_sdl_shell_flow_reaches_single_player_launch() {
    let temp_dir = TempDir::new().expect("temp directory should be created");
    let model = MainMenuModel::ctp2_landing(temp_dir.path().to_path_buf());

    let result = runtime::run_headless_events_for_test(
        &model,
        vec![
            mouse_down(623_i32, 471_i32),
            mouse_down(644_i32, 246_i32),
            mouse_down(400_i32, 574_i32),
        ],
    )
    .expect("headless SDL shell flow should reach launch");

    assert_eq!(
        result,
        MainMenuAction::LaunchSinglePlayer,
        "single-player shell launch should return a launch action"
    );
}

#[test]
fn headless_rendered_menu_is_not_alpha_uploaded_as_red() {
    let (average_red, average_green, average_blue) =
        render_headless_menu_average_color_for_test(PathBuf::from("ctp2"))
            .expect("headless menu frame should render with original assets");

    assert!(
        average_red < 180,
        "average red {average_red} indicates alpha may be interpreted as red"
    );
    assert!(
        average_green > 20 && average_blue > 20,
        "background should retain non-red color channels, got ({average_red}, {average_green}, {average_blue})"
    );
}

#[test]
fn accessible_activation_reports_missing_buttons() {
    let model = MainMenuModel::ctp2_landing(PathBuf::from("/ctp2"));

    assert_eq!(
        model.activate_accessible("Load Game"),
        Err(MainMenuError::ButtonNotFound {
            accessible_name: "Load Game".to_owned(),
        })
    );
}
