<script lang="ts">
  const { actor = $bindable() }: { actor: ActorExtendedData } = $props();
  
  import { sendMessage } from './sometsuki.svelte';
  import { selection } from './actors.svelte';
  import DragButton from './DragButton.svelte';
  import NumberField from './NumberField.svelte';
    import { FolderIcon, FrownIcon, GlobeIcon, InfoIcon, PrinterIcon } from 'svelte-feather-icons';

  function callMethod(methodName: string, ...values: any[]) {
    sendMessage({
      t: 'actor_call_method',
      p: selection.path,
      m: methodName, a: values,
    });
  }
  function fieldOnChange(methodName: string) {
    return (a: number) => callMethod(methodName, a);
  }

  function toRGBA(r: number, g: number, b: number, a: number): string {
    console.log(`rgba(${[r, g, b].map(n => Math.floor(n * 255)).join(',')},${a})`);
    return `rgba(${[r, g, b].map(n => Math.floor(n * 255)).join(',')},${a})`;
  }
  function fromRGBA(str: string): [ number, number, number, number ] {
    console.log(str);
    const [ r, g, b, a ] = str.slice(4, -1).split(',').map(n => parseInt(n.trim()));
    console.log(r, g, b, a);
    return [ r/255, g/255, b/255, a ];
  }

  function fromHexRGB(hex: string): [ number, number, number, 1 ] {
    let value = parseInt(hex.slice(1), 16);
    const b = value % 256;
    value = Math.floor(value / 256);
    const g = value % 256;
    value = Math.floor(value / 256);
    const r = value % 256;
    value = Math.floor(value / 256);
    return [ r / 255, g / 255, b / 255, 1 ];
  }
  function fromHexRGBA(hex: string): [ number, number, number, number ] {
    let value = parseInt(hex.slice(1), 16);
    const a = value % 256;
    value = Math.floor(value / 256);
    const b = value % 256;
    value = Math.floor(value / 256);
    const g = value % 256;
    value = Math.floor(value / 256);
    const r = value % 256;
    value = Math.floor(value / 256);
    return [ r / 255, g / 255, b / 255, a / 255 ];
  }

  function fromHex(hex: string): [ number, number, number, number ] {
    return hex.length > 7 ? fromHexRGBA(hex) : fromHexRGB(hex);
  }

  type Getter = keyof ActorExtendedData;
  type Setter = string;

  // eg. hidden
  interface BooleanAttr {
    type: 'boolean',
    getter: Getter
    setter: Setter,
    setterAsNumber?: boolean,
  }
  // eg. skewx, skewy
  interface Float2Attr {
    type: 'float2',
    getters: [Getter, Getter],
    setters: [Setter, Setter],
    step?: number,
  }
  // eg. xyz
  interface Float3Attr {
    type: 'float3',
    getters: [Getter, Getter, Getter],
    setters: [Setter, Setter, Setter],
    xyDragSetters?: [Setter, Setter],
    step?: number,
  }
  // eg. diffuse
  interface ColorAttr {
    type: 'color',
    getters: [Getter, Getter, Getter, Getter],
    setter: Setter,
  }
  // eg. rotationorder
  interface EnumAttr {
    type: 'enum',
    values: string[],
    getter: Getter,
    setter: Setter,
  }
  // eg. settext
  interface TextAttr {
    type: 'text',
    getter: Getter,
    setter: Setter,
  }

  const categories = [
    'Render',
    'Transform',
  ] as const;

  interface FieldDef {
    displayName: string,
    tooltip?: string,
    attrs: BooleanAttr | Float2Attr | Float3Attr | ColorAttr | EnumAttr,
  }

  const fieldDefs: Record<typeof categories [number], FieldDef[]> = {
    Transform: [
      {
        displayName: 'Position',
        attrs: {
          type: 'float3',
          getters: [ 'px', 'py', 'pz' ],
          setters: [ 'x', 'y', 'z' ],
          xyDragSetters: [ 'addx', 'addy' ],
        }
      },
      {
        displayName: 'Rotation',
        attrs: {
          type: 'float3',
          getters: [ 'rx', 'ry', 'rz' ],
          setters: [ 'rotationx', 'rotationy', 'rotationz' ],
        }
      },
      {
        displayName: 'Rotation Order',
        tooltip: 'Order of rotation with Euler angles',
        attrs: {
          type: 'enum',
          values: [ 'xyz', 'zyx', 'yzx' ],
          getter: 'ro',
          setter: 'SetRotationOrder',
        }
      },
      {
        displayName: 'Zoom',
        tooltip: 'Also known as scale',
        attrs: {
          type: 'float3',
          getters: [ 'zx', 'zy', 'zz' ],
          setters: [ 'zoomx', 'zoomy', 'zoomz' ],
          step: 0.01,
        }
      },
      {
        displayName: 'Base Zoom',
        tooltip: 'Applied before the actual zoom',
        attrs: {
          type: 'float3',
          getters: [ 'bzx', 'bzy', 'bzz' ],
          setters: [ 'basezoomx', 'basezoomy', 'basezoomz' ],
          step: 0.01,
        }
      },
      {
        displayName: 'Skew',
        attrs: {
          type: 'float2',
          getters: [ 'sx', 'sy' ],
          setters: [ 'skewx', 'skewy' ],
          step: 0.05,
        },
      }
    ],
    Render: [
      {
        displayName: 'Hidden',
        attrs: {
          type: 'boolean',
          getter: 'h',
          setter: 'hidden',
          setterAsNumber: true,
        }
      },
      {
        displayName: 'Diffuse',
        attrs: {
          type: 'color',
          getters: [ 'dr', 'dg', 'db', 'da' ],
          setter: 'diffuse',
        }
      }
    ],
  };
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

  .name::before {
    content: '•';
    color: var(--text-light);
    margin-right: 0.5em;
  }

  .axis.x { color: rgb(219, 72, 72); }
  .axis.y { color: rgb(96, 201, 55); }
  .axis.z { color: rgb(57, 97, 177); }

  summary {
    font-family: var(--font-display);
    color: var(--text-light);
    user-select: none;
    -webkit-user-select: none;
    margin-bottom: 0.5em;
    list-style-type: '> ';
    text-align: center;
    &::marker {
      font-family: var(--font-monospace);
    }
    &:hover::marker {
      color: var(--text);
    }
    &, &::marker {
      text-shadow:
        1px 0px 0px var(--background-color),
        2px 0px 0px var(--background-color),
        3px 0px 0px var(--background-color),
        4px 0px 0px var(--background-color),
        5px 0px 0px var(--background-color),
        -1px 0px 0px var(--background-color),
        -2px 0px 0px var(--background-color),
        -3px 0px 0px var(--background-color),
        -4px 0px 0px var(--background-color),
        -5px 0px 0px var(--background-color);
    }

    background: linear-gradient(
      to bottom,
      rgba(0, 0, 0, 0),
      rgba(0, 0, 0, 0) calc(50% - 0.1px),
      var(--text-light) 50%,
      var(--text-light) calc(50% + 0.5px),
      rgba(0, 0, 0, 0) calc(50% + 0.6px),
      rgba(0, 0, 0, 0) 100%
    );
  }
  details[open] > summary {
    list-style-type: 'v ';
  }

  .header{
    display: flex;
    flex-direction: row;
    gap: 8px;
    align-items: center;

    & .title-container{
      display: flex;
      flex-direction: column;

      & .jpath{
        vertical-align: center; opacity: 0.5; font-size: 10pt
      }
    }
  }

  .header-buttons{
    margin-left: auto;
    display: flex;
    flex-direction: row;
    gap: 8px;
    align-items: center;

    & div {
      display: flex;
      border: 1px solid var(--text-light);
      padding: 4px;
      align-items: center;
      justify-content: center;

      user-select: none;
      -webkit-user-select: none;

      cursor: help;

      aspect-ratio: 1 / 1;

      border-radius: 2px;
    }
  }

</style>

<div class="header">
  <InfoIcon></InfoIcon>
  <div class="title-container">
    <div>
      <span class="title" style="">{actor.t}</span>
      {#if actor.n !== ''}
        <span class="actor-name">"{actor.n}"</span>
      {/if}
    </div>
    <!-- Its like XPath but made by jade, therefore JPath -->
    <span class="jpath">placeholder &gt; placeholder &gt; cheese</span>
  </div>
  <div class="header-buttons">
    <div class="button"><FrownIcon size="1x"></FrownIcon></div>
    <div class="button"><PrinterIcon size="1x"></PrinterIcon></div>
    <div class="button"><GlobeIcon size="1x"></GlobeIcon></div>
  </div>
    
</div>


{#each categories as cat}
<details open={true}>
  <summary>{cat}</summary>
  
  <div class="fields">
    {#each fieldDefs[cat] as field}
    <div class="name">{field.displayName}</div>
    
    {@const attrs = field.attrs}
    {@const type = attrs.type}
    <div class="knobs">
      {#if type === 'boolean'}
      <input type="checkbox"
        bind:checked={actor[attrs.getter] as boolean}
        onchange={(ev) =>
          callMethod(attrs.setter, attrs.setterAsNumber
            ? (ev.target as HTMLInputElement).checked ? 1 : 0
            : (ev.target as HTMLInputElement).checked
          )
        }
      >
      {:else if type === 'float2'}
      <NumberField
        bind:value={actor[attrs.getters[0]] as number}
        onchange={fieldOnChange(attrs.setters[0])}
        step={attrs.step}
      ><span class="axis x">X</span></NumberField>
      <NumberField
        bind:value={actor[attrs.getters[1]] as number}
        onchange={fieldOnChange(attrs.setters[1])}
        step={attrs.step}
      ><span class="axis y">Y</span></NumberField>
      {:else if type === 'float3'}
      <NumberField
        bind:value={actor[attrs.getters[0]] as number}
        onchange={fieldOnChange(attrs.setters[0])}
        step={attrs.step}
      ><span class="axis x">X</span></NumberField>
      <NumberField
        bind:value={actor[attrs.getters[1]] as number}
        onchange={fieldOnChange(attrs.setters[1])}
        step={attrs.step}
      ><span class="axis y">Y</span></NumberField>
      <NumberField
        bind:value={actor[attrs.getters[2]] as number}
        onchange={fieldOnChange(attrs.setters[2])}
        step={attrs.step}
      ><span class="axis z">Z</span></NumberField>
      {:else if type === 'color'}
      <!-- TODO: replace with handmade color picker -->
      <input
        type="color"
        alpha="true"
        value={toRGBA(...attrs.getters.map(getter => actor[getter] as number))}
        onchange={ev => callMethod(attrs.setter, ...fromHex((ev.target as HTMLSelectElement).value))}
      >
      {:else if type === 'enum'}
      <select
        bind:value={actor[attrs.getter] as string}
        onchange={(ev) => callMethod(attrs.setter, (ev.target as HTMLSelectElement).value)}
      >
        {#each attrs.values as value}
        <option {value}>{value}</option>
        {/each}
      </select>
      {/if}
    </div>

    {#if type === 'float3' && attrs.xyDragSetters}
    <DragButton scale={1.5} onmove={(x, y) => {
      if (x !== 0) callMethod(attrs.xyDragSetters![0], x);
      if (y !== 0) callMethod(attrs.xyDragSetters![1], y);
    }}></DragButton>
    {:else}
    <div></div>
    {/if}
    {/each}
  </div>
</details>
{/each}