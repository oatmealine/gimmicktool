<script lang="ts">
	let { children } = $props();
</script>

<style>
  .popover {
    position: fixed;
    top: 0;
    left: 0;
    width: 100vw;
    width: 100dvw;
    height: 100vh;
    height: 100dvh;
    
    background-color: rgba(0, 0, 0, 0.4);
    user-select: none;
    -webkit-user-select: none;

    overflow: auto;

    display: flex;
    align-items: center;
    justify-content: center;
  }
  .popover-content {
    width: 350px;
    max-width: 100%;
    height: 450px;
    max-height: 100%;
    background-color: var(--background-color);

    padding: 1em;

    display: flex;
    flex-direction: column;
    align-items: center;
    justify-content: center;
    text-align: center;

    word-break: break-word;
  }

  :global(body):has(.popover) {
    overflow: hidden;
  }
</style>

<div class="popover">
  <div class="popover-content">
    {@render children()}
  </div>
</div>
