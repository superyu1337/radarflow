use memflow::{prelude::MemoryView, types::Address};
use anyhow::Result;

use crate::{dma::core::CheatCtx, structs::Vec3};

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

    pub fn get_team(&self, ctx: &mut CheatCtx) -> Result<i32> {
        let offset = ctx.offsets.get_var("m_iTeamNum")?;
        Ok(ctx.process.read(self.0 + offset)?)
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

    pub fn get_class_id(&self, ctx: &mut CheatCtx) -> Result<u32> {
        let ptr1 = ctx.process.read_addr32(self.0 + 0x8)?; //IClientNetworkable vtable
        let ptr2 = ctx.process.read_addr32(ptr1 + 2 * 0x4)?; //3rd function in the vtable (GetClientClass)
        let ptr3 = ctx.process.read_addr32(ptr2 + 0x1)?; //pointer to the ClientClass struct out of the mov eax
        Ok(ctx.process.read(ptr3 + 0x14)?) //classid
    }

    pub fn is_player(&self, ctx: &mut CheatCtx) -> Result<bool> {
        let class_id = self.get_class_id(ctx)?;
        Ok(class_id == 40)
    }

    pub fn is_bomb(&self, ctx: &mut CheatCtx) -> Result<bool> {
        let class_id = self.get_class_id(ctx)?;
        Ok(class_id == 34)
    }

    pub fn is_bomb_planted(&self, ctx: &mut CheatCtx) -> Result<bool> {
        let class_id = self.get_class_id(ctx)?;
        Ok(class_id == 129)
    }

    pub fn is_player_alive(&self, ctx: &mut CheatCtx) -> Result<bool> {
        Ok(self.is_player(ctx)?
            && self.get_health(ctx)? != 0
        )
    }

    pub fn is_player_resources(&self, ctx: &mut CheatCtx) -> Result<bool> {
        let class_id = self.get_class_id(ctx)?;
        Ok(class_id == 41)
    }
}
