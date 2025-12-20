/// This file was auto-generated


function getUint16Array(data: DataView, offset: number, count: number): number[] {
  const values = [];
  for(let x=0; x<count; x++) { values.push(data.getUint16(offset + (2*x), true)); }
  return values;
}

export const ClearGui = 1;
export const UpdateGui = 2;
export const DrawGui = 3;
type GameMessageType = 1 | 2 | 3;

export class UpdateGuiMessageParams {
  data: DataView;
  constructor(data: DataView) {
    this.data = data;
  }
  index_bytes_offset() { return this.data.getUint32(0, true); }
  index_bytes_size() { return this.data.getUint32(4, true); }
  vertex_bytes_offset() { return this.data.getUint32(8, true); }
  vertex_bytes_size() { return this.data.getUint32(12, true); }
}
export class DrawGuiMessageParams {
  data: DataView;
  constructor(data: DataView) {
    this.data = data;
  }
  draw_count() { return this.data.getUint32(0, true); }
  index_bytes_offset() { return this.data.getUint32(4, true); }
  vertex_bytes_offset() { return this.data.getUint32(8, true); }
  image_texture() { return this.data.getUint32(12, true); }
  font_texture() { return this.data.getUint32(16, true); }
  scissor() { return getUint16Array(this.data, 20, 4, ); }
}
export class GameUpdateIndex {
  data: DataView;
  constructor(data: DataView) {
    this.data = data;
  }
  messages_count() { return this.data.getUint32(0, true); }
  messages_size() { return this.data.getUint32(4, true); }
  messages_ptr() { return this.data.getUint32(8, true); }
  data_ptr() { return this.data.getUint32(12, true); }
}
export class GameUpdateMessage {
  ty: GameMessageType;
  params: DataView;
  constructor(ty: GameMessageType, params: DataView) {
    this.ty = ty;
    this.params = params;
  }
  update_gui() {
    return new UpdateGuiMessageParams(this.params);
  }
  draw_gui() {
    return new DrawGuiMessageParams(this.params);
  }
}
export class GameUpdatesApi {
  buffer: ArrayBuffer;
  messages_count: number;
  messages_size: number;
  messages_ptr: number;
  data_ptr: number;
  constructor(buffer: ArrayBuffer, output_index_ptr: number) {
    const index = new GameUpdateIndex(new DataView(buffer, output_index_ptr, 16));
    this.buffer = buffer;
    this.messages_count = index.messages_count();
    this.messages_size = index.messages_size();
    this.messages_ptr = index.messages_ptr();
    this.data_ptr = index.data_ptr();
  }
  get_message(index: number): GameUpdateMessage | null {
    if (index >= this.messages_count) {
      console.error(`Tried to read message beyond total message count ${index} >= ${this.messages_count}`);
      return null;
    }
    const message_ptr = this.messages_ptr + (index * 32);
    const ty = new DataView(this.buffer, message_ptr, 4).getUint32(0, true);
    if (ty < ClearGui || ty > DrawGui) {
      console.error(`Received unknown message type: ${ty}`);
      return null;
    }
    const params = new DataView(this.buffer, message_ptr+4, 28)
    return new GameUpdateMessage(ty as GameMessageType, params);

  }
  get_data(offset: number, size: number): Uint8Array {
    return new Uint8Array(this.buffer, this.data_ptr+offset, size);
  }
}
