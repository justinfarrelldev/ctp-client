//! One server-authoritative setup attempt; no automatic restart, rule substitution or cleanup.
use super::{ServerClient,ServerGameplaySession,LoginResponse,AI_COUNTRIES,json_i64,roster::PlayerRoster};
use crate::gameplay::{GameplayError,runtime_error};
use serde_json::{Value,json};
use std::time::{SystemTime,UNIX_EPOCH};

/// Fixed source route and bounded public failure; transport details may contain credentials.
fn failure(stage:&'static str,error:GameplayError)->GameplayError {
    let (status,message)=match error {
        GameplayError::Rejected {rejection}=>(Some(rejection.status),if rejection.code=="command_rejected"{"The server refused this setup request.".to_owned()}else{rejection.message}),
        GameplayError::Runtime {..}=>(None,"The server request could not complete. Check that the server is running, then try again.".to_owned()),
        GameplayError::Asset {..}=>(None,"The server returned an invalid setup response.".to_owned()),
        setup@GameplayError::Setup {..}=>return setup,
    };
    log::warn!("Game setup failed stage={stage} http_status={} reason={message}",status.map_or_else(||"unavailable".to_owned(),|status|status.to_string()));
    GameplayError::Setup {stage:stage.to_owned(),status,message}
}
/// An admission429 may retry the same body under the existing bounded policy;
/// semantic rejection/transport loss never creates a second setup or another country.
fn post(client:&ServerClient,path:&'static str,body:&Value)->Result<Value,GameplayError> {client.post_json(path,body).map_err(|error|failure(path,error))}
/// Creates the documented local game using the complete requested participant count.
pub(super) fn bootstrap(client:&ServerClient,leader_name:&str,empire_count:u8)->Result<ServerGameplaySession,GameplayError> {
    // Server internal/game MaxGamePlayers is8: reject, never truncate the requested roster.
    if !(1_u8..=8_u8).contains(&empire_count){return Err(GameplayError::Setup {stage:"setup options".to_owned(),status:None,message:"Choose between 1 and 8 empires; no server setup was created.".to_owned()});}
    client.check_health().map_err(|error|failure("/health",error))?;
    let login=login(client,leader_name)?;
    let game=post(client,"/game/create_game",&json!({"session_id":login.session_id,"owner_account_id":login.account_id,"name":format!("{leader_name}'s Local Game"),"style":"sequential","starting_settlers":1_i32,"starting_gold":0_i32,"is_public":false,"is_locked":true,"password_protected":false}))?;
    let game_id=json_i64(&game,"id").map_err(|error|failure("/game/create_game",error))?;
    post(client,"/game/select_country",&json!({"session_id":login.session_id,"game_id":game_id,"country":"Greeks"}))?;
    for (index,country) in AI_COUNTRIES.iter().take(usize::from(empire_count.saturating_sub(1_u8))).enumerate() {post(client,"/game/add_ai",&json!({"session_id":login.session_id,"game_id":game_id,"country":country,"name":format!("AI {}",index.saturating_add(1_usize))}))?;}
    let started=post(client,"/game/start_game",&json!({"session_id":login.session_id,"game_id":game_id}))?;
    let mut roster=PlayerRoster::from_game(&started,game_id,login.account_id).unwrap_or_else(|| {
        log::warn!("Original foreign player colors unavailable: start response lacks a complete bounded public roster");PlayerRoster::default().for_account(login.account_id)
    });
    let projection=client.fetch_resync_projection(game_id,login.session_id).map_err(|error|failure("/game/events",error))?;
    if let Some(received)=PlayerRoster::from_projection(&projection.interface){roster=received.for_account(login.account_id);}
    Ok(ServerGameplaySession {game_id,session_id:login.session_id,account_id:login.account_id,projection:Some(projection),roster})
}
/// Unique ephemeral login, retaining exact stage on failures without logging credentials.
fn login(client:&ServerClient,leader_name:&str)->Result<LoginResponse,GameplayError> {
    let suffix=SystemTime::now().duration_since(UNIX_EPOCH).map_err(|error|failure("/account/create_account",runtime_error(error.to_string())))?.as_nanos();
    let email=format!("ctp-client-{suffix}@local.invalid");let password=format!("ctp-client-{suffix}");
    post(client,"/account/create_account",&json!({"account":{"email":email,"name":leader_name,"experience_level":2_i32,"location":"Local","info":"Temporary local client account"},"password":password}))?;
    let response=post(client,"/account/login",&json!({"email":email,"password":password}))?;
    serde_json::from_value(response).map_err(|error|failure("/account/login",crate::gameplay::asset_error(error.to_string())))
}
