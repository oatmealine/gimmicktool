import { load } from '@tauri-apps/plugin-store';

const store = await load('config.json', { defaults: {
  consoleHistory: [],
} });

const HISTORY_SIZE = 100;

export async function consolePushHistory(cmd: string) {
  const prevHistory = await store.get<string[]>('consoleHistory') ?? [];

  if (prevHistory.length >= HISTORY_SIZE)
    prevHistory.shift();

  await store.set('consoleHistory',
    [...prevHistory.filter(c => c !== cmd), cmd]
  );
}