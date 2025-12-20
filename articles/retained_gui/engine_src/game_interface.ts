import { set_last_error } from "./error";
import type { EngineAssets } from "./assets";
import type { GameClient, GameClientInit, InitOutput } from "../build/retained_gui";
import { GameUpdatesApi, GameUpdateMessage } from "./game_interface_api";
export { ClearGui, UpdateGui, DrawGui } from "./game_interface_api";
export type { UpdateGuiMessageParams, DrawGuiMessageParams } from "./game_interface_api";

const GAME_SRC_PATH = "./retained_gui.js";

interface GameModule {
    GameClientInit: typeof GameClientInit;
    GameClient: typeof GameClient;
    default(): Promise<InitOutput>;
    save(client: GameClient): Uint8Array;
    load(bytes: Uint8Array): GameClient;
}

export interface GameInterfaceStartupParams {
    screen_width: number,
    screen_height: number,
}

export class GameUpdates {
    api: GameUpdatesApi;

    constructor(buffer: ArrayBuffer, output_index_ptr: number) {
        this.api = new GameUpdatesApi(buffer, output_index_ptr);
    }

    message_count(): number {
        return this.api.messages_count;
    }

    get_message(index: number): GameUpdateMessage | null {
        return this.api.get_message(index);
    }

    get_data(offset: number, size: number): Uint8Array {
        return this.api.get_data(offset, size);
    }
}


export class GameInterface {
    instance: GameClient;
    module: GameModule;
    reload_count: number = 0;
    memory: WebAssembly.Memory;

    free() {
        if (this.instance) { this.instance.free(); }
    }

    // @ts-ignore
    async init(): Promise<boolean> {
        this.module = await import(GAME_SRC_PATH)
            .catch((e) => { set_last_error(`Failed to load the game client`); return null; });
    
        if (!this.module) {
            return false;
        }
    
        const initOutput = await this.module.default();
        this.memory = initOutput.memory;

        return true;
    }

    start(assets: EngineAssets, params: GameInterfaceStartupParams): boolean {
        const mod = this.module;

        const initial_data: GameClientInit = mod.GameClientInit.new();

        // Config
        initial_data.view_size(params.screen_width, params.screen_height);

        // Assets
        initial_data.set_assets_bundle(assets.bundle);
        for (const [csv_name, csv_value] of assets.csv.entries()) {
            initial_data.upload_text_asset(csv_name, csv_value);
        }

        for (const [font_name, font_value] of assets.fonts.entries()) {
            initial_data.upload_bin_asset(font_name, new Uint8Array(font_value.atlas_data));
        }

        const instance = mod.GameClient.initialize(initial_data);
        if (!instance) {
            set_last_error("Failed to start game client");
            return false;
        }

        this.instance = instance;

        return true
    }

    async reload(): Promise<boolean> {
        try {
            this.reload_count += 1;
    
            const saved = this.module.save(this.instance);
        
            this.module = await import(`${GAME_SRC_PATH}?v=${this.reload_count}`);
            const initOutput = await this.module.default();

            this.instance = this.module.load(saved);
            this.memory = initOutput.memory;

            return true;
        } catch (e) {
            console.log(e);
            return false;
        }
    }

    updates(): GameUpdates {
        const output_index_ptr = this.instance.updates_ptr();
        return new GameUpdates(this.memory.buffer, output_index_ptr);
    }

    resize(width: number, height: number) {
        this.instance.resize(width, height);
    }
}