const { invoke } = window.__TAURI__.core;

let greetInputEl;
let greetMsgEl;

async function greet() {
  console.log('attempting connection');
  let pid, ver, verInfo;
  try {
    [ pid, ver, verInfo ] = await invoke('try_connect');  
  } catch(err) {
    console.log('fail: ', err);
    return;
  }
  console.log('success: ', pid, ver, verInfo);
  const external = await invoke('get_external', { pid, baseAddress: verInfo.base_address, size: verInfo.size });
  console.log('retrieved GetExternal range:');
  console.log(external);
}

window.addEventListener("DOMContentLoaded", () => {
  greetInputEl = document.querySelector("#greet-input");
  greetMsgEl = document.querySelector("#greet-msg");
  document.querySelector("#greet-form").addEventListener("submit", (e) => {
    e.preventDefault();
    greet();
  });
});
