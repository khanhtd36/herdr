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

## 0003 add shell_redraws_prompt C API setter

status: active

patch: `vendor/patches/libghostty-vt/0003-add-shell-redraws-prompt-setter.patch`

herdr issue: not opened (private fork, no upstream tracking)

upstream discussion: not opened; libghostty-vt already implements the
behavior in `Terminal.resize` (which passes
`flags.shell_redraws_prompt` to `Screen.resize` as `prompt_redraw`),
but the C API forces the flag to `.false` in `terminal_new` and exposes
no way to set it, so embedders can only reach it from inside the VT
byte stream via `OSC 133;A;redraw=`.

upstream pr: not opened

vendored base: `c5a21edfcbc2d5b46540ad91b7980aca31f5f1f3`

local files:

- `vendor/libghostty-vt/src/terminal/c/terminal.zig`
- `vendor/libghostty-vt/src/terminal/c/main.zig`
- `vendor/libghostty-vt/src/lib_vt.zig`
- `vendor/libghostty-vt/include/ghostty/vt/terminal.h`

reason: A shell that marks its prompt with OSC 133 redraws that prompt
after every SIGWINCH, moving up one row and erasing to end of screen.
When the prompt line fills the terminal width -- any right-aligned
prompt, such as starship's `$fill` -- reflow rewraps it onto two rows,
so that erase misses the top row and orphans a copy of the prompt. Real
Ghostty never shows this because it runs with `shell_redraws_prompt`
true and therefore clears marked prompt lines before reflowing; herdr,
going through the C API, got `.false` and the orphans. A 15-step
drag-resize left 7 stacked prompts instead of 1. The clear stays gated
inside the core on the cursor not being on command output, so shells
without OSC 133 shell integration are unaffected either way.

remove when: libghostty-vt exposes its own C API for configuring
`shell_redraws_prompt` (or stops forcing it to `.false` in
`terminal_new`), and the verification below passes without this patch.

verification:

```sh
cargo nextest run --locked resize_clears_marked_prompt_so_shell_redraw_does_not_stack
```

## 0004 return the topmost prompt continuation from promptIterator left_up

status: active

patch: `vendor/patches/libghostty-vt/0004-prompt-iterator-left-up-topmost-continuation.patch`

herdr issue: not opened (private fork, no upstream tracking)

upstream discussion: not opened

upstream pr: not opened

vendored base: `c5a21edfcbc2d5b46540ad91b7980aca31f5f1f3`

local files:

- `vendor/libghostty-vt/src/terminal/PageList.zig`

reason: `PromptIterator.nextLeftUp` walks up from a `.prompt_continuation`
row looking for the `.prompt` row that anchors the prompt. When it finds a
`.none` row it correctly returns `end_pin`, the topmost continuation it
reached. When it instead runs out of rows -- the anchor scrolled out of
scrollback, or was never written -- it returned `p`, the row it started
from, discarding every continuation it had just walked over.

That fallback is what patch 0003's resize-time prompt clear lands on in
practice. zsh emits `OSC 133;A` only when it prints a *new* prompt, never
on the redraw it does for a SIGWINCH, so after the first drag-resize the
prompt's rows carry nothing but `.prompt_continuation` and the `.prompt`
anchor is gone. The clear then started at the cursor row, left every
prompt row above it standing, and reflow stranded them as visible
duplicates -- one more copy per resize, which is the exact stacking
patch 0003 was supposed to prevent. Verified against a verbatim replay of
a live zsh + starship pane: 19 drag steps produced 5 stacked prompts
before this patch and exactly 1 after.

remove when: upstream `nextLeftUp` returns the topmost continuation on the
exhausted-rows path, and the verification below passes without this patch.

verification:

```sh
cargo nextest run --locked resize_clears_marked_prompt_so_shell_redraw_does_not_stack
```
