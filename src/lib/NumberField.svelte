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
    allowNull = false,
    defaultValue = 0,
  }: {
    children?: Snippet,
    value: number,
    step?: number,
    onchange: (a: number | undefined) => void,
    allowNull?: boolean,
    defaultValue?: number,
  } = $props();

  // svelte-ignore state_referenced_locally
  let lastValidValue: number | undefined = defaultValue;

  let editing = $state(false);
  let dragging = $state(false);

  const DRAG_THRES = 2;

  let knob: HTMLDivElement;
  // svelte-ignore non_reactive_update
  let input: HTMLInputElement;

  let mx = 0;
  let remainder = 0;
  function updatePosition(ev: MouseEvent) {
    mx += ev.movementX;

    if (!dragging && Math.abs(mx) > DRAG_THRES) {
      dragging = true;
    }
    if (dragging) {
      if (value === undefined || isNaN(value)) value = 0;

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

  function onChange(input: HTMLInputElement) {
    const newValue = input.valueAsNumber;
    if (input.value.trim().length === 0 || isNaN(newValue)) {
      if (allowNull) {
        // @ts-ignore you do not understand my intents
        onchange(undefined);
        // @ts-ignore you still do not understand my intents
        value = undefined;
        lastValidValue = undefined;
      } else {
        value = defaultValue;
      }
    } else {
      value = newValue;
      lastValidValue = value;
      onchange(newValue);
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
      & .nil {
        color: var(--text-light);
        font-style: italic;
      }
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
      placeholder="{allowNull ? 'nil' : defaultValue.toString()}"
      onchange={(ev) => onChange(ev.target as HTMLInputElement)}
      onfocusout={() => editing = false}
      onkeydown={(ev) => {
        if (ev.code === 'Enter') {
          onChange(ev.target as HTMLInputElement);
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
      {#if value !== undefined}
      {prettyNum(value, {precision: 3})}
      {:else}
      <span class="nil">nil</span>
      {/if}
    </div>
  {/if}
</div>