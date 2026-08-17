# libghostty-vt local patches

This file tracks intentional local changes applied on top of the vendored
`libghostty-vt` source. Remove a patch only when the vendored source commit
contains the upstream behavior and the listed verification still passes.

## 0001 default lib-vt panes to grapheme clustering

status: active

patch: `vendor/patches/libghostty-vt/0001-default-grapheme-cluster-mode.patch`

herdr issue: https://github.com/herdrdev/herdr/issues/243

upstream discussion: not opened; libghostty-vt currently exposes current mode mutation but no C API for configuring terminal default modes

upstream pr: not opened

vendored base: `c5a21edfcbc2d5b46540ad91b7980aca31f5f1f3`

local files:

- `vendor/libghostty-vt/src/terminal/c/terminal.zig`

reason: Herdr renders terminal cells directly and requires DEC private mode
2027 to store flags, ZWJ emoji, and other multi-codepoint grapheme clusters in
one cell. This patch makes clustering active for new terminals and keeps it as
the reset default so RIS (`ESC c`) does not disable it.

remove when: libghostty-vt exposes a C API for setting default mode 2027, or
upstream makes grapheme clustering the lib-vt default, and the reset-survival
regression passes without this patch.

verification:

```sh
cargo nextest run --locked grapheme_cluster_mode_is_default_and_survives_full_reset
cargo nextest run --locked grapheme_cluster_mode_renders_flag_emoji_in_single_wide_cell
cargo nextest run --locked grapheme_cluster_mode_renders_zwj_family_in_single_wide_cell
```

## 0002 add ghostty_terminal_clear_screen C API

status: active

patch: `vendor/patches/libghostty-vt/0002-add-terminal-clear-screen-c-api.patch`

herdr issue: not opened (private fork, no upstream tracking)

upstream discussion: not opened; libghostty-vt exposes `eraseDisplay`/
`eraseHistory` on `Terminal`/`Screen` but no C API to invoke them directly
without going through the VT byte parser

upstream pr: not opened

vendored base: `c5a21edfcbc2d5b46540ad91b7980aca31f5f1f3`

local files:

- `vendor/libghostty-vt/src/terminal/c/terminal.zig`
- `vendor/libghostty-vt/src/terminal/c/main.zig`
- `vendor/libghostty-vt/src/lib_vt.zig`
- `vendor/libghostty-vt/include/ghostty/vt/terminal.h`

reason: Herdr needs to clear a pane's primary-screen content and
scrollback from outside the VT byte stream (a keybind-triggered action,
not something the remote/local shell sent), without moving the cursor or
resetting colors/modes the way `ghostty_terminal_reset` does. The new
`ghostty_terminal_clear_screen` function always targets the primary
screen directly (not whichever screen is active), so it is also safe to
call while an alternate-screen program (vim, tmux) is running.

remove when: libghostty-vt exposes an equivalent C API for a
primary-screen-targeted erase-display-and-history operation upstream, and
the herdr-side clear_screen action's tests pass against it unmodified.

verification:

```sh
cargo nextest run --locked clear_screen
```

## 0003 add cursor_is_at_prompt and cursor_home C API

status: active

patch: `vendor/patches/libghostty-vt/0003-add-cursor-is-at-prompt-and-cursor-home-c-api.patch`

herdr issue: not opened (private fork, no upstream tracking)

upstream discussion: not opened; `Terminal.cursorIsAtPrompt()` already
exists upstream (used internally by real Ghostty's own `clear_screen`
keybind action in `src/termio/Termio.zig`) but is not exposed through the
C API, and there is no C API for repositioning the primary screen's
cursor outside the VT byte stream either.

upstream pr: not opened

vendored base: `c5a21edfcbc2d5b46540ad91b7980aca31f5f1f3`

local files:

- `vendor/libghostty-vt/src/terminal/c/terminal.zig`
- `vendor/libghostty-vt/src/terminal/c/main.zig`
- `vendor/libghostty-vt/src/lib_vt.zig`
- `vendor/libghostty-vt/include/ghostty/vt/terminal.h`

reason: herdr's `clear_screen` action needs to match real Ghostty's own
Cmd+K behavior (clear, home cursor, and nudge the shell to redraw its
prompt) instead of leaving the cursor stranded with no prompt visible.
Real Ghostty only homes the cursor and writes a form-feed byte to the
pty when `cursorIsAtPrompt()` says the cursor is at an idle,
shell-integration-reported prompt (see `Termio.clearScreen` in the real
Ghostty source) -- otherwise it risks injecting a control byte into
whatever is actually reading the pty (a running command, or a fullscreen
program in the alternate screen). `ghostty_terminal_cursor_is_at_prompt`
exposes that same upstream check so herdr can gate its own prompt-redraw
the same way. `ghostty_terminal_cursor_home` performs the actual
reposition, always targeting the primary screen like
`ghostty_terminal_clear_screen` does.

remove when: libghostty-vt exposes equivalent C API functions for
`cursorIsAtPrompt()` and a primary-screen cursor-home operation upstream,
and the herdr-side clear_screen tests pass against them unmodified.

verification:

```sh
cargo nextest run --locked pane_clear_screen
```
