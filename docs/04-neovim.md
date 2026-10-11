# Working in this repo with neovim

Written for someone new to both vim and neovim, against the config at
`~/dotfiles/config/nvim` (kickstart.nvim on `vim.pack`, with oil).

**If you have never used vim at all, run `:Tutor` first.** It is a built-in
interactive tutorial, about 30 minutes, covering modes, `hjkl`, `dw`, undo,
search and `:w`. If you have already done `vimtutor`, skip it: the lessons are
the same material and there is nothing new in the neovim port.

What `vimtutor` does not cover is the tier that actually makes vim fast, and
this document assumes you are past the basics. In rough order of value:

1. **Buffers, windows and tabs**, and the fact that they are three different
   things. See section 8.
2. **Text objects**: `ciw`, `ci"`, `ci(`, `cif`. The single biggest jump in
   editing speed after the basics.
3. **The `.` command**, which repeats your last change. Pairs with text objects.
4. **Marks and registers**.
5. **Macros**, `qq` to record and `@q` to replay.
6. **The quickfix list**, and `:cdo` to apply an edit across every entry.
7. **`:h`**, the built-in manual, which is genuinely excellent.

Section 9 schedules these against the phases of the project rather than asking
you to learn them from a list.

---

## 1. How to read a keybinding

Two different kinds of thing get written the same way:

| Written        | Means                            | Kind                     |
| -------------- | -------------------------------- | ------------------------ |
| `<C-o>`        | Hold **Ctrl**, press **o**       | Chord, keys together     |
| `<C-v>`        | Hold **Ctrl**, press **v**       | Chord                    |
| `<CR>`         | **Enter**                        | Single key               |
| `<Esc>`        | **Escape**                       | Single key               |
| `<leader>`     | **Space** in this config         | Single key               |
| `grd`          | **g**, then **r**, then **d**    | Sequence, one after another |
| `<leader>sf`   | **Space**, then **s**, then **f**| Sequence                 |

Sequences are typed in order like a word, with about a second of timeout. Type
them at normal speed.

`<leader>` is a prefix key so that config shortcuts do not collide with vim's
built-ins. It is set to Space at line 98 of `init.lua`.

## 2. Modes

Keys mean different things depending on mode.

- **Normal** is the default. Keys are commands. `d` deletes, `g` starts a
  sequence. Most of your time is here.
- **Insert** (`i`) types characters like a normal editor. `<Esc>` leaves it.
- **Visual** (`v`, or `<C-v>` for block) selects text.
- **Command** (`:`) is for `:w`, `:q` and friends.

Everything in this document is normal mode unless it says otherwise. If a
shortcut appears to do nothing, you are probably in insert mode. Press `<Esc>`.

## 3. What an LSP is

**Language Server Protocol.** A language server is a separate background program
that genuinely understands a language. It parses the project, models every type
and import, and answers questions about it.

Neovim does not understand TypeScript, it displays text. When you press `grd`,
neovim messages the server: "the cursor is at line 40 column 12 of this file,
where is that defined?" The server answers with a path and position, and neovim
opens it.

It exists because before LSP, every editor implemented every language
separately. Ten editors times twenty languages is two hundred implementations.
LSP makes it ten plus twenty.

In this setup:

- **`vtsls`** is the TypeScript server. It wraps `tsserver`, the same engine
  VS Code uses, so the intelligence is identical.
- **`biome`** is a second server handling formatting and lint.
- **Mason** installs them. `:Mason` to see what is there.

One server produces all of these: red squiggles (**diagnostics**), autocomplete,
`grd` and friends, hover docs, inlay hints, and code actions.

Worth connecting: the server reads each package's `tsconfig.json`. That is
exactly why omitting `DOM` from `engine`'s tsconfig turns `document` into a red
squiggle there but not in `client`. The architecture rule in
`01-architecture.md` is enforced by the same thing that gives you autocomplete.

### vtsls in a monorepo

Root detection is `root_pattern('tsconfig.json', 'package.json', 'jsconfig.json', '.git')`,
which finds the nearest ancestor. So you get **one vtsls process per package**,
three or four in total.

That is correct and wanted. It is how each package's own tsconfig gets applied,
which is what makes the `lib` guard work. Cross-package go-to-definition still
works, because pnpm symlinks the packages and their `exports` point at source.

If you ever see a single vtsls rooted at the repo root, a package tsconfig is
missing or malformed.

## 4. Navigating code: the `gr` family and the jumplist

Neovim reserves the `gr` prefix for language-server actions. Think "**g**o to
**r**elated thing".

| Keys  | Does                                                        |
| ----- | ----------------------------------------------------------- |
| `grd` | **Go to definition**                                        |
| `grr` | **Find references**, every place it is used                 |
| `grn` | **Rename** it safely across the whole project               |
| `gra` | **Code action**, offers fixes such as auto-import           |
| `gri` | Go to implementation                                        |
| `grt` | Go to type definition                                       |
| `K`   | Hover: show type and documentation                          |
| `gO`  | Symbols in this file                                        |
| `gW`  | Symbols across the project                                  |

Example. In `packages/game/src/systems/movement.ts` you see:

    const world = createWorld(1024)

Put the cursor on `createWorld`, press `grd`, and neovim opens
`packages/engine/src/ecs/world.ts` at the definition. Different file, different
package, no path typed. That is what makes a monorepo navigable.

### `<C-o>` and `<C-i>`

`<C-o>` is Ctrl+o. It means "go back to where I just was".

Vim keeps a **jumplist**, a history of significant jumps: `grd`, `/` searches,
`G`, opening a file from telescope. It is exactly your browser's back and
forward buttons.

- `<C-o>` back one jump
- `<C-i>` forward again
- `:jumps` to see the list

Small moves like `j`, `k`, `w` are not recorded, so the list stays useful.

### The loop that is actually how you read code

    grd      jump to where this is defined
    K        read its type and docs
    <C-o>    come back

Follow a chain three or four levels deep, then `<C-o>` a few times to unwind. It
is a stack. Once this is automatic, unfamiliar code stops being intimidating,
and it is the main reason a modal editor is pleasant on a large project.

## 5. oil: files are a buffer you edit

`-` opens the parent directory of the current file, cursor on that file. Inside
oil, `-` goes up another level, so it is one key all the way out of the tree.
`<leader>e` opens oil in a floating window instead.

The whole idea: **the directory is a text buffer**. Editing the text is the file
operation. Nothing touches disk until `:w`, and oil shows a confirmation of
exactly what it is about to do. A mistake before `:w` is just `u`.

`g?` inside oil shows all of its keybindings.

### Creating a file

1. `-` to open the directory
2. `o` to open a new line below the cursor (this puts you in insert mode)
3. Type the filename, for example `movement.ts`
4. `<Esc>` then `:w`
5. Confirm the preview

### Creating a directory

Same, but end the name with a slash: `systems/`

### Creating nested paths in one go

Type the whole path on the line: `systems/movement/walk.ts`. Intermediate
directories are created for you. Check the confirmation preview to be sure it is
doing what you meant.

### Renaming

Edit the text of the line and `:w`. `cw` over the name, type the new one.

Your config has `lsp_file_methods` enabled, so **renaming a file updates every
import that points at it**. In a monorepo that is a big deal, and most people do
not have it turned on.

### Deleting

`dd`, then `:w`. Your config sets `delete_to_trash = true`, so it goes to the
system trash and is recoverable. `dd` alone changes nothing until you write.

### Moving and copying

`dd` in one oil buffer, then `p` in another oil buffer at a different directory.
That is a move. `yy` and `p` is a copy. Ordinary vim editing, applied to the
filesystem, which is the entire point of oil.

### Creating a file without oil

`:e packages/game/src/thing.ts` then `:w` works, but only if every directory in
the path already exists. Oil is better when directories are being created.

## 6. Git

Use all three of these, for different things.

### In the buffer: gitsigns, which you already have

This is the part that is genuinely better in the editor than in a terminal,
because you are staging code while reading it with full LSP context.

| Keys          | Does                                        |
| ------------- | ------------------------------------------- |
| `]c` / `[c`   | Jump to next / previous changed hunk        |
| `<leader>hp`  | Preview the hunk under the cursor           |
| `<leader>hs`  | **Stage** this hunk (works on a visual selection too) |
| `<leader>hr`  | **Reset** this hunk, discarding it          |
| `<leader>hS`  | Stage the whole buffer                      |
| `<leader>hb`  | Blame this line                             |
| `<leader>hd`  | Diff this file against the index            |
| `<leader>tb`  | Toggle inline blame for every line          |

The review workflow: `]c` to the next change, `<leader>hp` to look at it,
`<leader>hs` to stage it or `<leader>hr` to throw it away. Repeat to the bottom
of the file. That is `git add -p`, except you can see the surrounding code and
jump to definitions while deciding.

### Committing: lazygit, in a tmux window

You already have `lazygit` installed. Run it in tmux window 3. It is a visual
TUI: `space` stages, `c` commits, `P` pushes, `?` shows help. For someone new to
git it is more discoverable than the CLI and it shows you what git is actually
doing.

`git commit` from the command line also works, and if `$EDITOR` is set to `nvim`
you get neovim for the commit message, with the diff visible below. Worth
setting in your shell config if it is not already:

    export EDITOR=nvim

### Why not a full git plugin in neovim

`vim-fugitive` and `neogit` are both good and both put the whole of git inside
the editor. They also each have their own vocabulary to learn on top of vim's,
and you are already learning enough. gitsigns plus lazygit covers everything
with nothing new to memorise. Revisit fugitive in a few months if the split
starts to annoy you.

## 7. A mode for editing level files

ASCII level grids want different settings from code. The important one is
`virtualedit`, without which you cannot place a character to the right of a
short line, which makes grid editing impossible.

Add this to `~/dotfiles/config/nvim/lua/custom/plugins/` as `levels.lua`:

    -- Level files for the Shared Ground project are ASCII grids. Editing a grid
    -- wants different settings from editing prose or code, mostly so that the
    -- cursor can move into the empty space to the right of a short line.
    vim.filetype.add {
      pattern = {
        ['.*/levels/.*%.txt'] = 'sglevel',
      },
    }

    vim.api.nvim_create_autocmd('FileType', {
      pattern = 'sglevel',
      callback = function()
        -- Move the cursor anywhere, including past the end of a line. Without
        -- this you cannot place a block to the right of a short row.
        vim.opt_local.virtualedit = 'all'
        -- Show the current column, so vertical alignment across the grid is
        -- visible while editing.
        vim.opt_local.cursorcolumn = true
        vim.opt_local.wrap = false
        vim.opt_local.list = false
        vim.opt_local.expandtab = false
      end,
    })

### The technique this unlocks

**Visual block mode**, `<C-v>`. It selects a rectangle rather than lines, which
is exactly the shape of a tile grid operation.

| Keys                     | Does                                          |
| ------------------------ | --------------------------------------------- |
| `<C-v>` then `j`/`k`     | Extend the block down or up                   |
| `<C-v>` block then `r#`  | Replace every character in the block with `#` |
| `<C-v>` block then `d`   | Delete a column of the grid                   |
| `<C-v>` block then `I`   | Insert text at the start of every line, `<Esc>` to apply |
| `<C-v>` block then `A`   | Append at the end of every line               |
| `r` in normal mode       | Replace one character without entering insert |

Building a vertical wall is `<C-v>`, `10j`, `r#`. Editing a platformer level in
visual block mode is one of the genuinely satisfying things vim does.

## 8. tmux layout

Your tmux prefix is `C-a` and pane navigation is `prefix h/j/k/l`. Neovim uses
bare `<C-h/j/k/l>` for windows. No collision.

    window 1   nvim, full screen, splits inside nvim
    window 2   split: `pnpm dev` above, `pnpm test --watch` below
    window 3   lazygit and shell

Use neovim's own splits for editing and keep tmux for long-running processes.
Putting editors in tmux panes is where the two sets of muscle memory start
fighting each other.

Inside neovim: `:vsplit` for a vertical split, `<C-w>` then `hjkl` to move
between them (or your `<C-h/j/k/l>` mappings), `<C-w>q` to close one.

## 9. What to practise, by phase

Learning `:cdo` in week one is pointless. Learning it the week you rename a
component across forty files makes it stick.

### Phase 0, writing new files

- oil for creating files and directories
- `<leader>sf` find files, `<leader><leader>` switch buffers
- `grd` then `<C-o>`, until it is automatic
- Insert-mode discipline: no arrow keys, `<Esc>` the moment you stop typing
- `:w` constantly

### Phase 1, tuning feel constants

This phase is hundreds of small edits to one file, which makes it the best
possible drill.

- `ciw` change inner word, then `.` to repeat on the next one
- `f` and `t` to jump to a character on the line, `;` to repeat
- **`<C-a>` and `<C-x>` increment and decrement the number under the cursor.**
  Made for exactly this. Tuning gravity becomes holding a key.
- `10<C-a>` adds ten
- `mm` to set mark `m`, backtick-`m` to jump back to it

### Phase 1, authoring levels

- `<C-v>` visual block, `r`, `I`, `A`
- `virtualedit` from section 7

### Phase 2 to 3, multi-file work

- `<leader>sg` live grep across the project
- `grn` LSP rename
- `<leader>q` diagnostics into the quickfix list
- `:cnext` and `:cprev` to walk the quickfix list
- `<leader>sr` resume the last telescope search

### Phase 4, the ECS rewrite

- Macros: `qq` to record into register `q`, `q` to stop, `@q` to play, `@@` to
  repeat, `10@q` to run it ten times
- `:%s/old/new/gc` search and replace with confirmation
- `:cdo s/old/new/g | update` to apply an edit to every quickfix entry
- mini.ai text objects: `cif` change a function body, `vaf` select a function

### Always

- `<C-o>` and `<C-i>`
- Press `<leader>` and wait when stuck: which-key lists everything
- `<leader>sk` to search every keymap by description
- `:h <topic>` for the manual, which is excellent. `<C-o>` works inside help too

## 10. Quick reference

| Keys           | Does                                    |
| -------------- | --------------------------------------- |
| `-`            | oil, parent directory                   |
| `<leader>e`    | oil in a float                          |
| `<leader>sf`   | Find files                              |
| `<leader>sg`   | Grep the project                        |
| `<leader><leader>` | Switch buffer                       |
| `<leader>sk`   | Search keymaps                          |
| `<leader>sh`   | Search help                             |
| `<leader>sr`   | Resume last search                      |
| `grd`          | Go to definition                        |
| `grr`          | Find references                         |
| `grn`          | Rename                                  |
| `gra`          | Code action                             |
| `K`            | Hover docs                              |
| `<C-o>` `<C-i>`| Jump back and forward                   |
| `<leader>q`    | Diagnostics to quickfix                 |
| `<leader>f`    | Format buffer                           |
| `<leader>th`   | Toggle inlay hints                      |
| `]c` `[c`      | Next and previous git hunk              |
| `<leader>hs`   | Stage hunk                              |
| `<leader>hp`   | Preview hunk                            |
| `<C-v>`        | Visual block mode                       |
| `<C-a>` `<C-x>`| Increment and decrement a number        |
