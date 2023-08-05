// Colors
const localColor = "#109856"
const teamColor = "#68a3e5"
const enemyColor = "#ec040b"
const bombColor = "#eda338"
const textColor = "#d1d1d1"

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

// Util function
function degreesToRadians (degrees) {
    return degrees * (Math.PI/180);     
}

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
                        let fillStyle = localColor

                        switch (data.Player.playerType) {
                            case "Team":
                                fillStyle = teamColor
                                break;

                            case "Enemy":
                                fillStyle = enemyColor
                                break;
                        }

                        drawEntity(
                            data.Player.pos.x, 
                            data.Player.pos.y, 
                            fillStyle, 
                            data.Player.isDormant,
                            data.Player.hasBomb,
                            data.Player.yaw
                        )
                    }
                });
            }
        } else {
            if (websocket != null) {
                ctx.font = "100px Arial";
                ctx.textAlign = "center"
                ctx.textBaseline = "middle"
                ctx.fillStyle = textColor
                ctx.fillText("Not on a server", 1024/2, 1024/2); 
            } else {
                ctx.font = "100px Arial";
                ctx.textAlign = "center"
                ctx.fillStyle = textColor
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
    ctx.fillStyle = bombColor;
    ctx.fill();
    ctx.closePath();

    if (planted && ((new Date().getTime() / 1000) % 1) > 0.5) {
        ctx.strokeStyle = enemyColor
        ctx.lineWidth = 1;
        ctx.stroke()
    }
}

function drawEntity(x, y, fillStyle, dormant, hasBomb, yaw) {
    if (map == null)
        return

    const circleRadius = 7

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
        // Draw circle

        ctx.beginPath();
        ctx.arc(offset_x, offset_y, circleRadius, 0, 2 * Math.PI);
        ctx.fillStyle = fillStyle;
        ctx.fill();

        if (hasBomb) {
            ctx.beginPath();
            ctx.arc(offset_x, offset_y, circleRadius / 2, 0, 2 * Math.PI);
            ctx.fillStyle = "#dbb81d";
            ctx.fill();
        }


        ctx.closePath();

        // Calculate arrowhead points
        const distance = circleRadius + 2
        const radius = distance + 5;
        const arrowWidth = 35;

        const arrowHeadX = offset_x + radius * Math.cos(yaw * (Math.PI / 180))
        const arrowHeadY = offset_y - radius * Math.sin(yaw * (Math.PI / 180))

        const arrowCornerX1 = offset_x + distance * Math.cos((yaw - arrowWidth) * (Math.PI / 180))
        const arrowCornerY1 = offset_y - distance * Math.sin((yaw - arrowWidth) * (Math.PI / 180))

        const arrowCornerX2 = offset_x + distance * Math.cos((yaw + arrowWidth) * (Math.PI / 180))
        const arrowCornerY2 = offset_y - distance * Math.sin((yaw + arrowWidth) * (Math.PI / 180))


        const cicleYaw = 90-yaw
        const startAngle = degreesToRadians(cicleYaw-arrowWidth)-Math.PI/2
        const endAngle = degreesToRadians(cicleYaw+arrowWidth)-Math.PI/2

        // Draw arrow

        /// Backside of the arrow
        ctx.beginPath();
        ctx.arc(offset_x, offset_y, distance, startAngle, endAngle)

        /// Draw from corners to arrowhead
        ctx.lineTo(arrowCornerX1, arrowCornerY1);
        ctx.lineTo(arrowHeadX, arrowHeadY);
        ctx.lineTo(arrowCornerX2, arrowCornerY2);
        ctx.closePath()

        ctx.fillStyle = 'white'
        ctx.fill();
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