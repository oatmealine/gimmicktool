export const key = {
  shift: false,
  alt:   false,
  ctrl:  false,
  meta:  false,
};

document.addEventListener('keydown', ev => {
  if (ev.shiftKey) key.shift = true;
  if (ev.altKey)   key.alt   = true;
  if (ev.ctrlKey)  key.ctrl  = true;
  if (ev.metaKey)  key.meta  = true;
});
document.addEventListener('keyup', ev => {
  if (ev.shiftKey) key.shift = false;
  if (ev.altKey)   key.alt   = false;
  if (ev.ctrlKey)  key.ctrl  = false;
  if (ev.metaKey)  key.meta  = false;
});