use memflow::{prelude::MemoryView, types::Address};

use anyhow::Result;

use crate::dma::core::CheatCtx;

pub struct ClientState {
    pub ptr: Address,
}

impl ClientState {
    pub fn get(ctx: &mut CheatCtx) -> Result<ClientState> {
        let offset = ctx.offsets.get_sig("dwClientState")?;
        let ptr = ctx.process.read_addr32(ctx.engine_module.base + offset)?;

        Ok(ClientState { ptr })
    }

    pub fn get_mapname(&self, ctx: &mut CheatCtx) -> Result<String> {
        let offset = ctx.offsets.get_sig("dwClientState_Map")?;
        Ok(ctx.process.read_char_string_n(self.ptr + offset, 64)?)
    }

    pub fn is_ingame(&self, ctx: &mut CheatCtx) -> Result<bool> {
        let offset = ctx.offsets.get_sig("dwClientState_State")?;
        let state: i32 = ctx.process.read(self.ptr + offset)?;
        Ok(state == 6)
    }
}
