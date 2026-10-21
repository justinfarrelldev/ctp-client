//! Isolated gateway receipts for new parameterized and retained operations.
use std::{net::TcpListener,thread};
use serde_json::json;
use super::tests::{authority,respond,payload_after_validating_envelope};
use super::super::authority::state_from_authority_projection;
use crate::gameplay::ui_model::{InGameUiModel,management::{ManagerCommand,ManagerIntent}};

#[test]
fn new_operations_send_exact_envelopes_and_only_display_received_authority() {
    let cases=[
        ("rename-city",json!({"city_id":"c","name":"  New Rome  "})),
        ("edit-production",json!({"city_id":"c","entries":[{"entry_id":"c:2"},{"item_id":"UNIT_WARRIOR"},{"entry_id":"c:1"}]})),
        ("set-governor",json!({"city_id":"c","enabled":true,"priority":"science"})),
        ("set-research-goal",json!({"advance_id":"ADVANCE_BALLISTICS"})),
        ("sleep",json!({"unit_id":"u"})),
        ("wake",json!({"unit_id":"u"})),
        ("set-movement-order",json!({"unit_id":"u","target_x":10_u16,"target_y":8_u16})),
        ("cancel-movement-order",json!({"unit_id":"u"})),
    ];
    for (route,payload) in cases {
        let listener=TcpListener::bind("127.0.0.1:0").expect("isolated mock endpoint");
        let port=listener.local_addr().expect("ephemeral port").port();
        let initial=json!({"revision":17_i64,"turn":3_u32,"world":{"width":20_u16,"height":12_u16,"min_x":0_u16,"min_y":0_u16},"tiles":[{"id":"a","x":1_u16,"y":1_u16,"terrain":"plains"}],"units":[{"id":"u","tile_id":"a","owner_id":"account-7"}],"cities":[{"id":"c","tile_id":"a","owner_id":"account-7","name":"Rome","queue":[{"entry_id":"c:1","id":"UNIT_WARRIOR"},{"entry_id":"c:2","id":"UNIT_WARRIOR"}]}],"rules":{"units":[{"id":"UNIT_WARRIOR"}],"governor_priorities":["science"]},"action_intents":[{"route":"set-movement-order","entity_id":"u","payload":{"unit_id":"u"}}],"legal_actions":[{"route":route,"entity_id":if ["sleep","wake","cancel-movement-order"].contains(&route) {"u"} else if route == "set-research-goal" {"account-7"} else {"c"},"payload":payload}]});
        let mut returned=initial.clone();
        returned["revision"]=json!(18_i64);
        if route == "rename-city" { returned["cities"][0_usize]["name"]=json!("New Rome"); }
        if route == "edit-production" { returned["cities"][0_usize]["queue"]=json!([{"entry_id":"c:2","id":"UNIT_WARRIOR"},{"entry_id":"c:3","id":"UNIT_WARRIOR"},{"entry_id":"c:1","id":"UNIT_WARRIOR"}]); }
        if route == "set-governor" { returned["cities"][0_usize]["governor"]=json!({"enabled":true,"priority":"science"}); }
        if route == "set-research-goal" { returned["research"]=json!({"goal_id":"ADVANCE_BALLISTICS"}); }
        if route == "sleep" { returned["units"][0_usize]["asleep"]=json!(true); }
        let worker=thread::spawn(move || respond(&listener,&returned));
        let mut active=authority(port,initial);
        let mut state=state_from_authority_projection(&active,active.session.projection.as_ref().expect("snapshot"));
        state.popups.clear();
        let mut ui=InGameUiModel::default();
        super::apply(ManagerIntent::Submit(ManagerCommand {revision:17,route:route.to_owned(),payload:payload.clone()}),&mut state,&mut ui,Some(&mut active));
        let request=worker.join().expect("mock thread").expect("exact operation reached gateway");
        assert!(request.line.starts_with(&format!("POST /game/44/{route} ")),"exact route: {}",request.line);
        assert_eq!(payload_after_validating_envelope(&request.body),payload,"no display price, local costs or substituted operation");
        assert_eq!(state.cities[0].name,if route == "rename-city" {"New Rome"} else {"Rome"},"receipt replaces state, including trimmed server name");
        assert_eq!(active.session.revision(),18,"authoritative revision applied");
    }
}
