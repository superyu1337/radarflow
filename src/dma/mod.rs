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

    let mut c4_owner_id: i32 = 0;

    // For poll rate timing
    let target_interval = Duration::from_nanos(SECOND_AS_NANO / config.poll_rate() as u64);
    let mut last_iteration_time = Instant::now();
    let mut missmatch_count = 0;

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
            let local_team = local.get_team(&mut ctx)?;
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
                let pos = entity.get_pos(&mut ctx)?;
                let yaw = entity.get_yaw(&mut ctx)?;
                let is_dormant = entity.get_dormant(&mut ctx)?;
                let mut has_bomb = false;

                if entity.is_player_resources(&mut ctx)? {
                    let m_iplayerc4 = 0x165c;
                    c4_owner_id = ctx.process.read(entity.0 + m_iplayerc4)?;
                } else if entity.is_bomb(&mut ctx)? && c4_owner_id == 0 && !is_dormant {
                    player_data.push(
                        EntityData::Bomb(
                            BombData::new(
                                pos, false
                            )
                        )
                    )
                } else if entity.is_bomb_planted(&mut ctx)? {
                    if c4_owner_id != 0 {
                        log::debug!("Bomb is planted, but bomb owner id is: {}", c4_owner_id);
                    }
    
                    player_data.push(
                        EntityData::Bomb(
                            BombData::new(
                                pos, true
                            )
                        )
                    )
                } else if entity.is_player_alive(&mut ctx)? {
                    let mut player_type = None;
    
                    // Check if it's an enemy
                    if entity.get_team(&mut ctx)? != local_team { 
                        player_type = Some(PlayerType::Enemy);
                    }
                    
                    // Check if it's a teammate
                    if entity.get_team(&mut ctx)? == local_team {
                        player_type = Some(PlayerType::Team);
                    }
    
                    // the entity has the same ptr as our localplayer, that's us.
                    if entity.0 == local.0 {
                        player_type = Some(PlayerType::Local);
                    }
    
                    // Check if the entity has the c4 or not.
                    if c4_owner_id != 0 && entity.get_index(&mut ctx)? == c4_owner_id { 
                        
                        // Fixing the issue where CT are shown holding the bomb due to dormancy
                        if entity.get_team(&mut ctx)? == 2 {
                            has_bomb = true;
                        }
                    }
    
                    player_data.push(
                        EntityData::Player(
                            PlayerData::new(
                                pos, 
                                yaw,
                                player_type.unwrap_or_default(),
                                is_dormant,
                                has_bomb
                            )
                        )
                    );
                }
            }

            let mut data = data_lock.write().await;
            *data = RadarData::new(true, map_name, player_data, local_yaw)
        }

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

    Ok(())
}
