import type { Channel } from '@tauri-apps/api/core'

declare global {
  interface Message {
    // type
    t: string,
    [key: string]: any,
  }
  
  type ConnectionEvent =
    | {
      event: 'connected',
      data: {
        name: string,
        version: string,
      },
    }
    | {
      event: 'message',
      data: { value: Message },
    }
    | {
      event: 'error',
      data: { message: 'string' },
    }
    | {
      event: 'disconnected'
    }
  
  interface VersionInfo {
    // the start of the external area
    baseAddress: number,
    // the size of the external area, in Slots
    size: number,
    // where an 8-byte build string is located
    buildAddress: number,
    // the expected 8-byte build string
    buildString: string,
  }

  // as passed by `actor_tree`
  interface ActorTreeNode {
    // type, eg. Sprite
    t: string,
    // name (or empty string for none)
    n: string,
    // hidden
    h: boolean,
    // children (or undefined for non-actorframes)
    c: ActorTreeNode[] | undefined,
    // text (or undefined for non-bitmaptexts)
    x: string,
  }
}

declare module '@tauri-apps/api/core' {
  function invoke(cmd: 'find_notitg_process'): Promise<[ number, string, VersionInfo ]>;
  function invoke(cmd: 'connect', data: {
    pid: number, baseAddress: number, size: number,
    channel: Channel<ConnectionEvent>
  }): Promise<null>;
  function invoke(cmd: 'send_message', data: { value: Message }): Promise<null>;
  function invoke(cmd: 'disconnect'): Promise<null>;
}