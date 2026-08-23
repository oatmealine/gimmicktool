<script lang="ts">
  import { onMount, onDestroy } from 'svelte';
  import { connection, sendMessage } from './sometsuki.svelte';
  import ActorNode from './ActorNode.svelte';

  let root: ActorTreeNode | null = $state.raw(null);

  function onConnected() {
    sendMessage({ t: 'poll_actor_tree', v: true });
  }

  function onMessage(ev: CustomEvent) {
    const msg = ev.detail as Message;

    if (msg.t !== 'actor_tree') return;
    ev.preventDefault();

    root = msg.d as ActorTreeNode;
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
    height: 100%;
    padding: 0.5em 0;
    font-size: 14px;
  }
  .loading {
    padding: 0.5em;
    color: var(--text-light);
    font-style: italic;
  }
</style>

{#if root}
  <div class="container">
    <ActorNode node={root} forceOpen={true}></ActorNode>
  </div>
{:else if connection.state === 'open'}
  <div class="loading">loading...</div>
{/if}