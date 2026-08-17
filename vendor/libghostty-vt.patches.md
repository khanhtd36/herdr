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

## 0002 add prompt-aware terminal clear_screen and cursor_home C API

status: active

patch: `vendor/patches/libghostty-vt/0002-add-terminal-clear-screen-c-api.patch`

herdr issue: not opened (private fork, no upstream tracking)

upstream discussion: not opened; libghostty-vt exposes `eraseDisplay`/
`eraseHistory`/`eraseActive` on `Terminal`/`Screen` and
`Terminal.cursorIsAtPrompt()` (already used internally by real Ghostty's
own `clear_screen` keybind action in `src/termio/Termio.zig`), but no C
API to invoke any of them directly without going through the VT byte
parser, and no C API for repositioning the primary screen's cursor
either.

upstream pr: not opened

vendored base: `c5a21edfcbc2d5b46540ad91b7980aca31f5f1f3`

local files:

- `vendor/libghostty-vt/src/terminal/c/terminal.zig`
- `vendor/libghostty-vt/src/terminal/c/main.zig`
- `vendor/libghostty-vt/src/lib_vt.zig`
- `vendor/libghostty-vt/include/ghostty/vt/terminal.h`

reason: Herdr needs to clear a pane's primary-screen content and
scrollback from outside the VT byte stream (a keybind-triggered action,
not something the remote/local shell sent), and to match real Ghostty's
own Cmd+K behavior when doing so: clear, home the cursor, and nudge the
shell to redraw its prompt, instead of leaving the cursor stranded with
no prompt visible. `ghostty_terminal_clear_screen` always erases
scrollback, and clears the whole active screen only when
`cursorIsAtPrompt()` says the cursor is at an idle, shell-integration-
reported prompt -- otherwise it erases only rows above the cursor in
place (VT100 ED1 semantics via `Screen.clearRows`, not
`Screen.eraseActive` as real Ghostty's own `Termio.clearScreen` uses
for this fallback: `eraseActive` physically removes and shifts pages,
and does not reliably erase the full requested range once the active
area spans more than one internal page, e.g. a still-growing buffer
such as a fresh SSH session's login banner that hasn't yet triggered
real scrollback), so the screen isn't left blank with a stranded
cursor when there's no shell integration to redraw a prompt afterward.
It returns that same prompt determination so callers can decide whether
to also home the cursor and send a form-feed byte to the pty: sending
that byte unconditionally would risk injecting a control byte into
whatever is actually reading the pty (a running command, or a
fullscreen program in the alternate screen), which real Ghostty also
avoids by gating on the same check. `ghostty_terminal_clear_screen`
always targets the primary screen directly (not whichever screen is
active), so it is also safe to call while an alternate-screen program
(vim, tmux) is running. `ghostty_terminal_cursor_home` performs the
actual cursor reposition, always targeting the primary screen the same
way.

remove when: libghostty-vt exposes equivalent C API functions for a
primary-screen-targeted, prompt-aware erase-display-and-history
operation and a primary-screen cursor-home operation upstream, and the
herdr-side clear_screen action's tests pass against them unmodified.

verification:

```sh
cargo nextest run --locked pane_clear_screen
```
