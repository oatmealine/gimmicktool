<script lang="ts">
  import ActorNode from './ActorNode.svelte';

  import { EyeIcon, EyeOffIcon } from 'svelte-feather-icons';
  import { sendMessage } from './sometsuki.svelte';
  import { setSelection, isSelected } from './actors.svelte';

  let {
    node,
    forceOpen = false,
    path = [],
  }: {
    node: ActorTreeNode,
    forceOpen?: boolean,
    path: number[],
  } = $props();
  let expanded = $state(forceOpen ?? false);

  function toggleHide() {
    //sendMessage({ t: 'actor_set_hidden', p: path, v: !node.h });
    sendMessage({ t: 'actor_call_method', p: path, m: 'hidden', a: [ !node.h ? 1 : 0 ] })
  }
</script>

<style>
  .node {
    &:hover, &.selected {
      background-color: rgba(0, 0, 0, 0.03);
      @media (prefers-color-scheme: dark) {
        background-color: rgba(255, 255, 255, 0.04);
      }
    }
  }
  .node.hidden, .node-closer.hidden {
    color: var(--text-light);
  }
  .expand {
    display: inline-block;
    color: var(--text-light);
    padding-right: 0.5em;

    user-select: none;
    -webkit-user-select: none;
  }
  .expand:hover {
    color: var(--text);
  }
  .children {
    display: flex;
    flex-direction: column;
  }

  .node, .node-closer {
    font-family: var(--font-monospace);
    padding-left: calc(0.25em + var(--left));
    cursor: default;
  }

  .children-placeholder {
    color: var(--text-light);
    user-select: none;
    -webkit-user-select: none;

    padding: 0 1ex;
  }

  .node:not(.hidden) .type, .node-closer:not(.hidden) .type {
    color: rgb(147, 149, 255);
    @media (prefers-color-scheme: light) {
      color: rgb(37, 39, 171);
    }
  }
  .node:not(.hidden) .attr, .node-closer:not(.hidden) .attr {
    color: rgb(255, 239, 147);
    @media (prefers-color-scheme: light) {
      color: rgb(146, 109, 44);
    }
  }
  .node:not(.hidden) .string, .node-closer:not(.hidden) .string {
    color: rgb(174, 255, 147);
    @media (prefers-color-scheme: light) {
      color: rgb(51, 150, 18);
    }
  }

  .buttons {
    display: inline-block;

    .button {
      color: var(--text-light);
      &:hover {
        color: var(--text);
      }
    }
  }
</style>

{#snippet buttons()}
  <div class="buttons">
    <div class="button hide" onclick={(e) => (toggleHide(), e.preventDefault(), e.stopPropagation())}>
      {#if node.h}
        <EyeOffIcon size="0.75x"></EyeOffIcon>
      {:else}
        <EyeIcon size="0.75x"></EyeIcon>
      {/if}
    </div>
  </div>
{/snippet}

<div
  class="node"
  class:selected={isSelected(path)}
  class:hidden={node.h}
  style="--left: {path.length}em"
  onclick={() => setSelection(path)}
>
  <!-- i really wish i didn't need to construct this so carefully so the whitespaces are correct -->

  {#if node.c}
    <div class="expand" onclick={(e) => (expanded = !expanded, e.preventDefault(), e.stopPropagation())}>{expanded ? 'v' : '>'}</div>
  {/if}&lt;<span class="type">
    {node.t}
  </span>{#if node.n !== ''}
  &nbsp;<span class="attr">
    Name
  </span>=<span class="string">
    "{node.n}"
  </span>{/if}{#if node.x}
  &nbsp;<span class="attr">
    Text
  </span>=<span class="string">
    "{node.x}"
  </span>{/if}{#if !node.c}/&gt;{:else}&gt;{/if}{#if node.c && !expanded}{#if node.c.length > 0}<span class="children-placeholder">
    ({node.c.length} child{node.c.length > 1 ? 'ren' : ''})
  </span>{/if}&lt;/<span class="type">{node.t}</span>&gt;{/if}
  {#if !node.c || !expanded}
  {@render buttons()}
  {/if}
</div>
{#if node.c && expanded}
  <div class="children">
    {#each node.c as child, i}
      <ActorNode node={child} path={[...path, i]}></ActorNode>
    {/each}
  </div>
  <div class="node-closer" class:hidden={node.h} style="--left: {path.length}em">
    &lt;/<span class="type">{node.t}</span>&gt;
    {@render buttons()}
  </div>
{/if}