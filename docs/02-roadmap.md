# Roadmap

Content led and always playable. Each phase has an exit criterion that must be
true before moving on. From Phase 1 onward there is always something a friend
can play.

The shape follows the lessons in `00-vision.md`: mechanics are prototyped and
playtested early, content starts long before the systems feel finished, and
networking waits until there is a game worth networking.

Two side tracks run alongside every phase, at whatever pace suits:

- **Art.** Learn Aseprite with small exercises: a tile, a character idle, a run
  cycle. None of it blocks the game
- **Story.** Parked. Ideas can be added to the parked section of
  `00-vision.md` whenever they come up, but no system waits on them

---

## Phase 0: a square that moves, in Rust

- The Rust Book, chapters 1 to 11 and 13, and Rustlings through `18_iterators`.
  Chapter 19 (patterns) during Phase 1. Smart pointers, threads and macros
  wait until something needs them
- Build for the browser once (WASM), to prove the pipeline before it matters

The steps, in order:

1. [x] Editor set up for Rust: rust-analyzer with clippy, rustfmt and taplo
       formatting on save
2. [x] A Cargo workspace with `crates/sim` and `crates/game`, Bevy pinned at
       `=0.19.1`, and `cargo tree -p sim` showing no rendering, window or audio
       crates
3. [ ] A window and a coloured rectangle, in `game` only
4. [ ] The square moves: a `SimPlugin` in `sim` running in `FixedUpdate` at
       60Hz, keyboard input collected per frame in `game` and consumed per tick
5. [ ] Gamepad input through a small action mapping layer, into one input
       value per player
6. [ ] A `clippy.toml` in `crates/sim` banning wall clocks, `HashMap`,
       `HashSet` and `std` trigonometry
7. [ ] A WASM build running in the browser (wasm-server-runner)

**Exit:** the square moves left and right at a constant speed, identically at
60Hz and 144Hz, natively and in the browser.

---

## Phase 1: movement that feels good

The longest phase, and the one that decides whether the game is any good.

- Load a level from LDtk: solid tiles and a spawn point
- Hand written actor and solid collision, Celeste style
- Gravity, jumping, `MotionState`
- The feel pass: coyote time, jump buffering, variable jump height, asymmetric
  rise and fall gravity, corner correction, acceleration and friction
- Dash
- All tuning constants hot reloaded
- A debug inspector

**Exit:** jumping and dashing feel good, not just work. A friend plays a test
level and you watch without explaining anything.

---

## Phase 2: two characters, local co-op

Two players on one machine before any networking. It is the real game, and the
simulation already takes input per player, so it costs little. Testing alone
works with a keyboard and a gamepad.

Each mechanic is a prototype first. It earns its place in the game by being fun
in a playtest, not by being finished.

- Two characters, each driven by its own input device, both through the per
  player input path
- Split screen, one camera per player
- The shared co-op moves: boost and pull up
- The first split tool pair: throw and glide. Others from `00-vision.md` as
  playtests ask for them
- Pushable box
- One interactable that opens its own small interface
- One non combat enemy, such as a patrol you must avoid
- A first sfxr style sound generator, enough for jump, land, dash and tool
  cues, so feel can be judged with sound
- A GGRS SyncTest session running in development builds, so determinism bugs
  show up now instead of during networking
- A few rough test levels, one per mechanic

**Exit:** two friends have played every prototype on one machine. A written
keep, change or cut list exists, and you have acted on it.

---

## Phase 3: a demo level with dialogue

The dialogue system's target. A few rooms, two or three NPCs, a small
self-contained situation, no plot. Built for one player first.

- The fact store, events and rules
- Rules authored in data files and hot reloaded
- A dialogue box for stop and talk scenes, and lines that play while you move
- An NPC who reacts to something you actually did
- One choice that sets a fact which changes something later in the level
- Saving and loading facts
- Voice: a few lines recorded with friends, looked up by line id, with a babble
  voice and then text as fallbacks
- Deployed to the web as a playtest link, for example itch.io

**Exit:** a friend plays the demo in a browser, an NPC reacts to what they
actually did, and they hear at least one recorded line.

---

## Phase 4: co-op levels, ugly and complete

- Outline a short set of co-op levels before building any: sets, then rooms,
  then the mechanic each one teaches
- Build every room in rough form. Shapes, no art
- Same room and cross room puzzles
- Dialogue working for two players, each with their own dialogue slot
- Ping and countdown
- Iterate on the levels from playtests

**Exit:** two people finish the whole set together on one machine.

---

## Phase 5: online co-op

- State hashing per tick and a desync report
- GGRS over matchbox, with a signaling server deployed
- A lobby: create and join by code
- Each player on their own screen, with per player visibility

**Exit:** two people on different machines finish the level set without
desyncing.

---

## Phase 6: shipped on the web

- The low resolution render target and camera, finished
- Palettes per level set and per player, and the palette swap shader
- Sprites and animation for the players and key objects
- Juice: screenshake, squash and stretch, particles, easing
- Music, sound, and voice lines
- Menus, settings, saves
- Deploy to the web

**Exit:** the target in `00-vision.md`. A link two friends open, and they
finish the game together online.

---

## Later, maybe

- The story, from the parked section of `00-vision.md`
- A Steam build: native packaging, Steamworks, possibly Steam networking
- More levels

---

## Stretch goals

Any, or none, whenever they fit:

- A WebSocket relay transport through the lobby Durable Object, measured
  against TURN

- Lighting and custom WGSL shaders
- aarthificial's UV lookup animation
- Procedural art and effects
- Replay of your best run per level

---

## The rule that keeps this on track

> Build a system when a level needs it. Build the simplest version that works.
> Upgrade it only when a playtest, a measurement, or a concrete level forces it.

The failure mode for this project is not a missing feature. It is two years of
excellent systems and nothing to play.
