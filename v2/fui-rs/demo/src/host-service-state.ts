type Listener<T> = (value: T) => void;

interface DemoShellState {
  tick: number;
  accentColor: number;
  darkMode: boolean;
}

const state: DemoShellState = {
  tick: 0,
  accentColor: 0x2563EBff,
  darkMode: false,
};

const tickListeners = new Set<Listener<number>>();
const accentListeners = new Set<Listener<number>>();
const darkModeListeners = new Set<Listener<boolean>>();

function notify<T>(listeners: Set<Listener<T>>, value: T): void {
  for (const listener of listeners) {
    listener(value);
  }
}

export function readDemoShellState(): DemoShellState {
  return state;
}

export function setDemoShellTick(value: number): void {
  if (state.tick === value) {
    return;
  }
  state.tick = value;
  notify(tickListeners, value);
}

export function setDemoShellAccentColor(value: number): void {
  if (state.accentColor === value) {
    return;
  }
  state.accentColor = value >>> 0;
  notify(accentListeners, state.accentColor);
}

export function setDemoShellDarkMode(value: boolean): void {
  if (state.darkMode === value) {
    return;
  }
  state.darkMode = value;
  notify(darkModeListeners, value);
}

export function subscribeDemoShellClockTick(listener: Listener<number>): () => void {
  tickListeners.add(listener);
  return () => tickListeners.delete(listener);
}

export function subscribeDemoShellAccentColor(listener: Listener<number>): () => void {
  accentListeners.add(listener);
  return () => accentListeners.delete(listener);
}

export function subscribeDemoShellDarkMode(listener: Listener<boolean>): () => void {
  darkModeListeners.add(listener);
  return () => darkModeListeners.delete(listener);
}

