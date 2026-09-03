<script lang="ts">
  const {
    scale,
    onmove,
  }: {
    scale: number,
    onmove: (x: number, y: number) => void,
  } = $props();
  import { MoveIcon } from 'svelte-feather-icons';

  let dragButton: HTMLDivElement;

  function updatePosition(ev: MouseEvent) {
    onmove(ev.movementX * scale, ev.movementY * scale);
  }
</script>

<style>
  .drag-button {
    display: flex;
    border: 1px solid var(--text-light);
    align-items: center;
    justify-content: center;

    user-select: none;
    -webkit-user-select: none;

    cursor: move;

    height: 1.4em;
    aspect-ratio: 1 / 1;

    border-radius: var(--border-radius);
  }
</style>

<div
  class="drag-button"
  bind:this={dragButton}
  onmousedown={async ev => {
    if (ev.button !== 0) return;
    ev.preventDefault();

    await dragButton.requestPointerLock();
    dragButton.addEventListener('mousemove', updatePosition);
  }}
  onmouseup={ev => {
    if (ev.button !== 0) return;
    ev.preventDefault();
    document.exitPointerLock();
    dragButton.removeEventListener('mousemove', updatePosition);
  }}
>
  <MoveIcon size="1x"></MoveIcon>
</div>