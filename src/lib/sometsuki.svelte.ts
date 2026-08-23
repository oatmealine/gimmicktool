import { Channel, invoke } from '@tauri-apps/api/core';

export const connection = $state({
  state: 'closed' as 'closed' | 'connecting' | 'open',
  desc: '',
  error: null as string | null,
});

let lastError: string | null = null;

export function setupListeners(channel: Channel<ConnectionEvent>) {
  channel.onmessage = event => {
    if (event.event === 'connected') {
      connection.state = 'open';
      console.log(`connected to ${event.data.name} v${event.data.version}`);

      document.dispatchEvent(new Event('sometsukiconnected'));
    } else if (event.event === 'error') {
      console.error(event.data.message);
      lastError = event.data.message;
    } else if (event.event === 'message') {
      const msg = event.data.value;

      const ev = new CustomEvent('sometsukionmessage', {
        detail: msg,
        cancelable: true,
      });
      const uncaught = document.dispatchEvent(ev);

      if (uncaught) {
        console.warn(`unknown message type ${msg.t}`);
      }
    } else if (event.event === 'disconnected') {
      console.log('disconnected :(');
      connection.state = 'closed';
      connection.error = lastError;
    }
  };
}

export function sendMessage(value: Message): Promise<null> {
  return invoke('send_message', { value });
}

export async function connect() {
  connection.state = 'connecting';

  try {
    connection.desc = 'finding process...';
    const [ pid, ver, { baseAddress, size } ] = await invoke('find_notitg_process');
    connection.desc = 'attempting connection...';
    console.log(`found process ${pid} running ${ver}: `, baseAddress, size);
    console.log('attempting connection');
    const channel = new Channel();
    setupListeners(channel);
    await invoke('connect', { pid, baseAddress, size, channel });
    connection.desc = 'waiting for response...';
    console.log('yay!');
  } catch(err) {
    connection.error = (err as string).toString();
    connection.state = 'closed';
  }
}

export async function disconnect() {
  lastError = null;
  await invoke('disconnect');
}