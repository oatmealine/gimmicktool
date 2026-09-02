<script lang="ts">
    import { onDestroy, onMount } from 'svelte';
  import { sendMessage } from './sometsuki.svelte';

  let outputs = $state([]) as { type: 'input' | 'output' | 'error', res: string }[];
  let history: string[] = [];
  let historyIndex = -1;
  let commandCache = '';

  let input: HTMLInputElement;
  let historyElem: HTMLDivElement;
  let anchor: HTMLDivElement;

  function pushToOutputs(part: { type: 'input' | 'output' | 'error', res: string }) {
    const anchorInView =
      historyElem.scrollTop + historyElem.clientHeight >= (historyElem.scrollHeight - 1);

    outputs.push(part);

    if (anchorInView)
      setTimeout(() => anchor.scrollIntoView(), 0);
  }

  function submit(ev: SubmitEvent) {
    ev.preventDefault();

    const cmd = input.value.trim();

    history.push(cmd);
    pushToOutputs({ type: 'input', res: cmd });

    if (cmd.length === 0) return;
    sendMessage({ t: 'eval', c: input.value });
    input.value = '';
  }

  function onMessage(ev: CustomEvent) {
    const msg = ev.detail as Message;

    if (msg.t !== 'eval_result') return;
    ev.preventDefault();

    pushToOutputs({ type: msg.o ? 'output' : 'error', res: msg.r });
  }
  
  function moveCursorToEnd(el: HTMLInputElement) {
    el.selectionStart = el.selectionEnd = el.value.length;
  }

  onMount(() => {
    historyElem.scroll(0, 1);

    //@ts-ignore
    document.addEventListener('sometsukionmessage', onMessage);
    
    input.addEventListener('keydown', (ev) => {
      const code = ev.code;

      if (history.length > 0) {
        if (code === 'ArrowUp') {
          if (historyIndex === -1) {
            commandCache = input.value;
            historyIndex = history.length - 1;
          } else if (historyIndex > 0) {
            historyIndex--;
          }
          input.value = history[historyIndex];
          setTimeout(() => moveCursorToEnd(input), 0);
        }
        if (code === 'ArrowDown') {
          if (historyIndex === -1) {
            // do nothing
          } else if (historyIndex < history.length - 1) {
            historyIndex++;
            input.value = history[historyIndex];
            setTimeout(() => moveCursorToEnd(input), 0);
          } else if (historyIndex === history.length - 1) {
            historyIndex = -1;
            input.value = commandCache;
            setTimeout(() => moveCursorToEnd(input), 0);
          }
        }
      }
    });
  });
  onDestroy(() => {
    //@ts-ignore
    document.removeEventListener('sometsukionmessage', onMessage);
  });
</script>

<style>
  .container {
    height: 100%;
    display: flex;
    flex-direction: column;
    justify-content: flex-end;

    font-family: var(--font-monospace);
    font-size: 13px;
  }
  input {
    flex: 0 0 auto;

    background-color: var(--background-color);
    border: none;
    border-top: 1px solid var(--text-light);
    padding: 0.5em;
    font-size: inherit;
    &:focus {
      outline: none;
    }

    margin-top: 1px;
  }

  .history {
    flex: 1 1 0;
    min-height: 0;

    overflow-y: auto;

    display: flex;
    flex-direction: column;
    justify-content: flex-end;
  }
  
  .line {
    &:not(:first-child) {
      border-top: 1px solid var(--text-light);
    }
    padding: 0.5em;
    &.error {
      color: var(--text-red);
    }

    display: flex;
    flex-direction: row;

    .icon {
      color: var(--text-light);
      margin-right: 0.25em;
    }
    .res {
      white-space: pre-wrap;
    }
  }

  .placeholder {
    font-style: italic;
    color: var(--text-light);
    padding: 0.5em;
  }

  form {
    display: contents;
  }

  .anchor {
    height: 1px;
  }
</style>

<div class="container">
  <div class="history" bind:this={historyElem}>
    {#each outputs as h}
      <div
        class="line"
        class:error={h.type === 'error'}
      >
        <div class="icon">
          {#if h.type === 'input'}
          &gt;
          {:else if h.type === 'output'}
          &lt;
          {:else if h.type === 'error'}
          ⚠
          {/if}
        </div>
        <div class="res">
          {h.res}
        </div>
      </div>
    {/each}
    {#if outputs.length === 0}
    <div class="placeholder">type an expression...</div>
    {/if}
    <div class="anchor" bind:this={anchor}></div>
  </div>
  
  <form onsubmit={submit}><input type="text" spellcheck="false" bind:this={input}></form>
</div>
