<script lang="ts">
  const { actor = $bindable() }: { actor: ActorExtendedData } = $props();
  
  import { sendMessage } from './sometsuki.svelte';
  import { selection } from './actors.svelte';
  import DragButton from './DragButton.svelte';
  import NumberField from './NumberField.svelte';

  function callMethod(methodName: string, value: any) {
    sendMessage({
      t: 'actor_call_method',
      p: selection.path,
      m: methodName, a: [ value ],
    });
  }
  function fieldOnChange(methodName: string) {
    return (a: number) => callMethod(methodName, a);
  }
</script>

<style>
  .fields {
    display: grid;
    grid-template-columns: auto 1fr auto;
    gap: 4px 0.5em;
    align-items: center;

    user-select: none;
    -webkit-user-select: none;

    width: 100%;

    .name {
      align-self: baseline;
    }
  }
  .knobs {
    display: grid;
    grid-template-columns: repeat(3, minmax(0, 1fr));
    gap: 0.5em;
  }
  @media (max-width: 450px) {
    .fields {
      grid-template-columns: auto 1fr min-content;
      align-items: flex-start;
      gap: 0.5em 0.5em;
      .name {
        margin-right: 0.5em;
      }
    }
    .knobs {
      display: flex;
      flex-direction: column;
      align-items: stretch;
      justify-content: flex-start;
      gap: 4px;
    }
  }
  .title {
    color: var(--text-header);
  }
  .actor-name {
    font-style: italic;
  }

  .axis.x { color: #f00; }
  .axis.y { color: #0f0; }
  .axis.z { color: #00f; }

  summary {
    font-family: var(--font-display);
    color: var(--text-light);
    user-select: none;
    -webkit-user-select: none;
    margin-bottom: 0.5em;
    list-style-type: '> ';
    &::marker {
      font-family: var(--font-monospace);
    }
    &:hover::marker {
      color: var(--text);
    }
  }
  details[open] > summary {
    list-style-type: 'v ';
  }
</style>

<span class="title">{actor.t}</span> {#if actor.n !== ''}<span class="actor-name">"{actor.n}"</span>{/if}

<details open={true}>
  <summary>Rendering</summary>
  
  <div class="fields">
    <div class="name">Hidden</div>
    <div class="knobs">
      <input type="checkbox" bind:checked={actor.h} onchange={(ev) => callMethod('hidden', (ev.target as HTMLInputElement).checked ? 1 : 0)}>
    </div>
    <div></div>
  </div>
</details>

<details open={true}>
  <summary>Transform</summary>
  
  <div class="fields">
    <div class="name">Position</div>
    <div class="knobs">
      <NumberField bind:value={actor.px} onchange={fieldOnChange('x')}><span class="axis x">X</span></NumberField>
      <NumberField bind:value={actor.py} onchange={fieldOnChange('y')}><span class="axis y">Y</span></NumberField>
      <NumberField bind:value={actor.pz} onchange={fieldOnChange('z')}><span class="axis z">Z</span></NumberField>
    </div>
    <DragButton scale={1.5} onmove={(x, y) => {
      actor.px += x;
      actor.py += y;
      sendMessage({
        t: 'actor_call_method',
        p: selection.path,
        m: 'xy',
        a: [ actor.px, actor.py ],
      });
    }}></DragButton>
  
    <div class="name">Rotation</div>
    <div class="knobs">
      <NumberField bind:value={actor.rx} onchange={fieldOnChange('rotationx')}><span class="axis x">X</span></NumberField>
      <NumberField bind:value={actor.ry} onchange={fieldOnChange('rotationy')}><span class="axis y">Y</span></NumberField>
      <NumberField bind:value={actor.rz} onchange={fieldOnChange('rotationz')}><span class="axis z">Z</span></NumberField>
    </div>
    <div></div>
    
    <div class="name">Rotation Order</div>
    <div class="knobs" style="font-family: var(--font-monospace)">
      {actor.ro}
    </div>
    <div></div>
    
    <div class="name">Zoom</div>
    <div class="knobs">
      <NumberField bind:value={actor.zx} onchange={fieldOnChange('zoomx')} step={0.01}><span class="axis x">X</span></NumberField>
      <NumberField bind:value={actor.zy} onchange={fieldOnChange('zoomy')} step={0.01}><span class="axis y">Y</span></NumberField>
      <NumberField bind:value={actor.zz} onchange={fieldOnChange('zoomz')} step={0.01}><span class="axis z">Z</span></NumberField>
    </div>
    <div></div>
    
    <div class="name">Base Zoom</div>
    <div class="knobs">
      <NumberField bind:value={actor.bzx} onchange={fieldOnChange('basezoomx')} step={0.01}><span class="axis x">X</span></NumberField>
      <NumberField bind:value={actor.bzy} onchange={fieldOnChange('basezoomy')} step={0.01}><span class="axis y">Y</span></NumberField>
      <NumberField bind:value={actor.bzz} onchange={fieldOnChange('basezoomz')} step={0.01}><span class="axis z">Z</span></NumberField>
    </div>
    <div></div>
  
    <div class="name">Skew</div>
    <div class="knobs">
      <NumberField bind:value={actor.sx} onchange={fieldOnChange('skewx')} step={0.01}><span class="axis x">X</span></NumberField>
      <NumberField bind:value={actor.sy} onchange={fieldOnChange('skewy')} step={0.01}><span class="axis y">Y</span></NumberField>
    </div>
    <div></div>
  </div>
</details>

{#if actor.x !== undefined}
<details open={true}>
  <summary>Text</summary>
  
  <div class="fields">
    <div class="name">Content</div>
    <div class="knobs">
      <div class="knob">
        <input bind:value={actor.x} oninput={(ev) => sendMessage({
          t: 'actor_call_method',
          p: selection.path,
          m: 'settext', a: [ ev.target!.value ],
        })}>
      </div>
    </div>
    <div></div>
  </div>
</details>
{/if}