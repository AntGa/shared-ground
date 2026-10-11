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
- **Story.** Develop the story and characters in sparring sessions as the
  mechanics settle. Writing it all up front is not the goal

---

## Phase 0: a square that moves, in Rust

- The Rust Book, chapters 1 to 10, and Rustlings, alongside the code below
  rather than before it
- Create a Cargo workspace with a pinned Bevy
- A window, a coloured rectangle, keyboard and gamepad input read into a per
  player input value on a fixed timestep, through a small action mapping layer
- Build for the browser once (WASM), to prove the pipeline before it matters

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

## Phase 2: prototype the mechanics

Each mechanic is a prototype first. It earns its place in the game by being fun
in a playtest, not by being finished.

- Grappling hook
- Pushable box
- One interactable that opens its own small interface
- One non combat enemy, such as a patrol you must avoid
- A first sfxr style sound generator, enough for jump, land, dash and grapple
  cues, so feel can be judged with sound
- A GGRS SyncTest session running in development builds, so determinism bugs
  show up now instead of during networking
- A few rough test levels, one per mechanic

**Exit:** a friend has played every prototype. A written keep, change or cut
list exists, and you have acted on it.

---

## Phase 3: the story spine

Starts with a dedicated story and abilities sparring session.

- The fact store, events and rules
- Rules authored in data files and hot reloaded
- A dialogue box for stop and talk scenes, and lines that play while you move
- One choice that sets a fact which a later rule reacts to
- Saving and loading facts
- Abilities gated by facts

**Exit:** an NPC who reacts to what you actually did, and a choice whose
consequence you can see later.

---

## Phase 4: the prequel, ugly and complete

- Outline the whole prequel top down: areas, then rooms, then beats
- Build every room in rough form. Shapes, placeholder text, no art
- Fill in dialogue as rough drafts
- Iterate on the levels from playtests

**Exit:** someone plays the greybox prequel from start to finish.

---

## Phase 5: the prequel, shipped on the web

- Decide and build the low resolution render target and camera
- Sprites and animation for the player and key objects
- Juice: screenshake, squash and stretch, particles, easing
- Music, sound, and the first voice lines from friends
- Menus, settings, saves
- Deploy to the web, for example itch.io

**Exit:** a link you can send someone, and they finish the prequel.

---

## Phase 6: co-op

- State hashing per tick and a desync report
- GGRS over matchbox, with a signaling server deployed
- A lobby: create and join by code
- Two cameras, one per player, and per player visibility
- Same room and cross room puzzles

**Exit:** two people on different machines finish a co-op level without
desyncing.

---

## Phase 7: the co-op campaign and Steam

- The main story and its levels, built the same way as Phase 4 and 5
- A Steam build: native packaging, Steamworks, possibly Steam networking

**Exit:** Target 2 from `00-vision.md`.

---

## Stretch goals

Any, or none, whenever they fit:

- A WebSocket relay transport through the lobby Durable Object, measured
  against TURN

- Lighting and custom WGSL shaders
- aarthificial's UV lookup animation
- Procedural art and effects
- Replay of your best run per level
- Parsing LDtk JSON directly instead of using a crate

---

## The rule that keeps this on track

> Build a system when a level needs it. Build the simplest version that works.
> Upgrade it only when a playtest, a measurement, or a concrete level forces it.

The failure mode for this project is not a missing feature. It is two years of
excellent systems and nothing to play.
