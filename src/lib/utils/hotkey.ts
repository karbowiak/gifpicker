// Utilities for capturing keyboard shortcuts and converting them to Tauri's
// accelerator string format (e.g. "Option+Cmd+G", "Ctrl+Shift+F5").
//
// We use `event.code` (physical key) rather than `event.key` (rendered char) so
// the captured hotkey is layout-independent — Cmd+Shift+2 on a US layout
// captures the same as Cmd+Shift+2 on a German layout.

/// Maps a KeyboardEvent.code to the Tauri accelerator key token.
/// Returns null for modifier keys (those are handled separately) and for codes
/// we don't support (mouse buttons, dead keys, etc.).
function codeToAccelerator(code: string): string | null {
  // KeyA → A, KeyZ → Z
  if (code.startsWith('Key') && code.length === 4) return code.slice(3);
  // Digit0 → 0, Digit9 → 9
  if (code.startsWith('Digit') && code.length === 6) return code.slice(5);
  // F1 → F1, F24 → F24
  if (/^F\d{1,2}$/.test(code)) return code;
  // Numpad0 → Num0 (Tauri accepts Num0..9)
  if (code.startsWith('Numpad') && code.length === 7) return `Num${code.slice(6)}`;

  switch (code) {
    case 'Space': return 'Space';
    case 'Enter': return 'Enter';
    case 'Tab': return 'Tab';
    case 'Backspace': return 'Backspace';
    case 'Delete': return 'Delete';
    case 'Escape': return 'Escape';
    case 'ArrowUp': return 'Up';
    case 'ArrowDown': return 'Down';
    case 'ArrowLeft': return 'Left';
    case 'ArrowRight': return 'Right';
    case 'Home': return 'Home';
    case 'End': return 'End';
    case 'PageUp': return 'PageUp';
    case 'PageDown': return 'PageDown';
    case 'Insert': return 'Insert';
    case 'Comma': return ',';
    case 'Period': return '.';
    case 'Slash': return '/';
    case 'Backslash': return '\\';
    case 'Semicolon': return ';';
    case 'Quote': return "'";
    case 'BracketLeft': return '[';
    case 'BracketRight': return ']';
    case 'Minus': return '-';
    case 'Equal': return '=';
    case 'Backquote': return '`';
    default: return null;
  }
}

const MODIFIER_CODES = new Set([
  'ControlLeft', 'ControlRight',
  'ShiftLeft', 'ShiftRight',
  'AltLeft', 'AltRight',
  'MetaLeft', 'MetaRight', 'OSLeft', 'OSRight',
]);

export function isModifierKey(code: string): boolean {
  return MODIFIER_CODES.has(code);
}

/// Build a Tauri accelerator string from a KeyboardEvent. Returns null if the
/// event is just a bare modifier (e.g. Shift on its own) or the key isn't
/// representable as an accelerator.
///
/// On macOS we prefer `Cmd` over `Meta`; on other platforms `Meta`/`Super`
/// stays as `Meta` to match the win-key.
export function eventToAccelerator(event: KeyboardEvent): string | null {
  const key = codeToAccelerator(event.code);
  if (!key) return null;

  const parts: string[] = [];
  if (event.ctrlKey) parts.push('Ctrl');
  if (event.altKey) parts.push(isMac() ? 'Option' : 'Alt');
  if (event.shiftKey) parts.push('Shift');
  if (event.metaKey) parts.push(isMac() ? 'Cmd' : 'Meta');
  parts.push(key);

  return parts.join('+');
}

/// Render an accelerator string using platform-native glyphs where applicable.
/// `Option+Cmd+G` → `⌥⌘G` on macOS, `Alt+Win+G` on Windows.
export function displayAccelerator(accelerator: string): string {
  if (!accelerator) return '';
  const parts = accelerator.split('+');
  if (isMac()) {
    const map: Record<string, string> = {
      Cmd: '⌘', Command: '⌘', Meta: '⌘', Super: '⌘',
      Ctrl: '⌃', Control: '⌃',
      Alt: '⌥', Option: '⌥',
      Shift: '⇧',
      // arrow & special — keep readable
      Enter: '↵', Tab: '⇥', Backspace: '⌫', Delete: '⌦',
      Escape: 'Esc', Space: 'Space',
      Up: '↑', Down: '↓', Left: '←', Right: '→',
    };
    return parts.map((p) => map[p] ?? p).join('');
  }
  return parts.join('+');
}

function isMac(): boolean {
  if (typeof navigator === 'undefined') return false;
  return /Mac|iPhone|iPad/.test(navigator.platform);
}
