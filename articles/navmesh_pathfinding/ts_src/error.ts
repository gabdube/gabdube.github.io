let LAST_ERROR: Error | null = null;

export class Error {
    message: string;
    traceback: string | null;

    constructor(msg: string, tb: string | null) {
        this.message = msg;
        this.traceback = tb;
    }
}

export function set_last_error(msg: string, tb?: string) {
    LAST_ERROR = new Error(msg, tb || null);
    console.log(LAST_ERROR);
}

export function get_last_error(): Error | null {
    return LAST_ERROR;
}

