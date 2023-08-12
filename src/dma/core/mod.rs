pub mod offsets;
pub mod csgo;

use memflow::prelude::v1::*;

use crate::structs::Connector;

use self::offsets::Offsets;

pub struct CheatCtx {
    pub process: IntoProcessInstanceArcBox<'static>,
    pub client_module: ModuleInfo,
    pub engine_module: ModuleInfo,
    pub offsets: Offsets,
}

impl CheatCtx {
    pub fn setup(connector: Connector, pcileech_device: String) -> anyhow::Result<CheatCtx> {
        let offsets = offsets::Offsets::new();
        let inventory = Inventory::scan();

        let os = { 
            if connector == Connector::Pcileech {
                let args = Args::new()
                    .insert("device", &pcileech_device);

                let connector_args = ConnectorArgs::new(None, args, None);                

                inventory.builder()
                    .connector(&connector.to_string())
                    .args(connector_args)
                    .os("win32")
                    .build()?
            } else {
                inventory.builder()
                .connector(&connector.to_string())
                .os("win32")
                .build()?
            }
        };

        let mut process = os.into_process_by_name("csgo.exe")?;

        let client_module = process.module_by_name("client.dll")?;

        let engine_module = process.module_by_name("engine.dll")?;

        let ctx = Self {
            process,
            client_module,
            engine_module,
            offsets,
        };

        Ok(ctx)
    }
}
