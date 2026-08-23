<script lang="ts">
  import 'modern-normalize/modern-normalize.css';
  import '@fontsource/ibm-plex-sans/400.css';
  import '@fontsource/ibm-plex-sans/500.css';
  import '@fontsource/ibm-plex-sans/700.css';
  import '@fontsource/ibm-plex-mono/400.css';
  import '../fonts.css';
  import '../style.css';

  import { connection, disconnect, connect } from '../lib/sometsuki.svelte';
  import { onMount } from 'svelte';
  import ActorTree from '../lib/ActorTree.svelte';
  import Popover from '$lib/Popover.svelte';
    import Console from '$lib/Console.svelte';

  //let tab = $state('actors') as 'actors' | 'console';
  let tab = $state('console') as 'actors' | 'console';

  // auto-connect immediately
  onMount(async () => {
    // to make sure the previous conn is disposed
    await disconnect();
    await connect();
  });

  // workaround to not see the connection screen flash for a frame
  connection.state = 'connecting';
</script>

<style>
  .connection-status {
    font-style: italic;
    color: var(--text-light);
  }
  .connection-error {
    color: var(--text-red);
    white-space: pre-wrap;
  }

  .tabs {
    padding: 0.5em;
    gap: 0.5em;
    border-bottom: 1px solid var(--text-light);
    margin-bottom: 1px;

    display: flex;
    flex-direction: row;
  }
  .tab {
    cursor: pointer;
    user-select: none;
    -webkit-user-select: none;
    &.active {
      font-weight: bold;
    }
  }

  main {
    height: 100%;

    display: flex;
    flex-direction: column;
  }
</style>

<main>
  <div class="tabs">
    <div class="tab" class:active={tab === 'actors'} onclick={() => tab = 'actors'}>✻ actors</div>
    <div class="tab" class:active={tab === 'console'} onclick={() => tab = 'console'}>> console</div>
  </div>

  {#if tab === 'actors'}
    <ActorTree></ActorTree>
  {:else if tab === 'console'}
    <Console></Console>
  {/if}
</main>

{#if connection.state !== 'open'}
  <Popover>
    {#if connection.state === 'connecting'}
      <div class="connection-status">{connection.desc}</div>
    {:else}
      <h1>Connect</h1>
      <button onclick={connect}>Connect</button><br>
      {#if connection.error}
        <div class="connection-error">{connection.error}</div>
      {:else}
        i know this looks ugly as fuckkkk rn trust the process i'll put more here later
      {/if}
    {/if}
  </Popover>
{/if}