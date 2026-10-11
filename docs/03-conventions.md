# Conventions

## Units

Time is in **ticks**, 60 per second. A duration of `9` means nine ticks, not
milliseconds.

Space is in **pixels** of the 320x180 render target. Positions are `i32`,
remainders and velocities are `f32`. Velocity is in pixels per tick. A tile is
8 pixels. See "Simulation units" in `01-architecture.md`.

Tuning constants are written in these units, with a comment giving the human
reading where it helps: `JUMP_HEIGHT: f32 = 26.0; // about 3 tiles`.

## The input model

Each tick, each player's input is one small, fixed size, `Copy` value. Buttons
are a bitmask:

    bit 0   left
    bit 1   right
    bit 2   up
    bit 3   down
    bit 4   jump
    bit 5   dash
    bit 6   grapple
    bit 7   interact, also confirm in dialogue and choices
    bit 8   pause request (online vote to pause)
    ...     room to grow

Nine bits already, so buttons are a `u16`, not a `u8`.

The grapple uses the direction bits for its eight way aim, so it needs no extra
input. An `aim` byte is reserved for a future ability.

Systems read the input and derive everything else inside the simulation. "Ticks
since jump was pressed", used by jump buffering, is simulation state, not
something the client sends.

The simulation always takes input per player, even in the solo prequel. A
session is fully described by a seed plus each player's list of inputs, which
is what makes replays and rollback possible.

## Levels

LDtk project files live under `levels/`. Tile layers hold meaning (solid, hazard,
one way), never appearance. Appearance comes from auto-tiling rules.

Entity identifiers in LDtk use `PascalCase` (`Box`, `Door`, `Trigger`). Event
and fact ids used in entity fields use `snake_case` (`on_enter_lab`).

## Naming

Standard Rust style, enforced by `rustfmt` and `clippy`:

- Types and components: `PascalCase` nouns. `Position`, `Velocity`,
  `MotionState`
- Systems: `snake_case` verb phrases. `apply_gravity`, `resolve_collisions`
- Constants: `SCREAMING_SNAKE_CASE`, one file per concern for tuning values
- Ids are newtypes, not bare integers. `FactId(u32)`, not `u32`, so they cannot
  be confused with each other or with counts
- Group related systems into a Bevy plugin per feature: `MovementPlugin`,
  `DialoguePlugin`

## Testing

- **Unit tests** with `cargo test` for pure logic: collision steps, rule
  selection, fact scoping
- **Determinism**: GGRS SyncTest in development builds from Phase 2, and per
  tick state hashing from Phase 6
- **Presentation**: verified by looking at it

## Formatting and linting

`rustfmt` on save, `clippy` in the editor. Do not commit unformatted code. Treat
clippy warnings as worth reading: many of them teach idiomatic Rust.

## Git

Conventional commits.

    feat(movement): add coyote time to jump handling
    fix(collision): stop actors clipping into moving solids
    refactor(dialogue): split rule matching from fact storage
    docs: record the render resolution decision
    chore: bump bevy

Scopes are feature or crate names, or omitted for repo wide changes.

Commit small and often. This is a learning project, so the history is a record
of what you understood and when.

Large binary assets (art, audio, voice recordings) may need Git LFS once they
arrive. Decide then.

## Comments

Comment the **why**, not the what. The what is in the code.

Tuning constants and anything determinism related deserve a comment explaining
the constraint. In six months the reason a `BTreeMap` was used instead of a
`HashMap`, or `libm` instead of `f32::sin`, will not be obvious, and someone
(you) will helpfully simplify it.
