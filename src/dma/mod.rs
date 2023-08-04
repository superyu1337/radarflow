pub mod core;

use std::sync::Arc;

use anyhow::Result;
use tokio::{sync::RwLock, time::{Duration, Instant}};

use memflow::{os::process::Process, prelude::MemoryView};

use crate::{structs::{communication::{RadarData, EntityData, BombData, PlayerType, PlayerData}, Config}, dma::core::csgo};

const SECOND_AS_NANO: u64 = 1000*1000*1000;
static ONCE: std::sync::Once = std::sync::Once::new();

pub async fn run(config: Config, data_lock: Arc<RwLock<RadarData>>) -> Result<()> {
    let mut ctx = core::CheatCtx::setup()?;

    // Avoid printing warnings and other stuff before the initial prints are complete
    tokio::time::sleep(Duration::from_millis(500)).await;

    // For poll rate timing
    let should_time = config.poll_rate() != 0;

    let target_interval = Duration::from_nanos(SECOND_AS_NANO / config.poll_rate() as u64);
    let mut last_iteration_time = Instant::now();
    let mut missmatch_count = 0;

    // self explainatory
    let mut c4_owner_id: i32 = 0;

    loop {
        if ctx.process.state().is_dead() {
            break;
        }

        let clientstate = csgo::ClientState::get(&mut ctx)?;
        
        if !clientstate.is_ingame(&mut ctx)? {
            let mut data = data_lock.write().await;
            *data = RadarData::empty();
        } else {
            let map_name = clientstate.get_mapname(&mut ctx)?;

            let local = csgo::Entity::get_local(&mut ctx)?;
            let local_yaw = local.get_yaw(&mut ctx)?;
            let mut player_data = Vec::with_capacity(64);

            let mut address = ctx.client_module.base + ctx.offsets.get_sig("dwEntityList")?;

            while !address.is_null() {
                let entry = csgo::CEntInfo::read(&mut ctx, address)?;
                address = entry.next.into();
    
                if entry.next == entry.previous {
                    break;
                }
    
                if entry.entity_ptr == 0 {
                    continue;
                }
    
                let entity = csgo::Entity::from_ptr(entry.entity_ptr.into())?;

                if let Some(class_id) = entity.get_class_id(&mut ctx)? {
                    match class_id {
                        csgo::ClassID::CCSPlayer => {
                            if !entity.is_alive(&mut ctx)? { continue }

                            let pos = entity.get_pos(&mut ctx)?;
                            let yaw = entity.get_yaw(&mut ctx)?;
                            let is_dormant = entity.get_dormant(&mut ctx)?;
                            let mut has_bomb = false;

                            let player_type = {
                                match entity.get_player_type(&mut ctx, &local)? {
                                    Some(t) => {
                                        if t == PlayerType::Spectator { continue } else { t }
                                    },
                                    None => { continue },
                                }
                            };
            
                            // Check if the entity has the c4 or not.
                            if c4_owner_id != 0 && c4_owner_id == entity.get_index(&mut ctx)? { 
                                has_bomb = true;
                                
                                // Fixing the issue where CT are shown holding the bomb
                                // I believe this issue is caused by dormancy, but I'm not sure at all.
                                //if entity.get_team(&mut ctx)? == Some(TeamID::CT) {
                                    //has_bomb = true;
                                //}

                            }
            
                            player_data.push(
                                EntityData::Player(
                                    PlayerData::new(
                                        pos, 
                                        yaw,
                                        player_type,
                                        is_dormant,
                                        has_bomb
                                    )
                                )
                            );
                        },
                        csgo::ClassID::CCSPlayerResources => {
                            let m_iplayerc4 = 0x165c;
                            c4_owner_id = ctx.process.read(entity.0 + m_iplayerc4)?;
                        },
                        csgo::ClassID::CPlantedC4 => {
                            let pos = entity.get_pos(&mut ctx)?;
                            if c4_owner_id == 0 {
                                player_data.push(
                                    EntityData::Bomb(
                                        BombData::new(
                                            pos, true
                                        )
                                    )
                                )
                            }
                        },
                        csgo::ClassID::CWeaponC4 => {
                            let pos = entity.get_pos(&mut ctx)?;
                            let is_dormant = entity.get_dormant(&mut ctx)?;

                            if c4_owner_id == 0 && !is_dormant {
                                player_data.push(
                                    EntityData::Bomb(
                                        BombData::new(
                                            pos, false
                                        )
                                    )
                                )
                            }

                        },
                    }
                }
            }

            let mut data = data_lock.write().await;
            *data = RadarData::new(true, map_name, player_data, local_yaw)
        }

        if should_time {
            let elapsed = last_iteration_time.elapsed();

            let remaining = match target_interval.checked_sub(elapsed) {
                Some(t) => t,
                None => {
                    if missmatch_count >= 25 {
                        ONCE.call_once(|| {
                            log::warn!("Remaining time till target interval was negative more than 25 times");
                            log::warn!("You should decrease your poll rate.");
                            log::warn!("elapsed: {}ns", elapsed.as_nanos());
                            log::warn!("target: {}ns", target_interval.as_nanos());
                        });
                    } else {
                        missmatch_count += 1;
                    }
                    Duration::from_nanos(0)
                },
            };
    
            #[cfg(target_os = "linux")]
            tokio_timerfd::sleep(remaining).await?;
    
            #[cfg(not(target_os = "linux"))]
            tokio::time::sleep(remaining).await;
    
            log::trace!("polling at {:.2}Hz", SECOND_AS_NANO as f64 / last_iteration_time.elapsed().as_nanos() as f64);
            log::trace!("elapsed: {}", elapsed.as_nanos());
            log::trace!("target: {}", target_interval.as_nanos());
            log::trace!("missmatch count: {}", missmatch_count);
    
            last_iteration_time = Instant::now();
        }
    }

    Ok(())
}
