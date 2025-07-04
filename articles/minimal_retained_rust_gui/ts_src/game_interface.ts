import { set_last_error } from "./error";
import { EngineAssets } from "./assets";
import { GameClient, GameClientInit } from "../build/minimal_retained_rust_gui_demo";

const GAME_SRC_PATH = "/articles/minimal_retained_rust_gui/minimal_retained_rust_gui_demo.js";

export interface GameInterfaceStartupParams {
    max_texture_size: number,
    screen_width: number,
    screen_height: number,
}


export class GameInterface {
    instance: GameClient;
    module: any;
    reload_count: number = 0;

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
    
        await this.module.default();

        return true;
    }

    start(assets: EngineAssets, params: GameInterfaceStartupParams): boolean {
        const mod = this.module;

        const initial_data: GameClientInit = mod.GameClientInit.new();

        // Config
        initial_data.max_texture_size(params.max_texture_size);
        initial_data.view_size(params.screen_width, params.screen_height);

        this.instance = mod.GameClient.initialize(initial_data);
        if (!this.instance) {
            set_last_error("Failed to start game client");
            return false;
        }

        return true
    }

    async reload(): Promise<boolean> {
        return true;
    }
}