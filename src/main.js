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
  channel.onmessage = msg => {
    if (msg.event === 'connected') {
      console.log(`connected to ${msg.data.name} v${msg.data.version}`);
      helloPromise();
    } else if (msg.event === 'error') {
      console.error(msg.data.message);
    } else if (msg.event === 'message') {
      console.log('msg', msg);
    } else if (msg.event === 'disconnected') {
      console.log(':(');
      onDisconnected();
    }
  };
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
}

window.addEventListener("DOMContentLoaded", async () => {  
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
