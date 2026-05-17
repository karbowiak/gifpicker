// Compute the next selection index given the current key press and grid layout.
// Returns null when the key doesn't change selection. Pure: no DOM, no stores.

export type GridDirection = 'up' | 'down' | 'left' | 'right';

const DIRECTION_KEYS: Record<string, GridDirection> = {
  ArrowUp: 'up',
  ArrowDown: 'down',
  ArrowLeft: 'left',
  ArrowRight: 'right',
};

export function keyToDirection(key: string): GridDirection | null {
  return DIRECTION_KEYS[key] ?? null;
}

export function nextIndex(
  current: number,
  direction: GridDirection,
  itemCount: number,
  cols: number,
): number | null {
  if (itemCount === 0) return null;
  const last = itemCount - 1;

  switch (direction) {
    case 'down':
      if (current + cols < itemCount) return current + cols;
      if (current < last) return last;
      return null;
    case 'up':
      if (current - cols >= 0) return current - cols;
      if (current > 0) return 0;
      return null;
    case 'right':
      return current < last ? current + 1 : null;
    case 'left':
      return current > 0 ? current - 1 : null;
  }
}

// Column count for the current window width. Mirrors the CSS breakpoints in
// MasonryLayout — keep both in sync if you change one.
export function gridColumnsForWidth(width: number): number {
  if (width <= 400) return 1;
  if (width <= 600) return 2;
  if (width <= 900) return 3;
  return 4;
}
