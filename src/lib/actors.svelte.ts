import { sendMessage } from './sometsuki.svelte';

export const selection = $state({
  path: null as number[] | null,
  data: null as ActorExtendedData | null,
});

export function setSelection(path: number[] | null) {
  if (path !== null && isSelected(path)) return;
  selection.path = path;
  selection.data = null;
  sendMessage({ t: 'poll_actor', p: path });
}

export function isSelected(path: number[]) {
  return (
    selection.path?.length === path.length &&
    selection.path?.every((v, i) => path[i] === v)
  );
}