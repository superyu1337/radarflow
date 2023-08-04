use enum_primitive_derive::Primitive;
use memflow::{prelude::MemoryView, types::Address};
use anyhow::Result;
use num_traits::FromPrimitive;

use crate::{dma::core::CheatCtx, structs::{Vec3, communication::PlayerType}};

#[repr(C)]
#[derive(Debug)]
pub struct CEntInfo {   
    pub entity_ptr: u32, 
    pub serial_number: u32,
    pub previous: u32,
    pub next: u32,
}

impl CEntInfo {
    pub fn read(ctx: &mut CheatCtx, address: Address) -> Result<CEntInfo> {
        Ok(ctx.process.read(address)?)
    }
}

unsafe impl dataview::Pod for CEntInfo {}

#[repr(i32)]
#[derive(Debug, Eq, PartialEq, Primitive)]
pub enum ClassID {
    CCSPlayer = 40,
    CCSPlayerResources = 41,
    CPlantedC4 = 129,
    CWeaponC4 = 34
}

#[repr(i32)]
#[derive(Debug, Eq, PartialEq, Primitive)]
pub enum TeamID {
    Spectator = 1,
    T = 2,
    CT = 3
}

pub struct Entity(pub Address);

impl Entity {
    pub fn from_ptr(ptr: Address) -> Result<Entity> {
        Ok(Entity(ptr))
    }

    pub fn get_local(ctx: &mut CheatCtx) -> Result<Entity> {
        let offset = ctx.offsets.get_sig("dwLocalPlayer")?;
        let ptr = ctx.process.read_addr32(ctx.client_module.base + offset)?;

        Ok(Entity(ptr))
    }

    pub fn get_index(&self, ctx: &mut CheatCtx) -> Result<i32> {
        Ok(ctx.process.read(self.0 + 0x64)?)
    }

    pub fn get_health(&self, ctx: &mut CheatCtx) -> Result<i32> {
        let offset = ctx.offsets.get_var("m_iHealth")?;
        Ok(ctx.process.read(self.0 + offset)?)
    }

    pub fn get_team(&self, ctx: &mut CheatCtx) -> Result<Option<TeamID>> {
        let offset = ctx.offsets.get_var("m_iTeamNum")?;
        let team: i32 = ctx.process.read(self.0 + offset)?;
        Ok(TeamID::from_i32(team))
    }

    pub fn get_dormant(&self, ctx: &mut CheatCtx) -> Result<bool> {
        let offset = ctx.offsets.get_sig("m_bDormant")?;
        let data: u8 = ctx.process.read(self.0 + offset)?;

        Ok(data != 0)
    }

    pub fn get_pos(&self, ctx: &mut CheatCtx) -> Result<Vec3> {
        let offset = ctx.offsets.get_var("m_vecOrigin")?;
        Ok(ctx.process.read(self.0 + offset)?)
    }

    pub fn get_yaw(&self, ctx: &mut CheatCtx) -> Result<f32> {
        let offset = ctx.offsets.get_var("m_angEyeAnglesY")?;
        Ok(ctx.process.read(self.0 + offset)?)
    }


    /// Gets ClassID of the entity as enum
    /// Returns None if it can't match the enum
    pub fn get_class_id(&self, ctx: &mut CheatCtx) -> Result<Option<ClassID>> {
        let ptr1 = ctx.process.read_addr32(self.0 + 0x8)?; //IClientNetworkable vtable
        let ptr2 = ctx.process.read_addr32(ptr1 + 2 * 0x4)?; //3rd function in the vtable (GetClientClass)
        let ptr3 = ctx.process.read_addr32(ptr2 + 0x1)?; //pointer to the ClientClass struct out of the mov eax
        let class_id: i32 = ctx.process.read(ptr3 + 0x14)?; //classid

        Ok(ClassID::from_i32(class_id))
    }

    pub fn get_player_type(&self, ctx: &mut CheatCtx, local: &Entity) -> Result<Option<PlayerType>> {
        if self.0 == local.0 {
            return Ok(Some(PlayerType::Local))
        }

        let team = {
            match self.get_team(ctx)? {
                Some(t) => t,
                None => { return Ok(None) },
            }
        };
        
        let local_team = {
            match local.get_team(ctx)? {
                Some(t) => t,
                None => { return Ok(None) },
            }
        };



        let player_type = {
            if team == TeamID::Spectator { 
                PlayerType::Spectator 
            } else if team != local_team {
                PlayerType::Enemy
            } else {
                PlayerType::Team
            }
        };

        Ok(Some(player_type))
    }

    /// Same as ::get_health > 0
    pub fn is_alive(&self, ctx: &mut CheatCtx) -> Result<bool> {
        Ok(self.get_health(ctx)? > 0)
    }
}
