# radarflow
A Web radar for CS:GO using [memflow](https://github.com/memflow/memflow)

## How can I run this?
First, you need to set up a virtual machine on linux using qemu.  
As of now, memflow's pcileech connector is not supported.

How to set up a VM on linux is way out of scope for this. You can find plenty of information online on how to do it.

After you have set up your VM, you can clone this repository on your host:  
`git clone https://github.com/superyu1337/radarflow.git`

Now you can run radarflow:  
`cargo run --release`


## Configuration
radarflow can read a configuration file named `Config.toml` from the directory it got invoked at.  
If you run with cargo, that file is simply going to be at the root of the cloned repository.  
If it can't find the configuration file, the internal defaults will be used instead.

The configuration file uses the [TOML](https://toml.io/en/) format.

### Default configuration
```toml
# Config.toml
web_port = 8000
web_path = "./web"
poll_rate = 60
```

### Configuration options
`web_port` (default: `8000`)  
Port to run the http webserver on

`web_path` (default: `"./web"`)  
Path to the files that the http webserver should serve.  
By default radarflow will serve a barebones client implementation from `"./web"`

`poll_rate` (default: `60`)  
How often per second the radarflow dma thread should poll the game for data

#### Important note
`poll_rate` is rather inaccurate on non-linux hosts. 
I'm looking to resolve this issue soon.

## Logging
This project uses the [`env_logger`](https://docs.rs/env_logger/0.10.0/env_logger/) crate.  
The environment variable is `RADARFLOW_LOG`.  
Refer to it's documentation on how to configure logging using environment variables.

## Detection Status
VAC: ✅ (Undetected)  
FaceIt: ❓ (Unknown, could work with proper spoofing)  
ESEA: ❓ (Unknown, could work with proper spoofing)  
