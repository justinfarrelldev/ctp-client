//! Server-observed national/city boundaries; native pixels stay in memory.
use crate::gameplay::{model::{GameState,MapPoint,PlayerId},runtime::{Camera,test_support::with_canvas,textures::GameplayTextures},server::ServerProjection};
use sdl2::pixels::{Color,PixelFormatEnum};
use serde_json::{Value,json};
use std::path::Path;

/// Public participant index disagrees with old setup order; no surviving own unit.
fn snapshot()->Value {json!({"revision":7_i64,"turn":1_u32,"viewer_player_id":"account-7","players":[{"id":"account-9","nation_id":"n9","color_index":0_u8},{"id":"account-7","nation_id":"n7","color_index":2_u8}],"tiles":[{"id":"center","x":1_u16,"y":1_u16,"terrain":"plains","owner_id":"account-7","influence_city_id":"city-a"},{"id":"a","x":2_u16,"y":2_u16,"terrain":"plains","owner_id":"account-7","influence_city_id":"city-a"},{"id":"b","x":3_u16,"y":2_u16,"terrain":"plains","owner_id":"account-7","influence_city_id":""}],"cities":[{"id":"city-a","owner_id":"account-7","tile_id":"center","population":1_u8,"name":"Rome"}]})}
/// Read only the projected tile's overlays, not the rest of the UI/map.
fn draw(value:Value,political:bool,influence:bool)->Result<Vec<u8>,String> {draw_with_camera(value,political,influence,Camera::for_test_identity())}
/// Exercise safe native clipping/current integer zoom without writing reference images.
fn draw_with_camera(value:Value,political:bool,influence:bool,camera:Camera)->Result<Vec<u8>,String> {
    let mut pixels=Vec::new();let result=with_canvas((800_u32,600_u32),|canvas| {
        let viewer:ServerProjection=serde_json::from_value(value).map_err(|error|error.to_string())?;let mut state=GameState::from_server_projection(&viewer,2_u8,1_u64);state.filters.political_borders=political;state.filters.city_influence=influence;
        let creator=canvas.texture_creator();let mut textures=GameplayTextures::load(&Path::new(env!("CARGO_MANIFEST_DIR")).join("ctp2"),&creator,true).map_err(|error|error.to_string())?;
        canvas.set_draw_color(Color::RGB(0_u8,0_u8,0_u8));canvas.clear();super::observations::draw(canvas,&mut textures,&state,&camera,MapPoint::new(2_u16,2_u16)).map_err(|error|error.to_string())?;pixels=canvas.read_pixels(None,PixelFormatEnum::RGBA32)?;Ok(())
    });result?;Ok(pixels)
}
/// Independent source `GenerateHitMask` southeast interior, including source headroom.
fn east_pixel(pixels:&[u8],row:usize,inset:usize)->&[u8] {
    let right=93_usize.saturating_sub(row.saturating_sub(24_usize).saturating_mul(2_usize));let offset=144_usize.saturating_add(row).saturating_mul(800_usize).saturating_add(right.saturating_sub(inset)).saturating_mul(4_usize);&pixels[offset..offset.saturating_add(3_usize)]
}
#[test]
fn nation_borders_city_influence_draws_original_white_two_on_two_off_rows_independently_of_nation_toggle() {
    let pixels=draw(snapshot(),false,true).expect("in-memory observed city boundary");
    for row in 24_usize..48_usize {for inset in 0_usize..3_usize {assert_eq!(east_pixel(&pixels,row,inset),if row&2_usize==0_usize {&[255_u8;3_usize]}else{&[0_u8;3_usize]},"original city-influence row{row}, inset{inset}, not a national/city-radius guess");}}
}
#[test]
fn nation_borders_projection_uses_persisted_participant_indices_and_explicit_viewer_without_units() {
    let viewer:ServerProjection=serde_json::from_value(snapshot()).expect("actual repaired wire schema");let state=GameState::from_server_projection(&viewer,2_u8,1_u64);
    assert_eq!(state.roster.slot("account-9"),Some(1_u8),"server participant0 maps original player1, not array/setup order");assert_eq!(state.roster.slot("account-7"),Some(3_u8),"persisted index2 maps original player3, reserving barbarian0");assert_eq!(state.cities[0_usize].owner,PlayerId::Human,"explicit viewer survives last settler founding");
}
#[test]
fn nation_borders_influence_unknown_and_hidden_city_facts_never_become_guessed_boundaries() {
    let mut cases=Vec::new();
    for index in [1_usize,2_usize] {let mut value=snapshot();value["tiles"][index]["influence_city_id"]=Value::Null;cases.push(value);let mut absent=snapshot();absent["tiles"][index].as_object_mut().expect("tile").remove("influence_city_id").expect("explicit field removed");cases.push(absent);}
    let mut missing=snapshot();missing["cities"]=json!([]);cases.push(missing);
    let mut wrong_city=snapshot();wrong_city["tiles"][1_usize]["influence_city_id"]=json!("another-city");cases.push(wrong_city);
    let mut hidden=snapshot();hidden["cities"][0_usize]["stale"]=json!(true);hidden["tiles"][0_usize]["stale"]=json!(true);cases.push(hidden);
    let mut no_center=snapshot();no_center["tiles"].as_array_mut().expect("tiles").remove(0_usize);cases.push(no_center);
    let mut no_influence=snapshot();no_influence["tiles"][1_usize]["influence_city_id"]=json!("");cases.push(no_influence);
    for value in cases {let pixels=draw(value,false,true).expect("read-only visibility case");assert!(pixels.chunks_exact(4_usize).all(|pixel|pixel[..3_usize]==[0_u8;3_usize]),"unknown assignment or hidden city's current visibility must not create an outline");}
    let off=draw(snapshot(),false,false).expect("both overlays disabled");assert!(off.chunks_exact(4_usize).all(|pixel|pixel[..3_usize]==[0_u8;3_usize]),"city-influence setting actually suppresses white pixels");
    let mut remembered=snapshot();remembered["tiles"][1_usize]["stale"]=json!(true);let expected=draw(snapshot(),false,true).expect("visible");assert_eq!(draw(remembered,false,true).expect("remembered assignment with permitted visible city"),expected,"remembered assignment remains observed, not reconstructed from a city radius");
    let mut paired=snapshot();paired["tiles"][2_usize]["influence_city_id"]=json!("another-city");assert_eq!(draw(paired,false,true).expect("same-nation city boundary, neighbor identity only"),expected,"different observed city assignment suffices, not its hidden name/center");
    let mut independent=snapshot();independent["tiles"][1_usize]["owner_id"]=Value::Null;assert_eq!(draw(independent,false,true).expect("influence without national fact"),expected,"known influence must not require a guessed national owner");
}
#[test]
fn nation_borders_known_unowned_differs_from_unknown_and_all_four_sides_follow_source_hit_mask() {
    for (dx,dy,rows,west) in [(1_u16,0_u16,24_usize..48_usize,false),(0_u16,1_u16,24_usize..48_usize,true),(0_u16,0_u16,0_usize..24_usize,true),(2_u16,0_u16,0_usize..24_usize,false)] {
        let mut value=snapshot();let (x,y)=if dx==0_u16&&dy==0_u16 {(1_u16,2_u16)}else if dx==2_u16 {(2_u16,1_u16)}else{(2_u16.saturating_add(dx),2_u16.saturating_add(dy))};value["tiles"][2_usize]["x"]=json!(x);value["tiles"][2_usize]["y"]=json!(y);value["tiles"][2_usize]["owner_id"]=json!("");
        let pixels=draw(value.clone(),true,false).expect("explicit observed unowned border");
        for row in rows {let left=if row<24_usize {46_usize.saturating_sub(row.saturating_mul(2_usize))}else{row.saturating_sub(24_usize).saturating_mul(2_usize)};let column=if west {left}else{91_usize.saturating_sub(left)};let offset=144_usize.saturating_add(row).saturating_mul(800_usize).saturating_add(column).saturating_mul(4_usize);assert_ne!(&pixels[offset..offset.saturating_add(3_usize)],&[0_u8;3_usize],"solid owned/unowned side row{row}");}
        value["tiles"][2_usize]["owner_id"]=Value::Null;let unknown=draw(value,true,false).expect("unknown owner");assert!(unknown.chunks_exact(4_usize).all(|pixel|pixel[..3_usize]==[0_u8;3_usize]),"null ownership never substitutes known unowned");
    }
}
#[test]
fn nation_borders_cached_receipts_apply_growth_shrink_capture_visibility_and_removal_without_ghosts() {
    let result=with_canvas((800_u32,600_u32),|canvas| {
        let creator=canvas.texture_creator();let mut textures=GameplayTextures::load(&Path::new(env!("CARGO_MANIFEST_DIR")).join("ctp2"),&creator,true).map_err(|error|error.to_string())?;let camera=Camera::for_test_identity();
        for (revision,stage,white) in [(7_i64,"initial",true),(8_i64,"growth",false),(9_i64,"shrink",true),(10_i64,"hidden-capture",false),(11_i64,"reobserved",true),(12_i64,"removed",false)] {
            let mut value=snapshot();value["revision"]=json!(revision);
            match stage {"growth"=>{value["cities"][0_usize]["population"]=json!(7_u8);value["tiles"][2_usize]["influence_city_id"]=json!("city-a");},"shrink"=>value["tiles"][2_usize]["influence_city_id"]=json!(""),"hidden-capture"=>{value["cities"][0_usize]["stale"]=json!(true);value["tiles"][0_usize]["stale"]=json!(true);value["tiles"][1_usize]["stale"]=json!(true);},"removed"=>{value["cities"]=json!([]);value["tiles"][1_usize]["influence_city_id"]=json!("");},"reobserved"=>{value["cities"][0_usize]["owner_id"]=json!("account-9");value["tiles"][0_usize]["owner_id"]=json!("account-9");value["tiles"][1_usize]["owner_id"]=json!("account-9");},"initial"=>{},_=>return Err("unknown test stage".to_owned())}
            let viewer:ServerProjection=serde_json::from_value(value).map_err(|error|error.to_string())?;let mut state=GameState::from_server_projection(&viewer,2_u8,1_u64);state.filters.political_borders=false;
            canvas.set_draw_color(Color::RGB(0_u8,0_u8,0_u8));canvas.clear();super::observations::draw(canvas,&mut textures,&state,&camera,MapPoint::new(2_u16,2_u16)).map_err(|error|error.to_string())?;let pixels=canvas.read_pixels(None,PixelFormatEnum::RGBA32)?;
            if pixels.chunks_exact(4_usize).any(|pixel|pixel[..3_usize]==[255_u8;3_usize])!=white {return Err(format!("cached observed assignments/visibility not refreshed at {stage}"));}
        }
        Ok(())
    });assert!(result.is_ok(),"native common-viewer receipt lifecycle: {result:?}");
}
#[test]
fn nation_borders_native_row_raster_keeps_current_zoom_and_partially_clipped_tiles() {
    for zoom in 1_u32..=3_u32 {let mut camera=Camera::for_test_identity();for _ in 1_u32..zoom {camera.zoom_in();}let (x,y)=camera.tile_to_screen(MapPoint::new(2_u16,2_u16));let pixels=draw_with_camera(snapshot(),false,true,camera).expect("current integer zoom");let scale=i32::try_from(zoom).expect("bounded zoom");let left=x.saturating_add(83_i32.saturating_mul(scale));let top=y.saturating_add(52_i32.saturating_mul(scale));for dx in 0_i32..3_i32.saturating_mul(scale) {let offset=usize::try_from(top.saturating_mul(800_i32).saturating_add(left.saturating_add(dx)).saturating_mul(4_i32)).expect("onscreen");assert_eq!(&pixels[offset..offset.saturating_add(3_usize)],&[255_u8;3_usize],"three source pixels expand only by existing zoom");}}
    let mut camera=Camera::for_test_identity();camera.pan_pixels(70_i32,0_i32);assert!(camera.tile_to_screen(MapPoint::new(2_u16,2_u16)).0<0_i32,"fixture must actually clip the tile");let pixels=draw_with_camera(snapshot(),false,true,camera).expect("partly clipped original tile");assert!(pixels.chunks_exact(4_usize).any(|pixel|pixel[..3_usize]==[255_u8;3_usize]),"fix original all-or-nothing offscreen tile rejection without unsafe writes");
}
#[test]
fn nation_borders_explicit_unknown_or_invalid_palette_never_falls_back_to_setup_or_self_blue() {
    for players in [Value::Null,json!([{"id":"account-7","nation_id":"n7"}]),json!([{"id":"account-7","nation_id":"n7","color_index":null}]),json!([{"id":"account-7","nation_id":"n7","color_index":16_u8}]),json!([{"id":"account-7","nation_id":"n7","color_index":-1_i8}]),json!([{"id":"account-7","nation_id":"n7","color_index":0_u8},{"id":"account-9","nation_id":"n9","color_index":0_u8}]),json!([{"id":"account-7","nation_id":"n7","color_index":0_u8},{"id":"account-7","nation_id":"n7","color_index":1_u8}])] {
        let mut value=snapshot();value["players"]=players;let viewer:ServerProjection=serde_json::from_value(value).expect("unavailable public metadata must not break snapshot");let state=GameState::from_server_projection(&viewer,2_u8,1_u64);assert!(state.roster.uses_server_slots(),"new schema cannot become legacy guessing");assert_eq!(PlayerId::Human.color_slot(&state.roster),None,"unknown persisted self color is not blue/zero");
    }
    let mut value=snapshot();value["players"].as_array_mut().expect("public players").reverse();let viewer:ServerProjection=serde_json::from_value(value).expect("reordered public list");let state=GameState::from_server_projection(&viewer,2_u8,1_u64);assert_eq!(state.roster.slot("account-7"),Some(3_u8),"array order cannot reassign persisted slots");
}
