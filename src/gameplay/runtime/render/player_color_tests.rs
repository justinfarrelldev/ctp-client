//! Native original-palette herald/flag/city pixels; no captures, files or live services.
use crate::gameplay::{
    model::{CityId,GameState,MapPoint,PlayerId},
    runtime::{Camera,MovementInputState,test_support::with_canvas,textures::GameplayTextures},
    server::{ServerProjection,roster::PlayerRoster},
    ui_model::InGameUiModel,
};
use sdl2::pixels::{Color,PixelFormatEnum};
use serde_json::json;
use alloc::collections::BTreeSet;
use std::path::Path;

/// Public membership is fixed before sightings; the viewer deliberately occupies slot4.
fn fixture(remembered:bool)->GameState {
    let viewer:ServerProjection=serde_json::from_value(json!({"revision":7_i64,"turn":1_u32,
        "tiles":[{"id":"a","x":2_u16,"y":2_u16,"terrain":"plains","stale":remembered},{"id":"b","x":5_u16,"y":2_u16,"terrain":"plains","stale":remembered},{"id":"c","x":8_u16,"y":2_u16,"terrain":"plains","stale":remembered}],
        "units":[{"id":"u42","type_id":"UNIT_WARRIOR","owner_id":"account-42","tile_id":"a"},{"id":"u43","type_id":"UNIT_WARRIOR","owner_id":"account-43","tile_id":"b"},{"id":"u44","type_id":"UNIT_WARRIOR","owner_id":"account-44","tile_id":"c"}],
        "cities":[{"id":"c42","owner_id":"account-42","tile_id":"a","population":1_u32,"name":"A","stale":remembered},{"id":"c43","owner_id":"account-43","tile_id":"b","population":1_u32,"name":"B","stale":remembered},{"id":"c44","owner_id":"account-44","tile_id":"c","population":1_u32,"name":"C","stale":remembered}]
    })).expect("permitted observations");
    let mut state=GameState::from_server_projection(&viewer,4_u8,1_u64);
    state.roster=PlayerRoster::from_game(&json!({"id":44_i64,"players":[{"id":91_i64,"account_id":42_i64},{"id":92_i64,"account_id":43_i64},{"id":93_i64,"account_id":44_i64},{"id":94_i64,"account_id":7_i64}]}),44_i64,7_i64).expect("accepted public roster");state
}
#[test]
fn player_colors_native_flags_heralds_city_boxes_and_fog_share_distinct_original_palette_slots() {
    let result=with_canvas((800_u32,600_u32),|canvas| {
        let root=Path::new(env!("CARGO_MANIFEST_DIR")).join("ctp2");let creator=canvas.texture_creator();let mut textures=GameplayTextures::load(&root,&creator,true).map_err(|error|error.to_string())?;let camera=Camera::for_test_identity();
        let state=fixture(false);let colors:Vec<_>=state.units.iter().map(|unit|textures.player_color(&state,&unit.owner,false)).collect();
        let unique:BTreeSet<_>=colors.iter().map(|color|(color.r,color.g,color.b)).collect();if unique.len()!=3_usize {return Err("foreign flags still collapse to one color".to_owned());}
        canvas.set_draw_color(Color::RGB(0_u8,0_u8,0_u8));canvas.clear();super::actors::draw_units(canvas,&mut textures,&state,&camera,&MovementInputState::default()).map_err(|error|error.to_string())?;
        let actors=canvas.read_pixels(None,PixelFormatEnum::RGBA32)?;
        for color in &colors {if !actors.chunks_exact(4_usize).any(|pixel|pixel[..3_usize]==[color.r,color.g,color.b]){return Err("original actor herald/flag palette color missing".to_owned());}}
        for remembered in [false,true] {
            let observed=fixture(remembered);canvas.set_draw_color(Color::RGB(0_u8,0_u8,0_u8));canvas.clear();super::cities::draw_labels(canvas,&textures,&observed,&InGameUiModel::default(),&camera,None).map_err(|error|error.to_string())?;let pixels=canvas.read_pixels(None,PixelFormatEnum::RGBA32)?;
            for city in &observed.cities {
                let (x,y)=camera.tile_to_screen(city.position);let offset=usize::try_from(y.saturating_sub(11_i32).saturating_mul(800_i32).saturating_add(x.saturating_add(1_i32)).saturating_mul(4_i32)).map_err(|error|error.to_string())?;let color=textures.player_color(&observed,&city.owner,remembered);
                if pixels[offset..offset.saturating_add(3_usize)]!=[color.r,color.g,color.b]{return Err("city population box does not share the original normal/dark owner color".to_owned());}
                let raw=observed.viewer.as_ref().and_then(|viewer|viewer.cities.iter().find(|raw|CityId::from_server_id(&raw.id)==city.id)).ok_or_else(||"exact observed city".to_owned())?;
                if textures.observed_owner_color(&observed,&raw.owner_id)!=textures.player_color(&observed,&city.owner,false) {return Err("territorial owner uses a different normal palette".to_owned());}
            }
        }
        for remembered in [false,true] {
            let mut viewer:ServerProjection=serde_json::from_value(json!({"revision":if remembered {28_i64}else{8_i64},"turn":1_u32,"tiles":[{"id":"a","x":2_u16,"y":2_u16,"terrain":"plains","owner_id":"account-42","stale":remembered},{"id":"b","x":3_u16,"y":2_u16,"terrain":"plains","owner_id":"account-43","stale":remembered}]})).map_err(|error|error.to_string())?;
            for known_neighbor in [true,false] {
                if !known_neighbor {viewer.revision=viewer.revision.saturating_add(10_i64);viewer.tiles.retain(|tile|tile.id=="a");}
                let mut observed=GameState::from_server_projection(&viewer,4_u8,1_u64);observed.roster=state.roster.clone();observed.filters.political_borders=true;
                canvas.set_draw_color(Color::RGB(0_u8,0_u8,0_u8));canvas.clear();super::observations::draw(canvas,&mut textures,&observed,&camera,MapPoint::new(2_u16,2_u16)).map_err(|error|error.to_string())?;let pixels=canvas.read_pixels(None,PixelFormatEnum::RGBA32)?;
                if known_neighbor {
                    let (x,y)=camera.tile_to_screen(MapPoint::new(2_u16,2_u16));let offset=usize::try_from(y.saturating_add(48_i32).saturating_mul(800_i32).saturating_add(x.saturating_add(92_i32)).saturating_mul(4_i32)).map_err(|error|error.to_string())?;let color=textures.player_color(&observed,&PlayerId::Server("account-42".to_owned()),false);
                    if pixels[offset..offset.saturating_add(3_usize)]!=[color.r,color.g,color.b]{return Err("visible/remembered political border must use original NORMAL player color".to_owned());}
                } else if pixels.chunks_exact(4_usize).any(|pixel|pixel[..3_usize]!=[0_u8;3_usize]) {return Err("public roster fabricated a border against an unknown neighbor".to_owned());}
            }
        }
        let mut unavailable=state.clone();unavailable.roster=PlayerRoster::default().for_account(7_i64);
        assert_eq!(textures.observed_owner_color(&unavailable,"account-7"),textures.player_color(&unavailable,&PlayerId::Human,false),"missing roster preserves exact self-only fallback");
        assert_eq!(textures.observed_owner_color(&unavailable,"human"),textures.player_color(&unavailable,&PlayerId::Server("human".to_owned()),false),"authenticated foreign legacy-looking ID is neutral, never mistaken for self");
        assert_eq!(textures.observed_owner_color(&state,"account-7"),textures.player_color(&state,&PlayerId::Human,false),"known own slot4 is not forced blue");
        assert_ne!(textures.player_color(&state,&PlayerId::Barbarians,false),colors[0_usize],"barbarians use source slot0, not the same first civilization color");
        assert_eq!(PlayerId::Server("unlisted".to_owned()).color_slot(&state.roster),None,"missing metadata never guesses a native slot");
        assert!(state.tile(MapPoint::new(20_u16,10_u16)).is_some_and(|tile|tile.fog==crate::gameplay::model::FogState::Hidden),"public membership never fabricates hidden geography");
        Ok(())
    });
    assert!(result.is_ok(),"native source owner colors: {result:?}");
}
