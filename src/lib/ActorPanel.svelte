<script lang="ts">
  const { actor = $bindable() }: { actor: ActorExtendedData } = $props();
  
  import { sendMessage } from './sometsuki.svelte';
  import { getPathValue, selection } from './actors.svelte';
  import DragButton from './DragButton.svelte';
  import NumberField from './NumberField.svelte';
  import { FolderIcon, FrownIcon, GlobeIcon, InfoIcon, PrinterIcon, StarIcon } from 'svelte-feather-icons';
  import { onMount } from 'svelte';

  import modList from '../modlist_v4_9.txt?raw';
    import { addToFavorites, getFavoriteIdx, isFavorited, removeFromFavorites } from './favoriteFields.svelte';

  function callMethod(methodName: string, ...values: any[]) {
    sendMessage({
      t: 'actor_call_method',
      p: selection.path,
      m: methodName, a: values,
    });
  }
  function setMod(modName: string, ...values: any[]) {
    sendMessage({
      t: 'player_set_mod',
      p: selection.path,
      m: modName, a: values,
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
  interface FloatAttr {
    type: 'float',
    getter: Getter,
    setter: Setter,
    step?: number,
  }

  interface ModAttrPercent {
    type: 'mod',
    modType: 'percent',
    name: string,
    default: number | undefined,
    nillable?: boolean,
  }
  interface ModAttrBool {
    type: 'mod',
    modType: 'bool',
    name: string,
    default: boolean | undefined,
    nillable?: boolean,
  }
  interface ModAttrEnum {
    type: 'mod',
    modType: 'enum',
    name: string,
    values: string[],
    default: string | undefined,
    nillable?: boolean,
  }

  type ModAttr = ModAttrPercent | ModAttrBool | ModAttrEnum;

  const categories = [
    'Render',
    'Transform',
    'Mods',
  ] as const;

  interface FieldDef {
    displayName: string,
    tooltip?: string,
    attrs: BooleanAttr | FloatAttr | Float2Attr | Float3Attr | ColorAttr | EnumAttr | ModAttr,
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
    Mods: [],
  };

  if (actor.t === 'Player') {
    const enums: Record<string, string[]> = {};

    for (const line of modList.split('\n')) {
      if (line.length === 0) continue;
      if (line.startsWith('//')) continue;
      if (line.startsWith('Enums:')) continue;

      // help me
      if (line.startsWith('    ') && line[4] !== ' ') {
        const enumName = line.split(':')[0].trim();
        const enumValues = line.split(':')[1].trim().split(', ');

        enums[enumName] = enumValues;
      }

      if (line.startsWith(' ')) continue;

      const modName = line.split(':')[0];

      if (
        modName === 'ClearAll' ||
        modName === 'Random' ||
        modName === 'FromString'
      )
        continue;

      // TODO
      if (modName.endsWith('Col')) continue;

      // these suck so bad
      const args = line.split(':')[1].split(';')[0].trim().split(', ');
      const valueDefault = line.split(':')[2].split(';')[0].trim();

      const valueArg = args[args.length - 1] === 'float approach'
        ? args[args.length - 2]
        : args[args.length - 1];

      const type = valueArg.split(' ')[0];
      const nillable = valueArg.endsWith('or nil');

      if (type === 'float') {
        fieldDefs.Mods.push({
          displayName: modName,
          attrs: {
            type: 'mod',
            modType: 'percent',
            name: modName,
            default: valueDefault !== 'nil'
              ? parseFloat(valueDefault)
              : undefined,
            nillable,
          }
        });
      } else if (type === 'bool') {
        fieldDefs.Mods.push({
          displayName: modName,
          attrs: {
            type: 'mod',
            modType: 'bool',
            name: modName,
            default: valueDefault !== 'nil'
              ? valueDefault === 'true'
              : undefined,
            nillable,
          }
        });
      } else if (type === 'enum') {
        const enumName = valueArg.split(' ')[1];
        const enumValues = enums[enumName];
        if (!enumValues) {
          console.warn(`unknown enum: ${enumValues}`);
          continue;
        }
        
        fieldDefs.Mods.push({
          displayName: modName,
          attrs: {
            type: 'mod',
            modType: 'enum',
            name: modName,
            values: enumValues,
            default: valueDefault !== 'nil'
              ? valueDefault
              : undefined,
            nillable,
          }
        });
      } else {
        console.warn(line);
        console.warn(`unknown mod type: ${type}`);
      }
    }
  }

  let pathOverflowing = $state(false);
  let pathContainer: HTMLDivElement;

  onMount(() => {
    console.log(pathContainer.scrollWidth, pathContainer.clientWidth);
    if (pathContainer.scrollWidth > pathContainer.clientWidth) {
      pathOverflowing = true;
    }
  });
</script>

<style>
  .fields {
    display: grid;
    grid-template-columns: auto auto 1fr auto;
    grid-auto-rows: 1fr;
    gap: 4px 0.5em;
    align-items: baseline;

    user-select: none;
    -webkit-user-select: none;

    width: 100%;

    .field-button {
      grid-column: 1;
    }
    .name {
      align-self: baseline;
      grid-column: 2;
    }
    .knobs {
      grid-column: 3;
    }
  }
  .knobs {
    display: grid;
    grid-template-columns: repeat(3, minmax(0, 1fr));
    gap: 0.5em;
  }
  @media (max-width: 450px) {
    .fields {
      grid-template-columns: auto auto 1fr min-content;
      grid-auto-rows: initial;
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
  
  .field {
    display: contents;
    &.favorited > * {
      order: calc(-1 - var(--fav-index));
    }
  }
  .field input[type=checkbox] {
    align-self: center;
  }
  .field-button {
    color: var(--text-light);
    .field.favorited &, &:hover {
      color: var(--text);
    }
    width: 1ex;
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
  
    list-style-type: '';

    gap: 0.25em;
    display: flex;
    justify-content: center;
    align-items: center;

    background: linear-gradient(
      to bottom,
      rgba(0, 0, 0, 0) calc(50% - 0.1px),
      var(--text-light) 50%,
      var(--text-light) calc(50% + 0.5px),
      rgba(0, 0, 0, 0) calc(50% + 0.6px)
    );
    
    & .summary-marker::after {
      font-family: var(--font-monospace);
      content: '>';
      padding: 0 0.25em;
      background: var(--background-color);
    }
    &:hover .summary-marker::after {
      color: var(--text);
    }
    & .summary-inner {
      padding: 0 0.25em;
      background: var(--background-color);
    }
  }
  details[open] > summary .summary-marker::after {
    content: 'v';
  }
  details {
    padding-bottom: 0.5em;
  }

  .header {
    display: flex;
    flex-direction: row;
    gap: 0.5em;
    align-items: center;
    padding: 0 0.25em;

    & .header-left {
      flex: 1 1 0;
      min-width: 0;

      display: flex;
      flex-direction: row;
      align-items: center;
      gap: 0.5em;

      :global(svg) {
        flex: 0 0 auto;
      }

      & .title {
        flex: 1 1 0;
        min-width: 0;
        display: flex;
        flex-direction: column;
      }
    }
  }
  
  .title {
    color: var(--text-header);
  }
  .actor-name {
    font-style: italic;
  }

  .jpath {
    font-size: 80%;
    color: var(--text-sub);

    width: 100%;

    position: relative;

    & .fader {
      position: absolute;
      left: 0;
      top: 0;
      bottom: 0;
      width: 2em;
      background: linear-gradient(to right, var(--background-color), rgba(0, 0, 0, 0));

      pointer-events: none;
      -webkit-user-select: none;
      user-select: none;
    }
    &:not(.overflowing) .fader {
      display: none;
    }

    & .path-container {
      display: flex;

      overflow: hidden;
      overscroll-behavior-x: contain;
      scroll-snap-type: x mandatory;

      & .path-part {
        &:not(:last-child)::after {
          margin: 0 0.2em;
          content: '⟩';
        }
        &:last-child {
          scroll-snap-align: end;
        }
      }
    }
  }

  .header-buttons {
    display: flex;
    flex-direction: row;
    gap: 0.5em;
    align-items: center;
    flex: 0 0 auto;

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

      border-radius: var(--border-radius);
    }
  }
</style>

<div class="header">
  <div class="header-left">
    <InfoIcon></InfoIcon>
    <div class="title">
      <div>
        <span class="actor-type" style="">{actor.t}</span>
        {#if actor.n !== ''}
          <span class="actor-name">"{actor.n}"</span>
        {/if}
      </div>
      <!-- Its like XPath but made by jade, therefore JPath -->
      <div class="jpath" class:overflowing={pathOverflowing}>
        <div class="path-container" bind:this={pathContainer}>
          {#each [0, ...selection.path!] as pathPart, i}
            {@const part = getPathValue(selection.path!.slice(0, i))!}
            <span class="path-part">{part.n || part.t}</span>
          {/each}
        </div>
        <div class="fader"></div>
      </div>
    </div>
  </div>
  <div class="header-buttons">
    <div class="button"><FrownIcon size="1x"></FrownIcon></div>
    <div class="button"><PrinterIcon size="1x"></PrinterIcon></div>
    <div class="button"><GlobeIcon size="1x"></GlobeIcon></div>
  </div>    
</div>


{#each categories as cat}
<details open={true}>
  <summary>
    <div class="summary-marker"></div>
    <div class="summary-inner">{cat}</div>
    <div></div>
  </summary>
  
  <div class="fields">
    {#each fieldDefs[cat] as field}
    {@const fav = isFavorited(field.displayName)}
    {@const attrs = field.attrs}
    {@const type = attrs.type}

    <div class="field" class:favorited={fav} style:--fav-index={getFavoriteIdx(field.displayName)}>
      <div
        class="field-button"
        onclick={() =>
          !fav
            ? addToFavorites(field.displayName)
            : removeFromFavorites(field.displayName)
        }>
        {#if fav}
        <!-- doing 0.5em and hoping it's equal to 1ex here is finnicky as fuck -->
        <StarIcon size="0.5x"></StarIcon>
        {:else}
        •
        {/if}
      </div>
      <div class="name">{field.displayName}</div>
      
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
        {:else if type === 'float'}
        <NumberField
          bind:value={actor[attrs.getter] as number}
          onchange={fieldOnChange(attrs.setter)}
          step={attrs.step}
        ></NumberField>
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
        {:else if type === 'mod'}
          {#if attrs.modType === 'percent'}
          <NumberField
            bind:value={actor[`m_${attrs.name}`] as number}
            onchange={(a) => setMod(attrs.name, a, -1)}
            step={0.1}
          ></NumberField>
          {:else if attrs.modType === 'bool'}
          <input type="checkbox"
            bind:checked={actor[`m_${attrs.name}`] as boolean}
            onchange={(ev) => setMod(attrs.name, (ev.target as HTMLInputElement).checked)}
          >
          {:else if attrs.modType === 'enum'}
          <select
            bind:value={actor[`m_${attrs.name}`] as string}
            onchange={(ev) => setMod(attrs.name, (ev.target as HTMLSelectElement).value)}
          >
            {#each attrs.values as value}
            <option {value}>{value.split('_')[1]}</option>
            {/each}
          </select>
          {/if}
        {/if}
      </div>

      {#if type === 'float3' && attrs.xyDragSetters}
      <DragButton scale={1.5} onmove={(x, y) => {
        if (x !== 0) callMethod(attrs.xyDragSetters![0], x);
        if (y !== 0) callMethod(attrs.xyDragSetters![1], y);
      }}></DragButton>
      {/if}
    </div>
    {/each}
  </div>
</details>
{/each}