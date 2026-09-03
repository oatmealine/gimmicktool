<script lang="ts">
  import { onMount, onDestroy } from 'svelte';
  import { connection, sendMessage } from './sometsuki.svelte';
  import ActorNode from './ActorNode.svelte';
  import { getActorRoot, getPathValue, isSelected, selection, setActorRoot, setPathValue, setSelection } from './actors.svelte';
  import ActorPanel from './ActorPanel.svelte';

  function onConnected() {
    sendMessage({ t: 'poll_actor_tree', v: true });
  }

  function onMessage(ev: CustomEvent) {
    const msg = ev.detail as Message;

    if (
      msg.t !== 'actor_tree_init' && msg.t !== 'actor_tree_update' &&
      msg.t !== 'actor_init' && msg.t !== 'actor_update' &&
      msg.t !== 'actor_delete'
    ) return;
    ev.preventDefault();

    if (msg.t === 'actor_tree_init') {
      setActorRoot(msg.d as ActorTreeNode);
    } else if (msg.t === 'actor_tree_update') {
      if (msg.e === 'set') {
        if (selection.path && isSelected(msg.p))
          setSelection(null);
        setPathValue(msg.p, msg.d);
      } else if (msg.e === 'update') {
        const value = getPathValue(msg.p)!;
        for (const [ k, v ] of Object.entries(msg.d)) {
          //@ts-ignore
          value[k] = v;
        }
      } else if (msg.e === 'delete') {
        if (selection.path && isSelected(msg.p))
          setSelection(null);
        setPathValue(msg.p, undefined);
      }
    } else if (msg.t === 'actor_init') {
      selection.data = null; // fix a stupid rune bug
      selection.data = msg.d;
    } else if (msg.t === 'actor_update') {
      if (selection.path && isSelected(msg.p)) {
        selection.data = { ...selection.data, ...msg.d };
      }
    } else if (msg.t === 'actor_delete') {
      if (selection.path && isSelected(msg.p)) {
        setSelection(null);
      }
    }
  }

  onMount(() => {
    // to account for being reloaded during dev
    if (connection.state === 'open')
      onConnected();

    document.addEventListener('sometsukiconnected', onConnected);
    //@ts-ignore
    document.addEventListener('sometsukionmessage', onMessage);
  });
  
  onDestroy(() => {
    document.removeEventListener('sometsukiconnected', onConnected);
    //@ts-ignore
    document.removeEventListener('sometsukionmessage', onMessage);

    if (connection.state === 'open')
      sendMessage({ t: 'poll_actor_tree', v: false });
  });
</script>

<style>
  .container {
    display: flex;
    flex-direction: column;

    height: 100%;
  }
  .tree {
    flex: 1 1 0;
    min-height: 0;
    font-size: 10pt;
    --left: 0em;
    
    padding: 0.5em 0;
    overflow: auto;
  }
  .selected {
    flex: 0 0 auto;
    padding: 0.5em;
    border-top: 1px solid var(--text-light);
    margin-top: 1px;

    max-height: 50vh;
    height: 20em;

    overflow: auto;
  }
  .loading {
    padding: 0.5em;
    color: var(--text-light);
    font-style: italic;
  }
</style>

{#if getActorRoot()}
  <div class="container">
    <div class="tree">
      <ActorNode node={getActorRoot()!} forceOpen={true} path={[]}></ActorNode>
    </div>
    {#if selection.path}
      <div class="selected">
        {#if !selection.data}
          <div class="loading">loading...</div>
        {:else}
          <ActorPanel bind:actor={selection.data}></ActorPanel>
        {/if}
      </div>
    {/if}
  </div>
{:else if connection.state === 'open'}
  <div class="loading">loading...</div>
{/if}