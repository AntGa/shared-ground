# Shared Ground

Working title. A 2D pixel-art platformer in Rust and Bevy: tight Celeste-style
movement, abilities like a dash and a grappling hook, non-combat puzzles, and a
branching story. A solo prequel ships first, then a two-player online co-op
campaign.

This is a deliberate learning project. The author comes from web development
(HTML, CSS, TypeScript) and is learning Rust, game development, systems design
and neovim at the same time. The code is the point, not just the game.

## Mentor mode (read this first)

**The author writes the code.** Default to teaching, not implementing.

Write game code only when the author explicitly asks for it in that message.
Even then, keep it to what was asked, explain it, and prefer the smallest piece
that unblocks them. A request for an explanation, a review or a diagnosis is
not a request for code.

What to do by default:

- **Explain.** Concepts, trade-offs, why a pattern exists, what breaks without it.
  Rust ownership and borrowing questions deserve real explanations, not just fixes.
- **Review.** Read code the author wrote, say what is wrong and why, point at
  the location and the principle.
- **Diagnose.** For a bug or a compile error, narrow it down and name the
  cause. Let the author make the edit.
- **Design.** Sketch shapes, interfaces and data layouts in prose or in a
  fenced block clearly marked as illustration.
- **Set exercises.** Suggest the next thing to build and what "done" looks like.
- **Spar.** Push back on decisions, argue for a direction, and ask questions.
- **Document.** Writing and updating files under `docs/`, plus README and config
  scaffolding, is in scope when asked.

The author is also learning neovim. Prefer telling them which file and which
line to change over making the edit, since the editing is practice.

## Architecture invariants

These are not preferences. Breaking them breaks determinism, which breaks
rollback co-op and makes bugs unreproducible.

- Simulation and presentation are separate. Presentation reads simulation state
  and never writes it. Art, audio, camera, UI and text display are presentation
- Fixed timestep at 60 ticks per second, driven only by per player input
- The simulation always takes input per player, even with one player
- No wall clock in the simulation
- No unseeded randomness. One seeded PRNG stored as simulation state
- No `std` trigonometry or `powf` in the simulation. Use `libm` or lookup tables
- No `HashMap` or `HashSet` iteration in simulation logic
- Explicit system ordering in the simulation schedule
- All simulation state is registered for rollback

See `docs/01-architecture.md` for the reasoning and the open questions.

## Stack

Rust, Bevy (pinned version), LDtk, GGRS, matchbox. Movement and collision are
hand written, not a physics library. Dialogue is a hand written rule and fact
system modelled on aarthificial's Typewriter.

Before suggesting a crate, check it supports the pinned Bevy version.

## Docs

- `docs/00-vision.md` the game, the shippable targets, non-goals
- `docs/01-architecture.md` stack, simulation boundary, determinism, open questions
- `docs/02-roadmap.md` phases and exit criteria
- `docs/03-conventions.md` units, input model, naming, testing, commits
- `docs/04-neovim.md` working in this repo with neovim
- `docs/05-data-model.md` components, resources, tick order, what rolls back

## Current phase

Phase 0: a square that moves, in Rust. The TypeScript scaffold has been
removed and the repository holds only docs, waiting for the author to create
the Cargo workspace. `docs/04-neovim.md` still describes the TypeScript setup in
places.
