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
fn landing_page_button_hitboxes_include_rendered_button_art() {
    let model = MainMenuModel::ctp2_landing(PathBuf::from("/ctp2"));
    let single_player = model
        .button_by_id(MainMenuButtonId::SinglePlayer)
        .expect("single-player button should exist");
    let ldl_bottom = single_player
        .rect
        .y
        .saturating_add(i32::try_from(single_player.rect.height).expect("height should fit i32"));

    assert_eq!(
        model.activate_at_point(623_i32, ldl_bottom.saturating_add(4_i32)),
        Some(MainMenuAction::SinglePlayer),
        "button hitboxes should include the visible native 39px button art, not only the shorter LDL height"
    );
}

#[test]
fn shared_button_render_geometry_preserves_hover_and_disabled_rules() {
    let rect = MenuRect::new(10, 20, 100, 20);

    assert_eq!(
        runtime::button_render_geometry_for_test(rect, ButtonStyleId::BlueLarge, true, 50, 55,),
        (MenuRect::new(10, 20, 100, 39), true),
        "enabled large buttons should hover across the full native CTP2 art height"
    );
    assert_eq!(
        runtime::button_render_geometry_for_test(rect, ButtonStyleId::BlueLarge, false, 50, 55,),
        (MenuRect::new(10, 20, 100, 39), false),
        "disabled buttons should keep the rendered hit area but never enter hover state"
    );
    assert_eq!(
        runtime::button_render_geometry_for_test(
            MenuRect::new(1, 2, 70, 24),
            ButtonStyleId::Ctp2Small,
            true,
            70,
            25,
        ),
        (MenuRect::new(1, 2, 70, 24), true),
        "small popup buttons should keep their native 24px art bounds"
    );
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
fn original_button_click_sound_is_resolved_from_original_game_assets() {
    let (archive, entry, byte_len, riff_header) =
        runtime::button_click_sound_summary_for_test(std::path::Path::new("ctp2"))
            .expect("button click sound should load from original immutable CTP2 sound archive");

    assert_eq!(
        archive, "ctp2_data/default/sound/sound.zfs",
        "button click should come from the base CTP2 default sound archive"
    );
    assert_eq!(
        entry, "cclick1.wav",
        "AUI execute click maps through SOUND_ID_BUTTONCLICK to CCLICK1.WAV"
    );
    assert!(
        byte_len > 44,
        "WAV data should include a header and payload"
    );
    assert_eq!(
        riff_header, *b"RIFF",
        "button click should be WAV/RIFF data"
    );
}

#[test]
fn original_slider_sounds_are_resolved_from_original_game_assets() {
    let summaries = runtime::slider_sound_summaries_for_test(std::path::Path::new("ctp2"))
        .expect("slider sounds should load from original immutable CTP2 sound archive");

    assert_eq!(
        summaries.len(),
        2_usize,
        "C3Slider should provide separate up and down sound effects"
    );
    assert_eq!(
        summaries[0].1, "cslideup.wav",
        "GAMESOUNDS_SLIDERUP maps through SOUND_ID_SLIDERUP to CSLIDEUP.WAV"
    );
    assert_eq!(
        summaries[1].1, "cslidedn.wav",
        "GAMESOUNDS_SLIDERDOWN maps through SOUND_ID_SLIDERDOWN to CSLIDEDN.WAV"
    );
    for (archive, _entry, byte_len, riff_header) in summaries {
        assert_eq!(
            archive, "ctp2_data/default/sound/sound.zfs",
            "slider sounds should come from the base CTP2 default sound archive"
        );
        assert!(
            byte_len > 44,
            "WAV data should include a header and payload"
        );
        assert_eq!(
            riff_header, *b"RIFF",
            "slider sound should be WAV/RIFF data"
        );
    }
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
fn listbox_scrollbar_arrows_have_visible_overlay_glyphs() {
    let plan =
        runtime::listbox_scrollbar_arrow_glyph_plan_for_test(MenuRect::new(303, 238, 177, 89));

    assert_eq!(
        plan.len(),
        2_usize,
        "vertical listbox scrollbars should draw visible up and down arrow glyph overlays"
    );
    assert!(
        plan.iter()
            .any(|glyph| glyph.0 == "up" && glyph.1.height > 0),
        "the top arrow button should include a visible up-arrow glyph"
    );
    assert!(
        plan.iter()
            .any(|glyph| glyph.0 == "down" && glyph.1.height > 0),
        "the bottom arrow button should include a visible down-arrow glyph"
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
fn shell_left_top_descriptions_use_original_black_text_color() {
    assert_eq!(
        runtime::shell_left_top_text_color_for_test(),
        runtime::button_label_text_color_for_test(),
        "world-shape side descriptions should use black shell text, not the blue version-label color"
    );
}

#[test]
fn leader_name_initial_backspace_removes_one_character() {
    assert_eq!(
        runtime::leader_name_after_initial_backspace_for_test(),
        "Julius Caesa",
        "first Backspace after focusing the leader-name field should not clear the whole name"
    );
}

#[test]
fn leader_name_accepts_keydown_text_entry_without_text_input_duplicates() {
    assert_eq!(
        runtime::leader_name_after_keydown_edit_for_test(),
        "Ninjaboy",
        "focused leader-name field should accept normal keydown text and suppress duplicate SDL text-input echoes"
    );
}

#[test]
fn new_game_header_labels_draw_above_overlapping_button_art() {
    let draw_order = runtime::shell_control_draw_order_for_test(shell::ShellScreenId::NewGame);
    let difficulty_label_index = draw_order
        .iter()
        .position(|name| *name == "DifficultyLabel")
        .expect("DifficultyLabel should be drawn");
    let difficulty_button_index = draw_order
        .iter()
        .position(|name| *name == "DifficultyButton")
        .expect("DifficultyButton should be drawn");

    assert!(
        difficulty_label_index > difficulty_button_index,
        "header labels should render after overlapping native-height button art so their text is not clipped"
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
        runtime::initial_music_track_for_test(2_usize, false, 0_u64),
        2_usize,
        "non-random startup should respect the persisted selected track"
    );
    assert_eq!(
        runtime::initial_music_track_for_test(1_usize, true, 0_u64),
        2_usize,
        "random startup should not always replay the persisted first track"
    );
    assert!(
        runtime::mixer_chunk_size_for_test() <= 512_i32,
        "UI SFX should use a low-latency SDL_mixer chunk size"
    );
    assert!(
        runtime::mixer_allocated_channels_for_test() >= 8_i32,
        "UI SFX should have enough channels for overlapping click and slider sounds"
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
        runtime::music_track_relative_path_for_test(1_usize)
            .ends_with("ctp2_program/ctp/music/Track02.ogg"),
        "music picker row 1 (A Fortiori) should map to original CD track/file Track02"
    );
    assert!(
        runtime::music_track_relative_path_for_test(10_usize)
            .ends_with("ctp2_program/ctp/music/Track11.ogg"),
        "music picker row 10 should map to the final original CD track/file Track11"
    );
}

#[test]
fn music_completion_matches_original_auto_repeat_semantics() {
    assert_eq!(
        runtime::next_music_track_after_finish_for_test(3_usize, false, true, false, 0_u64),
        Some(1_usize),
        "Random Order should continue to another track after Amaethon even when Auto Repeat is off"
    );
    assert_eq!(
        runtime::next_music_track_after_finish_for_test(10_usize, false, false, false, 0_u64),
        None,
        "playlist playback should stop at the final row when Auto Repeat is off"
    );
    assert_eq!(
        runtime::next_music_track_after_finish_for_test(10_usize, true, false, false, 0_u64),
        Some(1_usize),
        "playlist playback should wrap at the final row when Auto Repeat is on"
    );
    assert_eq!(
        runtime::next_music_track_after_finish_for_test(1_usize, false, false, true, 0_u64),
        None,
        "a user-picked track should play once when Auto Repeat is off"
    );
    assert_eq!(
        runtime::next_music_track_after_finish_for_test(1_usize, true, false, true, 0_u64),
        Some(1_usize),
        "a user-picked track should repeat the same row when Auto Repeat is on"
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
fn high_resolution_menu_preparation_keeps_physical_mouse_coordinates() {
    let temp_dir = TempDir::new().expect("temp directory should be created");

    let state = run_with_headless_sdl_environment(temp_dir.path().to_path_buf(), |model| {
        runtime::prepared_canvas_state_for_test(model, (1920_u32, 1080_u32))
    })
    .expect("headless renderer state should be readable");

    assert_eq!(
        state.logical_size,
        (0_u32, 0_u32),
        "renderer logical size must stay disabled so SDL does not rewrite mouse coordinates before menu hit-testing"
    );
    assert_eq!(
        state.viewport.height, state.output_size.1,
        "the prepared renderer should fill the usable physical output height instead of drawing through an 800x600 logical renderer"
    );
}

#[test]
fn resize_from_widescreen_to_1280x768_leaves_no_unpainted_menu_lines() {
    let temp_dir = TempDir::new().expect("temp directory should be created");

    let black_pixels = run_with_headless_sdl_environment(temp_dir.path().to_path_buf(), |model| {
        runtime::black_pixels_inside_viewport_after_resize_for_test(
            model,
            (1920_u32, 1080_u32),
            (1280_u32, 768_u32),
        )
    })
    .expect("headless resize render should be readable");

    assert_eq!(
        black_pixels, 0_usize,
        "the active menu viewport should be fully repainted after resizing from 1920x1080 to 1280x768"
    );
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
