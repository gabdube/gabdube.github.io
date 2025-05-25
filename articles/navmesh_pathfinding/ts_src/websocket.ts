const WEBSOCKET_HOST = "localhost:8001"
const VALID_MESSAGE_NAMES: string[] = ["FILE_CHANGED"];

export class WebSocketMessage {
    name: string;
    data: string;
    constructor(name: string, data: string) {
        this.name = name;
        this.data = data;
    }
}

export class EngineWebSocket {
    socket: WebSocket | null = null;
    messages: WebSocketMessage[] = [];
    messages_count: number = 0;
    opened: boolean = false;

    async open() {
        let socket: WebSocket;
        try {
            socket = new WebSocket("ws://"+WEBSOCKET_HOST);
            socket.binaryType = "arraybuffer";
        
            socket.addEventListener("open", (event) => {
                this.opened = true;
            });

            socket.addEventListener('error', (event) => {
                console.log("Error while opening websocket connection!");
                console.log(event);
                this.opened = false;
                this.socket = null;
            });
        
            socket.addEventListener("message", (event: MessageEvent) => {
                if (typeof event.data === "string") {
                    on_text_message(this, JSON.parse(event.data))
                } else {
                    on_bin_message(event.data);
                }
            });

            socket.addEventListener("close", (event) => {
                this.opened = false;
            })

        } catch {
            // No dev server
        }
    }
}

function on_text_message(ws: EngineWebSocket, message: any) {
    if (message.name && message.data) {
        if (!VALID_MESSAGE_NAMES.includes(message.name)) {
            console.error("Unknown message:", message);
            return;
        }

        let ws_message = new WebSocketMessage(message.name, message.data);
        ws.messages[ws.messages_count] = ws_message;
        ws.messages_count += 1;
    } else {
        console.error("Unknown message:", message);
    }
}

function on_bin_message(data: ArrayBuffer) {
}
