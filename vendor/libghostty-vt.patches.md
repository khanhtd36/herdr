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

## 0002 add prompt-aware terminal clear_screen C API

status: active

patch: `vendor/patches/libghostty-vt/0002-add-terminal-clear-screen-c-api.patch`

herdr issue: not opened (private fork, no upstream tracking)

upstream discussion: not opened; libghostty-vt exposes `eraseDisplay`/
`eraseHistory`/`eraseActive` on `Terminal`/`Screen` and
`Terminal.cursorIsAtPrompt()` (already used internally by real Ghostty's
own `clear_screen` keybind action in `src/termio/Termio.zig`), but no C
API to invoke any of them directly without going through the VT byte
parser.

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
own Cmd+K behavior when doing so: clear and nudge the shell to redraw
its prompt, instead of leaving the cursor stranded with no prompt
visible. `ghostty_terminal_clear_screen` always erases scrollback, and
clears the whole active screen only when `cursorIsAtPrompt()` says the
cursor is at an idle, shell-integration-reported prompt -- otherwise it
erases only rows above the cursor via `Screen.eraseActive`, exactly as
real Ghostty's own `Termio.clearScreen` does, so the screen isn't left
blank with a stranded cursor when there's no shell integration to
redraw a prompt afterward. `eraseActive` physically removes those rows
and shifts the survivors up, which is what actually lands the shell's
prompt at the top left without writing anything to the pty; an in-place
clear such as `Screen.clearRows` erases the same cells but leaves the
prompt stranded at its old row, so it is not a valid substitute.
`eraseActive` does, however, mark only the shifted rows dirty and not
the rows regrown at the bottom to refill the active area, which leaves
a differential renderer such as herdr's painting stale content below
the prompt; this patch therefore marks the whole active area dirty
after the erase. It returns that same prompt determination so
callers can decide whether to send a form-feed byte to the pty: sending
that byte unconditionally would risk injecting a control byte into
whatever is actually reading the pty (a running command, or a
fullscreen program in the alternate screen), which real Ghostty also
avoids by gating on the same check. `ghostty_terminal_clear_screen`
always targets the primary screen directly (not whichever screen is
active), so it is also safe to call while an alternate-screen program
(vim, tmux) is running. It never touches cursor position itself, in
either branch, matching real Ghostty's own clear_screen action exactly:
repositioning is left entirely to the shell's own response to the form
feed (zle/readline's clear-screen widget), since a local reposition
first -- before that response arrives -- would silently move the
cursor without going through the pty, leaving the shell's own
relative-scroll math (computed against what it still thinks the cursor
row is) to fight the mismatch instead of landing where it should. An
earlier version of this patch added a separate
`ghostty_terminal_cursor_home` and called it before sending the form
feed; that was removed once this exact mismatch was diagnosed as the
cause of the redrawn prompt landing at the wrong row. In the
not-at-prompt branch no repositioning is needed at all, because
`eraseActive`'s row shift carries the cursor to the top on its own.

remove when: libghostty-vt exposes an equivalent C API function for a
primary-screen-targeted, prompt-aware erase-display-and-history
operation upstream, and the herdr-side clear_screen action's tests pass
against it unmodified.

verification:

```sh
cargo nextest run --locked pane_clear_screen
```
