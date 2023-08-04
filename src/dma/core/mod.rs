mod offset_man;
pub mod csgo;

use memflow::prelude::v1::*;

use self::offset_man::Offsets;

pub struct CheatCtx {
    pub process: IntoProcessInstanceArcBox<'static>,
    pub client_module: ModuleInfo,
    pub engine_module: ModuleInfo,
    pub offsets: Offsets,
}

impl CheatCtx {
    pub fn setup() -> anyhow::Result<CheatCtx> {
        let inventory = Inventory::scan();

        let os = inventory.builder()
            .connector("qemu")
            .os("win32")
            .build()?;

        let mut process = os.into_process_by_name("csgo.exe")?;

        let client_module = process.module_by_name("client.dll")?;

        let engine_module = process.module_by_name("engine.dll")?;

        let offsets = offset_man::get_offsets()?;

        let ctx = Self {
            process,
            client_module,
            engine_module,
            offsets,
        };

        Ok(ctx)
    }
}
