<script lang="ts">
  import ActorNode from './ActorNode.svelte';
  let { node, forceOpen = false }: { node: ActorTreeNode, forceOpen?: boolean } = $props();
  let expanded = $state(forceOpen ?? false);
</script>

<style>
  .node {
    &:hover {
      background-color: rgba(255, 255, 255, 0.04);
    }
  }
  .node.hidden, .node-closer.hidden {
    color: var(--text-light);
  }
  .expand {
    display: inline-block;
    cursor: pointer;
    color: var(--text-light);
    padding-right: 0.5em;

    user-select: none;
    -webkit-user-select: none;
  }
  .expand:hover {
    color: var(--text);
  }
  .children {
    padding-left: 1em;

    display: flex;
    flex-direction: column;
  }

  .node, .node-closer {
    font-family: var(--font-monospace);
    user-select: none;
    -webkit-user-select: none;
    padding-left: 0.25em;
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
      color: rgb(54, 57, 230);
    }
  }
  .node:not(.hidden) .attr, .node-closer:not(.hidden) .attr {
    color: rgb(255, 239, 147);
    @media (prefers-color-scheme: light) {
      color: rgb(219, 147, 32);
    }
  }
  .node:not(.hidden) .string, .node-closer:not(.hidden) .string {
    color: rgb(174, 255, 147);
    @media (prefers-color-scheme: light) {
      color: rgb(42, 168, 0);
    }
  }
</style>

<div class="node" class:hidden={node.h}>
  <!-- i really wish i didn't need to construct this so carefully so the whitespaces are correct -->

  {#if node.c}
    <div class="expand" onclick={() => expanded = !expanded}>{expanded ? 'v' : '>'}</div>
  {/if}&lt;<span class="type">
    {node.t}
  </span>
  {#if node.n !== ''}
  <span class="attr">
    Name
  </span>=<span class="string">
    "{node.n}"
  </span>{/if}{#if node.x && node.x !== ''}
  <span class="attr">
    Text
  </span>=<span class="string">
    "{node.x}"
  </span>{/if}{#if !node.c}/&gt;{:else}&gt;{/if}{#if node.c && !expanded}{#if node.c.length > 0}<span class="children-placeholder">
    ({node.c.length} child{node.c.length > 1 ? 'ren' : ''})
  </span>{/if}&lt;/<span class="type">{node.t}</span>&gt;{/if}
</div>
{#if node.c && expanded}
  <div class="children">
    {#each node.c as child}
      <ActorNode node={child}></ActorNode>
    {/each}
  </div>
  <div class="node-closer" class:hidden={node.h}>&lt;/<span class="type">{node.t}</span>&gt;</div>
{/if}