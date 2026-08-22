const { invoke, Channel } = window.__TAURI__.core;

let helloPromise;

function onDisconnected() {
  const connectButton = document.querySelector('#connect-btn');
  const error = document.querySelector('#connect-error');

  error.innerText = 'disconnected';
  error.style.display = '';
  
  connectButton.removeAttribute('disabled');
  connectButton.innerText = 'connect';
  connectButton.style.display = '';
}

function setupListeners(channel) {
  channel.onmessage = event => {
    if (event.event === 'connected') {
      console.log(`connected to ${event.data.name} v${event.data.version}`);
      helloPromise();
    } else if (event.event === 'error') {
      console.error(event.data.message);
    } else if (event.event === 'message') {
      const msg = event.data.value;
      switch (msg.t) {
        case 'actor_tree':
          renderTree(msg.d);
          break;
        default:
          console.warn(`unknown message type ${msg.t}`);
          break;
      }
    } else if (event.event === 'disconnected') {
      console.log(':(');
      onDisconnected();
    }
  };
}

function sendMessage(value) {
  return invoke('send_message', { value });
}

async function connect() {
  const [ pid, ver, { baseAddress, size } ] = await invoke('find_notitg_process');
  console.log(`found process ${pid} running ${ver}: `, baseAddress, size);
  console.log('attempting connection');
  const channel = new Channel();
  setupListeners(channel);
  const promise = new Promise(resolve => helloPromise = resolve);
  await invoke('connect', { pid, baseAddress, size, channel });
  console.log('yay!');
  await promise;

  await sendMessage({ t: 'poll_actor_tree' });
}

window.addEventListener('DOMContentLoaded', async () => {  
  // clean up old connection if it exists
  await invoke('disconnect');

  const connectButton = document.querySelector('#connect-btn');
  const error = document.querySelector('#connect-error');
  error.style.display = 'none';

  connectButton.addEventListener('click', async () => {
    error.style.display = 'none';
    connectButton.setAttribute('disabled', '');
    connectButton.innerText = 'connecting...';

    try {
      await connect();
      connectButton.style.display = 'none';
    } catch(e) {
      connectButton.removeAttribute('disabled');
      connectButton.innerText = 'connect';
      
      error.style.display = '';
      error.innerText = e.toString();
    }
  });
});
