<!-- blender-like draggable number input -->
<script lang="ts">
  import type { Snippet } from 'svelte';
  import prettyNum from 'pretty-num';
  import { key } from './keyboard.svelte';

	let {
    children,
    value = $bindable(),
    onchange,
    step = 1,
  }: {
    children?: Snippet,
    value: number,
    onchange: (a: number) => void,
    step?: number,
  } = $props();

  let editing = $state(false);
  let dragging = $state(false);

  const DRAG_THRES = 2;

  let knob: HTMLDivElement;
  let input: HTMLInputElement;

  let mx = 0;
  let remainder = 0;
  function updatePosition(ev: MouseEvent) {
    mx += ev.movementX;

    if (!dragging && Math.abs(mx) > DRAG_THRES) {
      dragging = true;
    }
    if (dragging) {
      let base = step;
      let mult = 1;
      if (key.alt) base = 0.25;
      if (key.shift) mult = 0.1;

      const delta = ev.movementX * base * mult;
      if (key.alt) {
        remainder += value % 1;
        value -= value % 1;
        remainder += delta;
        if (Math.abs(remainder) >= 1) {
          value += Math.trunc(remainder);
          remainder -= Math.trunc(remainder);
        }
      } else {
        value += delta;
      }
      onchange(value);
    }
  }

  function release() {
    if (!document.pointerLockElement) return;

    document.exitPointerLock();
    knob.removeEventListener('mousemove', updatePosition);
    if (!dragging) {
      editing = true;
      setTimeout(() => {
        input.focus();
        input.select();
      }, 0);
    } else {
      dragging = false;
    }
  }
</script>

<style>
  .knob {
    display: flex;
    flex-direction: row;
    border: 1px solid var(--text-light);
    border-radius: var(--border-radius);
    align-items: center;
    width: 100%;
    min-height: 1.38em; /* hacky input height fix */

    .label {
      padding: 0 0.5em;
      border-right: 1px solid var(--text-light);
      font-family: var(--font-monospace);
    }

    &:has(> input:focus) {
      border-color: var(--accent-color);
    }

    .preview, input {
      width: 100%;
      padding: 0 0.5em;
    }

    .preview {
      cursor: text;
      overflow: hidden;
      white-space: nowrap;
      text-overflow: clip;
    }

    input {
      display: block;
      overflow: auto;
      
      -moz-appearance: textfield;
      appearance: none;
      &::-webkit-outer-spin-button,
      &::-webkit-inner-spin-button {
        display: none;
      }
    }
  }
</style>

<div class="knob" bind:this={knob} onmouseup={(ev) => {
  if (ev.button !== 0) return;
  ev.preventDefault();
  release();
}}>
  {#if children !== undefined}
    <div class="label">
      {@render children()}
    </div>
  {/if}
  {#if editing}
    <input
      type="number"
      bind:value={value}
      bind:this={input}
      onchange={(ev) => onchange((ev.target as HTMLInputElement).valueAsNumber)}
      onfocusout={() => editing = false}
      onkeydown={(ev) => {
        if (ev.code === 'Enter') {
          onchange((ev.target as HTMLInputElement).valueAsNumber);
          editing = false;
        }
      }}
    >
  {:else}
    <div class="preview" onmousedown={async ev => {
      if (ev.button !== 0) return;

      mx = 0;
      await knob.requestPointerLock();
      knob.addEventListener('mousemove', updatePosition);
    }}>
      {prettyNum(value, {precision: 3})}
    </div>
  {/if}
</div>