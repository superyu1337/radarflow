// Common
canvas = null
ctx = null

// radarflow specific
image = null
map = null
mapName = null
loaded = false
entityData = null
update = false

// networking
websocket = null
websocketAddr = `ws://${window.location.hostname}:8001`

function render() {
    if (update) {
        if (loaded) {
            update = false
            drawImage()

            if (entityData != null) {
                entityData.forEach(function(data) {
                    if (data.Bomb !== undefined) {
                        drawBomb(data.Bomb.pos.x, data.Bomb.pos.y, data.Bomb.isPlanted)
                    } else {
                        let fillStyle = "#42f566"

                        switch (data.Player.playerType) {
                            case "Team":
                                fillStyle = "#4287f5"
                                break;

                            case "Enemy":
                                fillStyle = "#db1d1d"
                                break;
                        }

                        drawEntity(
                            data.Player.pos.x, 
                            data.Player.pos.y, 
                            fillStyle, 
                            data.Player.isDormant
                        )
                    }
                });
            }
        } else {
            if (websocket != null) {
                ctx.font = "100px Arial";
                ctx.textAlign = "center"
                ctx.textBaseline = "middle"
                ctx.fillStyle = "#d1d1d1"
                ctx.fillText("Not on a server", 1024/2, 1024/2); 
            } else {
                ctx.font = "100px Arial";
                ctx.textAlign = "center"
                ctx.fillStyle = "#d1d1d1"
                ctx.fillText("Disconnected", 1024/2, 1024/2); 
            }
        }
    }

    if (websocket != null) {
        websocket.send("requestInfo");
    }
}

function drawImage() {
    if (image != null && canvas != null)
        ctx.drawImage(image, 0, 0, image.width, image.height, 0, 0, canvas.width, canvas.height)
}

function drawBomb(x, y, planted) {
    if (map == null)
        return

    let offset_x = x - map.pos_x;
    let offset_y = y - map.pos_y;

    offset_x /= map.scale;
    offset_y /= -map.scale;

    ctx.beginPath();
    ctx.arc(offset_x, offset_y, 5, 0, 2 * Math.PI);
    ctx.fillStyle = "#dbb81d";
    ctx.fill();
    ctx.closePath();

    if (planted) {
        ctx.strokeStyle = "#db1d1d"
        ctx.lineWidth = 2;
        ctx.stroke()
    }
}

function drawEntity(x, y, fillStyle, dormant, hasBomb) {
    if (map == null)
        return

    let offset_x = x - map.pos_x;
    let offset_y = y - map.pos_y;

    offset_x /= map.scale;
    offset_y /= -map.scale;

    if (dormant) {
        ctx.font = "20px Arial";
        ctx.textAlign = "center"
        ctx.fillStyle = fillStyle
        ctx.fillText("?", offset_x, offset_y); 
    } else {
        ctx.beginPath();
        ctx.arc(offset_x, offset_y, 7, 0, 2 * Math.PI);
        ctx.fillStyle = fillStyle;
        ctx.fill();
        ctx.closePath();
    }

    if (hasBomb) {
        ctx.strokeStyle = "#dbb81d"
        ctx.lineWidth = 2;
        ctx.stroke()
    }
}

function loadMap(mapName) {
    console.log(`[radarflow] loading map ${mapName}`)
    loaded = true;
    const map_img = new Image();
    map_img.src = `assets/image/${mapName}.jpg`;

    fetch(`assets/json/${mapName}.json`)
        .then(response => response.json())
        .then(data => {
            map = data;
        })
        .catch(error => {
            console.error('Error loading JSON file:', error);
        });

    map_img.onload = () => {
        image = map_img;
        update = true;
    };
}

function unloadMap() {
    console.log("[radarflow] unloading map")
    ctx.clearRect(0, 0, canvas.width, canvas.height)
    map = null
    mapName = null
    loaded = false,
    update = true
    requestAnimationFrame(render);
}

function connect() {
    if (websocket == null) {
        let socket = new WebSocket(websocketAddr)

        socket.onopen = () => {
            console.log("[radarflow] Connection established")
            websocket.send("requestInfo");
        };
        
        socket.onmessage = (event) => {
            if (event.data == "error") {
                console.log("[radarflow] Server had an unknown error")
            } else {
                let data = JSON.parse(event.data);

                if (data.ingame == false) {
                    mapName = null
                    entityData = null

                    if (loaded)
                        unloadMap()
                } else {
                    if (!loaded) {
                        mapName = data.mapName
                        entityData = data.entityData
                        loadMap(mapName)
                    } else {
                        entityData = data.entityData
                    }
                }

                update = true
                requestAnimationFrame(render);
            }
        };
        
        socket.onclose = (event) => {
            if (event.wasClean) {
                console.log("[radarflow] connection closed");
            } else {
                console.log("[radarflow] connection died");
            }

            playerData = null
            websocket = null
            unloadMap()

            setTimeout(function() {
                connect();
            }, 1000);
        };
    
        socket.onerror = (error) => {
            console.log(`[radarflow] websocket error: ${error}`);
        };

        websocket = socket;
    } else {
        setTimeout(() => {
            connect();
        }, 1000);
    }
}

addEventListener("DOMContentLoaded", (e) => {
    canvas = document.getElementById('canvas');
    canvas.width = 1024;
    canvas.height = 1024;
    ctx = canvas.getContext('2d');

    console.log(`[radarflow] connecting to ${websocketAddr}`)
    connect()
});