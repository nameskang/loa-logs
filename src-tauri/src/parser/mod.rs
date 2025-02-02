pub mod encounter_state;
mod entity_tracker;
mod id_tracker;
pub mod models;
mod party_tracker;
mod status_tracker;

#[macro_use]
mod maros;

use crate::parser::encounter_state::{EncounterState, get_class_from_id};
use crate::parser::entity_tracker::{get_current_and_max_hp, EntityTracker};
use crate::parser::id_tracker::IdTracker;
use crate::parser::models::EntityType;
use crate::parser::party_tracker::PartyTracker;
use crate::parser::status_tracker::{StatusEffectTargetType, StatusTracker};
use anyhow::Result;
use chrono::Utc;
use log::{warn, info};
use meter_core::packets::definitions::*;
use meter_core::packets::opcodes::Pkt;
use meter_core::{start_capture, start_raw_capture};
use std::cell::RefCell;
use std::rc::Rc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::thread;
use std::time::{Duration, Instant};
use tauri::{Manager, Window, Wry};

pub fn start(window: Window<Wry>, ip: String, port: u16, raw_socket: bool) -> Result<()> {
    let id_tracker = Rc::new(RefCell::new(IdTracker::new()));
    let party_tracker = Rc::new(RefCell::new(PartyTracker::new(id_tracker.clone())));
    let status_tracker = Rc::new(RefCell::new(StatusTracker::new(party_tracker.clone())));
    let mut entity_tracker = EntityTracker::new(
        status_tracker.clone(),
        id_tracker.clone(),
        party_tracker.clone(),
    );
    let mut state = EncounterState::new(window.clone());
    let rx = if raw_socket {
        if !meter_core::check_is_admin() {
            warn!("Not running as admin, cannot use raw socket");
            loop {
                window.emit("admin", "")?;
                thread::sleep(Duration::from_millis(5000));
            }
        }
        meter_core::add_firewall()?;
        match start_raw_capture(ip, port) {
            Ok(rx) => rx,
            Err(e) => {
                warn!("Error starting capture: {}", e);
                return Ok(());
            }
        }
    } else {
        match start_capture(ip, port) {
            Ok(rx) => rx,
            Err(e) => {
                warn!("Error starting capture: {}", e);
                return Ok(());
            }
        }
    };

    let mut last_update = Instant::now();
    let duration = Duration::from_millis(100);

    let reset = Arc::new(AtomicBool::new(false));
    let pause = Arc::new(AtomicBool::new(false));

    let meter_window_clone = window.clone();
    window.listen_global("reset-request", {
        let reset_clone = reset.clone();
        let meter_window_clone = meter_window_clone.clone();
        move |_event| {
            reset_clone.store(true, Ordering::Relaxed);
            info!("resetting meter");
            meter_window_clone.emit("reset-encounter", "").ok();
        }
    });
    
    window.listen_global("pause-request", {
        let pause_clone = pause.clone();
        move |_event| {
            let prev = pause_clone.fetch_xor(true, Ordering::Relaxed);
            if prev {
                info!("unpausing meter");
            } else {
                info!("pausing meter");
            }
            meter_window_clone.emit("pause-encounter", "").ok();
        }
    });

    while let Ok((op, data)) = rx.recv() {
        if reset.load(Ordering::Relaxed) {
            state.soft_reset(true);
            reset.store(false, Ordering::Relaxed);
        }
        if pause.load(Ordering::Relaxed) {
            continue;
        }
        match op {
            Pkt::CounterAttackNotify => {
                if let Some(pkt) = parse_pkt(&data, PKTCounterAttackNotify::new, "PKTCounterAttackNotify") {
                    if let Some(entity) = entity_tracker.entities.get(&pkt.source_id) {
                        state.on_counterattack(entity);
                    }
                }
            }
            Pkt::DeathNotify => {
                if let Some(pkt) = parse_pkt(&data, PKTDeathNotify::new, "PKTDeathNotify") {
                    if let Some(entity) = entity_tracker.entities.get(&pkt.target_id) {
                        debug_print!("death", &(&entity.name, entity.entity_type, entity.id));
                        state.on_death(entity);
                    }
                }
            }
            Pkt::IdentityGaugeChangeNotify => {
                if let Some(pkt) = parse_pkt(&data, PKTIdentityGaugeChangeNotify::new, "PKTIdentityGaugeChangeNotify") {
                    state.on_identity_gain(pkt);
                }
            }
            Pkt::InitEnv => {
                if let Some(pkt) = parse_pkt(&data, PKTInitEnv::new, "PKTInitEnv") {
                    let entity = entity_tracker.init_env(pkt);
                    debug_print!("init env", &entity);
                    state.on_init_env(entity);
                }
            }
            Pkt::InitPC => {
                if let Some(pkt) = parse_pkt(&data, PKTInitPC::new, "PKTInitPC") {
                    let (hp, max_hp) = get_current_and_max_hp(&pkt.stat_pair);
                    let entity = entity_tracker.init_pc(pkt);
                    info!("local player: {:?}, class: {:?}, ilvl: {:?}, id: {:?}", entity.name, get_class_from_id(&entity.class_id), entity.gear_level, entity.character_id);
                    // debug_print!("init pc", &entity);

                    state.on_init_pc(entity, hp, max_hp)
                }
            }
            Pkt::MigrationExecute => {
                if let Some(pkt) = parse_pkt(&data, PKTMigrationExecute::new, "PKTMigrationExecute") {
                    entity_tracker.migration_execute(pkt);
                }
            }
            Pkt::NewPC => {
                if let Some(pkt) = parse_pkt(&data, PKTNewPC::new, "PKTNewPC") {
                    let (hp, max_hp) = get_current_and_max_hp(&pkt.pc_struct.stat_pair);
                    let entity = entity_tracker.new_pc(pkt);
                    debug_print!("new pc", &(&entity.name, get_class_from_id(&entity.class_id), entity.id, entity.character_id, entity.gear_level));
                    state.on_new_pc(entity, hp, max_hp);
                }
            }
            Pkt::NewNpc => {
                if let Some(pkt) = parse_pkt(&data, PKTNewNpc::new, "PKTNewNpc") {
                    let (hp, max_hp) = get_current_and_max_hp(&pkt.npc_struct.stat_pair);
                    let entity = entity_tracker.new_npc(pkt, max_hp);
                    debug_print!("new npc", &(&entity.name, entity.entity_type, entity.id, entity.npc_id, hp, max_hp));
                    state.on_new_npc(entity, hp, max_hp);
                }
            }
            Pkt::NewNpcSummon => {
                if let Some(pkt) = parse_pkt(&data, PKTNewNpcSummon::new, "PKTNewNpcSummon") {
                    let (hp, max_hp) = get_current_and_max_hp(&pkt.npc_data.stat_pair);
                    let entity = entity_tracker.new_npc_summon(pkt, max_hp);
                    debug_print!("new summon", &(&entity.name, entity.entity_type, entity.id, entity.npc_id, entity.owner_id, hp, max_hp));
                    state.on_new_npc(entity, hp, max_hp);
                }
            }
            Pkt::NewProjectile => {
                if let Some(pkt) = parse_pkt(&data, PKTNewProjectile::new, "PKTNewProjectile") {
                    entity_tracker.new_projectile(pkt);
                }
            }
            Pkt::ParalyzationStateNotify => {
                if let Some(pkt) = parse_pkt(&data, PKTParalyzationStateNotify::new, "PKTParalyzationStateNotify") {
                    state.on_stagger_change(pkt);
                }
            }
            Pkt::PartyInfo => {
                if let Some(pkt) = parse_pkt(&data, PKTPartyInfo::new, "PKTPartyInfo") {
                    entity_tracker.party_info(pkt);
                    let local_player_id = entity_tracker.local_player_id;
                    if let Some(entity) = entity_tracker.entities.get(&local_player_id) {
                        state.update_local_player(entity);
                    }
                }
            }
            Pkt::PartyLeaveResult => {
                if let Some(pkt) = parse_pkt(&data, PKTPartyLeaveResult::new, "PKTPartyLeaveResult") {
                    party_tracker
                        .borrow_mut()
                        .remove(pkt.party_instance_id, pkt.name);
                }
            }
            Pkt::PartyStatusEffectAddNotify => {
                if let Some(pkt) = parse_pkt(&data, PKTPartyStatusEffectAddNotify::new, "PKTPartyStatusEffectAddNotify") {
                    entity_tracker.party_status_effect_add(pkt);
                }
            }
            Pkt::PartyStatusEffectRemoveNotify => {
                if let Some(pkt) = parse_pkt(&data, PKTPartyStatusEffectRemoveNotify::new, "PKTPartyStatusEffectRemoveNotify") {
                    entity_tracker.party_status_effect_remove(pkt);
                }
            }
            Pkt::PartyStatusEffectResultNotify => {
                if let Some(pkt) = parse_pkt(&data, PKTPartyStatusEffectResultNotify::new, "PKTPartyStatusEffectResultNotify") {
                    party_tracker.borrow_mut().add(
                        pkt.raid_instance_id,
                        pkt.party_instance_id,
                        pkt.character_id,
                        0,
                        None,
                    );
                }
            }
            Pkt::RaidBossKillNotify => {
                state.on_phase_transition(1);
                state.raid_clear = true;
                debug_print!("phase", &1);
            }
            Pkt::RaidResult => {
                state.on_phase_transition(0);
                debug_print!("phase", &0);
            }
            Pkt::RemoveObject => {
                if let Some(pkt) = parse_pkt(&data, PKTRemoveObject::new, "PKTRemoveObject") {
                    for upo in pkt.unpublished_objects {
                        status_tracker
                            .borrow_mut()
                            .remove_local_object(upo.object_id);
                    }
                }
            }
            Pkt::SkillCastNotify => {
                // identity skills
                // idk if i want to use this
                // only gets sent on certain identity casts
                // e.g. arcana cards, sorc ignite (only turning off)
                // let pkt = PKTSkillCastNotify::new(&data)?;
                // let mut entity = entity_tracker.get_source_entity(pkt.caster);
                // entity = entity_tracker.guess_is_player(entity, pkt.skill_id);
                // println!("skill cast notify {:?}", &entity);
                // parser.on_skill_start(entity, pkt.skill_id as i32, Utc::now().timestamp_millis());
            }
            Pkt::SkillStartNotify => {
                if let Some(pkt) = parse_pkt(&data, PKTSkillStartNotify::new, "PKTSkillStartNotify") {
                    let mut entity = entity_tracker.get_source_entity(pkt.source_id);
                    entity = entity_tracker.guess_is_player(entity, pkt.skill_id);
                    state.on_skill_start(entity, pkt.skill_id as i32, Utc::now().timestamp_millis());
                }
            }
            Pkt::SkillStageNotify => {
                // let pkt = PKTSkillStageNotify::new(&data);
            }
            Pkt::SkillDamageAbnormalMoveNotify => {
                if let Some(pkt) = parse_pkt(&data, PKTSkillDamageAbnormalMoveNotify::new, "PKTSkillDamageAbnormalMoveNotify") {
                    let owner = entity_tracker.get_source_entity(pkt.source_id);
                    let local_character_id = id_tracker
                        .borrow()
                        .get_local_character_id(entity_tracker.local_player_id);
                    for event in pkt.skill_damage_abnormal_move_events.iter() {
                        let target_entity =
                            entity_tracker.get_or_create_entity(event.skill_damage_event.target_id);
                        let source_entity = entity_tracker.get_or_create_entity(pkt.source_id);
                        let (se_on_source, se_on_target) = status_tracker
                            .borrow_mut()
                            .get_status_effects(&owner, &target_entity, local_character_id);
                        state.on_damage(
                            &owner,
                            &source_entity,
                            &target_entity,
                            event.skill_damage_event.damage,
                            pkt.skill_id as i32,
                            pkt.skill_effect_id as i32,
                            event.skill_damage_event.modifier as i32,
                            event.skill_damage_event.cur_hp,
                            event.skill_damage_event.max_hp,
                            se_on_source,
                            se_on_target,
                        );
                    }
                }
            }
            Pkt::SkillDamageNotify => {
                if let Some(pkt) = parse_pkt(&data, PKTSkillDamageNotify::new, "PktSkillDamageNotify") {
                    let owner = entity_tracker.get_source_entity(pkt.source_id);
                    let local_character_id = id_tracker
                        .borrow()
                        .get_local_character_id(entity_tracker.local_player_id);
                    for event in pkt.skill_damage_events.iter() {
                        let target_entity = entity_tracker.get_or_create_entity(event.target_id);
                        // source_entity is to determine battle item
                        let source_entity = entity_tracker.get_or_create_entity(pkt.source_id);
                        let (se_on_source, se_on_target) = status_tracker
                            .borrow_mut()
                            .get_status_effects(&owner, &target_entity, local_character_id);
                        state.on_damage(
                            &owner,
                            &source_entity,
                            &target_entity,
                            event.damage,
                            pkt.skill_id as i32,
                            pkt.skill_effect_id as i32,
                            event.modifier as i32,
                            event.cur_hp,
                            event.max_hp,
                            se_on_source,
                            se_on_target,
                        );
                    }
                }
            }
            Pkt::StatusEffectAddNotify => {
                if let Some(pkt) = parse_pkt(&data, PKTStatusEffectAddNotify::new, "PKTStatusEffectAddNotify") {
                    entity_tracker
                        .build_and_register_status_effect(&pkt.status_effect_data, pkt.object_id)
                }
            }
            Pkt::StatusEffectDurationNotify => {
                if let Some(pkt) = parse_pkt(&data, PKTStatusEffectDurationNotify::new, "PKTStatusEffectDurationNotify") {
                    status_tracker.borrow_mut().update_status_duration(
                        pkt.effect_instance_id,
                        pkt.target_id,
                        pkt.expiration_tick,
                        StatusEffectTargetType::Local,
                    );
                }
            }
            Pkt::StatusEffectRemoveNotify => {
                if let Some(pkt) = parse_pkt(&data, PKTStatusEffectRemoveNotify::new, "PKTStatusEffectRemoveNotify") {
                    status_tracker.borrow_mut().remove_status_effects(
                        pkt.object_id,
                        pkt.status_effect_ids,
                        StatusEffectTargetType::Local,
                    );
                }
            }
            Pkt::TriggerBossBattleStatus => {
                if state.encounter.fight_start == 0 || state.encounter.current_boss_name.is_empty() {
                    state.on_phase_transition(3);
                    debug_print!("phase", &3);
                } else {
                    state.on_phase_transition(2);
                    debug_print!("phase", &2);
                }
            }
            Pkt::TriggerStartNotify => {
                if let Some(pkt) = parse_pkt(&data, PKTTriggerStartNotify::new, "PKTTriggerStartNotify") {
                    match pkt.trigger_signal_type {
                        57 | 59 | 61 | 63 | 74 | 76 => {
                            state.raid_clear = true;
                            debug_print!("phase", &"clear".to_string())
                        }
                        58 | 60 | 62 | 64 | 75 | 77 => {
                            state.raid_clear = false;
                            debug_print!("phase", &"wipe".to_string())
                        }
                        _ => {}
                    }
                }
            }
            Pkt::ZoneObjectUnpublishNotify => {
                if let Some(pkt) = parse_pkt(&data, PKTZoneObjectUnpublishNotify::new, "PKTZoneObjectUnpublishNotify") {
                    status_tracker
                        .borrow_mut()
                        .remove_local_object(pkt.object_id);
                }
            }
            Pkt::StatusEffectSyncDataNotify => {
                // let pkt = PKTStatusEffectSyncDataNotify::new(&data);
                // shields
            }
            Pkt::TroopMemberUpdateMinNotify => {
                // let pkt = PKTTroopMemberUpdateMinNotify::new(&data);
                // shields
            }
            _ => {
                continue;
            }
        }

        if last_update.elapsed() >= duration || state.raid_end || state.boss_dead_update {
            let boss_dead = state.boss_dead_update;
            if state.boss_dead_update {
                debug_print!("boss_dead_update", &true);
                state.boss_dead_update = false;
            }
            let mut clone = state.encounter.clone();
            let window = window.clone();
            tokio::task::spawn(async move {
                if !clone.current_boss_name.is_empty() {
                    let current_boss = clone.entities.get(&clone.current_boss_name).cloned();
                    if let Some(mut current_boss) = current_boss {
                        if boss_dead {
                            current_boss.is_dead = true;
                            current_boss.current_hp = 0;
                        }
                        clone.current_boss = Some(current_boss);
                    } else {
                        clone.current_boss_name = String::new();
                    }
                }
                clone.entities.retain(|_, e| {
                    (e.entity_type == EntityType::PLAYER || e.entity_type == EntityType::ESTHER)
                        && e.damage_stats.damage_dealt > 0
                });
                if !clone.entities.is_empty() {
                    window
                        .emit("encounter-update", Some(clone))
                        .expect("failed to emit encounter-update");
                }
            });

            last_update = Instant::now();
        }

        if state.raid_end {
            state.soft_reset(true);
            state.raid_end = false;
            state.saved = false;
        }
    }

    Ok(())
}

fn parse_pkt<T, F>(data: &[u8], new_fn: F, pkt_name: &str) -> Option<T>
where
    F: FnOnce(&[u8]) -> Result<T, anyhow::Error>,
{
    match new_fn(data) {
        Ok(packet) => Some(packet),
        Err(e) => {
            warn!("Error parsing {}: {}", pkt_name, e);
            None
        }
    }
}