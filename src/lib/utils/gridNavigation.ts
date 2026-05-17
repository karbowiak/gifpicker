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

import type { TileSize } from '$lib/types';

// Column counts per tile size and breakpoint. Mirrors the CSS in
// MasonryLayout — keep both in sync if you change one.
const COLS_BY_SIZE: Record<TileSize, { desktop: number; tablet: number; mobile: number; narrow: number }> = {
  small:  { desktop: 6, tablet: 5, mobile: 3, narrow: 1 },
  medium: { desktop: 4, tablet: 3, mobile: 2, narrow: 1 },
  large:  { desktop: 3, tablet: 2, mobile: 2, narrow: 1 },
};

export function gridColumnsForWidth(width: number, size: TileSize = 'medium'): number {
  const cols = COLS_BY_SIZE[size] ?? COLS_BY_SIZE.medium;
  if (width <= 400) return cols.narrow;
  if (width <= 600) return cols.mobile;
  if (width <= 900) return cols.tablet;
  return cols.desktop;
}
