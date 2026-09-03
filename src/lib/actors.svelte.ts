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

let root: ActorTreeNode | null = $state(null);

export function getPathValue(path: number[]) {
  if (!root) throw 'no actor tree loaded';

  let node: ActorTreeNode | undefined = root;
  for (const i of path) {
    node = node?.c?.[i];
  }
  return node;
}
export function setPathValue(path: number[], value: ActorTreeNode | undefined) {
  if (path.length === 0) root = value ?? null;
  if (!root) throw 'no actor tree loaded';

  let node: ActorTreeNode = root;
  for (const i of path.slice(0, -1)) {
    if (!node.c) throw `invalid path ${path.join('/')}`;
    node = node?.c?.[i];
    if (!node) throw `invalid path ${path.join('/')}`;
  }
  
  if (!node.c) throw `invalid path ${path.join('/')}`;

  if (value === undefined) {
    node.c.splice(path[path.length - 1], 1);
  } else {
    node.c[path[path.length - 1]] = value;
  }
}
export function setActorRoot(value: ActorTreeNode | null) {
  root = value;
}
export function getActorRoot() {
  return root;
}