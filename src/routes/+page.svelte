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

  let tab = $state('actors') as 'actors' | 'console';
  //let tab = $state('console') as 'actors' | 'console';

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

  .app-header {
    gap: 0.5em;
    border-bottom: 1px solid var(--text-light);
    margin-bottom: 1px;

    display: flex;
    flex-direction: row;
    
    flex: 0 0 auto;
  }
  .tabs {
    display: flex;
    flex-direction: row;
    overflow-x: auto;
    gap: 0.5em;
    padding: 0.3em 0.5em;

    flex: 1 1 0;
    min-width: 0;
  }
  .status {
    flex: 0 0 auto;
    font-family: var(--font-display);

    border-left: 1px solid var(--text-light);
    padding: 0.3em 0.5em;

    display: flex;
    flex-direction: row;
    align-items: center;

    gap: 0.3em;

    user-select: none;
    -webkit-user-select: none;

    .dot {
      border-radius: 100vw;
      width: 0.5em;
      height: 0.5em;
    }
    &.open .dot {
      background-color: #6af66a;
      @media (prefers-color-scheme: light) {
        background-color: #6cd56c;
      }
    }
    &.connecting .dot {
      background-color: #ffbe44;
    }
    &.closed .dot {
      background-color: #fe5e5e;
    }
  }
  .tab {
    user-select: none;
    -webkit-user-select: none;
    font-family: var(--font-display);

    text-wrap: nowrap;

    background: linear-gradient(currentColor 0 0) 
      bottom left/
      var(--underline-width, 0%) 0.05em
      no-repeat;
    transition: .06s background-size, .06s color;
    
    &:not(.active) {
      cursor: pointer;
    }
    &.active {
      color: var(--accent-color);
    }
    &.active, &:hover {
      --underline-width: 100%;
    }
  }
  .content {
    flex: 1 1 0;
    min-height: 0;
    overflow-y: auto;
  }

  main {
    height: 100vh;

    display: flex;
    flex-direction: column;
  }
</style>

<main>
  <div class="app-header">
    <div class="tabs">
      <div class="tab" class:active={tab === 'actors'} onclick={() => tab = 'actors'}>✻ actors</div>
      <div class="tab" class:active={tab === 'console'} onclick={() => tab = 'console'}>> console</div>
    </div>
    <div class="status"
      class:open={connection.state === 'open'}
      class:connecting={connection.state === 'connecting'}
      class:closed={connection.state === 'closed'}
    >
      <div class="dot"></div>
      {#if connection.state === 'open'}
      connected
      {:else if connection.state === 'connecting'}
      connecting
      {:else if connection.state === 'closed'}
      closed
      {/if}
    </div>
  </div>

  <div class="content">
    {#if tab === 'actors'}
      <ActorTree></ActorTree>
    {:else if tab === 'console'}
      <Console></Console>
    {/if}
  </div>
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