/// Interface between the wasm client and the engine
import { set_last_error } from "./error";
import { EngineAssets } from "./assets";
import { GameClient, GameClientInit } from "../build/navmesh_pathfinding_demo";

const GAME_SRC_PATH = "/articles/navmesh_pathfinding/navmesh_pathfinding_demo.js";

export class GameUpdates {
    protocol: any;
    buffer: ArrayBuffer;
    messages_count: number;
    messages_size: number;
    messages_ptr: number;
    base_data_ptr: number;

    constructor(protocol: any, buffer: ArrayBuffer, output_index_ptr: number) {
        this.protocol = protocol;
        this.buffer = buffer;

        const index = new this.protocol.OutputIndex(buffer, output_index_ptr);
        this.messages_count = index.messages_count();
        this.messages_size = index.messages_size();
        this.messages_ptr = index.messages_ptr();
        this.base_data_ptr = index.data_ptr();
    }

    get_message(index: number): any {
        const offset = this.messages_ptr + (index * this.messages_size);
        return new this.protocol.OutputMessage(this.buffer, offset);
    }

    get_data(offset: number, size: number) {
        return this.buffer.slice(this.base_data_ptr + offset, this.base_data_ptr + offset + size);
    }
}

export class GameInterface {
    instance: GameClient;
    module: any;
    protocol: any = null;
    reload_count: number = 0;

    // @ts-ignore
    async init(): Promise<boolean> {
        this.module = await import(GAME_SRC_PATH)
            .catch((e) => { set_last_error(`Failed to load the game client`); return null; });
    
        if (!this.module) {
            return false;
        }
    
        await this.module.default();

        // Protocol is a javascript module generated that the game client that informs the engine of the client data layout
        const proto = this.module.protocol();
        const blob = new Blob([proto], { type: 'application/javascript' });
        const moduleUrl = URL.createObjectURL(blob);
        this.protocol = await import(moduleUrl);
    
        return true;
    }

    start(assets: EngineAssets): boolean {
        const mod = this.module;

        const initial_data: GameClientInit = mod.GameClientInit.new();
        initial_data.set_assets_bundle(assets.bundle);
        for (const [csv_name, csv_value] of assets.csv.entries()) {
            initial_data.upload_text_asset(csv_name, csv_value);
        } 

        this.instance = mod.GameClient.initialize(initial_data);
        if (!this.instance) {
            set_last_error("Failed to start game client");
            return false;
        }

        return true
    }

    async reload(): Promise<boolean> {
        try {
            this.reload_count += 1;
    
            const saved = this.module.save(this.instance);
        
            this.module = await import(`${GAME_SRC_PATH}?v=${this.reload_count}`);
            await this.module.default();

            this.instance = this.module.load(saved);

            return true;
        } catch (e) {
            console.log(e);
            return false;
        }
    }

    updates(): GameUpdates {
        const buffer = this.get_memory();
        const output_index_ptr = this.instance.updates_ptr();
        return new GameUpdates(this.protocol, buffer, output_index_ptr);
    }

    private get_memory(): ArrayBuffer {
        if (this.module) {
            // If the module was already initialized, this only returns the wasm memory
            return this.module.initSync().memory.buffer;
        } else {
            throw "Client module is not loaded";
        }
    }
}

