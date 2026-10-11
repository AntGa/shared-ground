# Architecture

This document records what is decided and lists what is still open. Open
questions are marked **Open** and are settled in architecture sparring, then
moved into the decided sections with the reasoning.

## Stack

| Concern             | Choice                                        | Status                |
| ------------------- | --------------------------------------------- | --------------------- |
| Language            | Rust                                          | Decided               |
| Framework           | Bevy, version pinned                          | Decided               |
| ECS                 | Bevy's                                        | Decided               |
| Movement, collision | Hand written, Celeste style                   | Decided               |
| Level editor        | LDtk                                          | Decided               |
| Dialogue            | Hand written, rule and fact based             | Decided               |
| Netcode             | Rollback, GGRS                                | Decided               |
| Transport           | WebRTC peer to peer, matchbox                 | Decided               |
| Signaling server    | Own, Cloudflare Worker plus Durable Objects   | Decided               |
| Rendering           | Bevy 2D to start, custom pixel renderer later | Decided in shape      |
| Render resolution   | 320x180, 8 pixel tiles, whole number scaling  | Decided               |
| Simulation units    | `i32` pixels, `f32` remainder and velocity    | Decided               |
| Audio               | bevy_seedling, fallback bevy_kira_audio       | Decided, check at pin |
| UI                  | bevy_ui for players, egui for dev tools       | Decided               |
| Platforms           | WASM in the browser, native for Steam         | Decided               |

### Why Bevy

Bevy is a framework more than an engine: an ECS, a scheduler, windowing, input
and an asset pipeline, with rendering open to replacement. That leaves the
interesting systems (movement, dialogue, rendering, netcode integration) to be
written here, without writing an ECS first. aarthificial chose it for the same
reasons when he left Unity.

Bevy's ECS also suits learning Rust. Systems are plain functions that take
queries, so the ownership fights that come from big graphs of objects holding
references to each other mostly never happen.

### Bevy versions

Bevy is pre 1.0 and breaks its API every release, roughly every three to four
months. The rules:

- Pin one version in `Cargo.toml`. Upgrade only on purpose, when a new version
  has something wanted, with the official migration guide open
- Before adding a third party crate, check its compatibility table against the
  pinned Bevy version
- When a tutorial does not compile, check which Bevy version it targets first

## The simulation and presentation split

This is the load bearing boundary, the same idea as the old TypeScript plan.

- **Simulation**: everything that decides what happens. Positions, velocities,
  collisions, abilities, puzzle state, enemy behaviour, the dialogue fact store
  and rule selection. Deterministic. Runs in the rollback schedule.
- **Presentation**: everything that shows what happened. Sprites, animation,
  camera, particles, lighting, text crawl, voice playback, menus. Free to use
  real time, floats, randomness and anything else. Never feeds back into the
  simulation.

Information flows one way: simulation to presentation. Presentation reads
simulation components and never writes them.

### Enforced by the compiler

The boundary is a crate boundary in a Cargo workspace:

    crates/sim     the simulation. Depends on Bevy's sub-crates for the ECS,
                   app and math (bevy_ecs, bevy_app, bevy_math), never on
                   rendering, audio, windowing or input
    crates/game    the binary. Depends on full bevy and on sim. Presentation,
                   input reading, asset loading, networking glue

Dependency direction: `game -> sim`. Never the reverse.

Two mechanical guards, neither relying on discipline:

1. **Cargo dependencies.** `sim` does not list `bevy_render`, `bevy_audio` and
   the rest, so a `use` of them inside `sim` does not compile. The equivalent
   of the old tsconfig `lib` trick
2. **Clippy bans.** A `clippy.toml` in `crates/sim` uses `disallowed-methods`
   and `disallowed-types` to ban wall clocks (`Instant::now`, `SystemTime`),
   `HashMap` and `HashSet`, and `std` float trigonometry. The ban list is the
   determinism contract below, written as config

Bevy's sub-crate versions are pinned once in `[workspace.dependencies]`, so
`sim` and `game` can never disagree about which Bevy they use.

`bevy_math` is built with its `libm` feature in `sim`, so its float functions
(`bevy::math::ops`) have specified precision across platforms.

A useful side effect: `sim` tests run headless with `cargo test`, without
compiling a renderer.

Rejected alternatives:

- One crate with a module boundary checked by review. Leaks are silent
- A `sim` crate with no Bevy at all. It would mean writing ECS style storage by
  hand, which is a non-goal

Level data crosses the boundary as plain simulation data. See "Levels" below.

## The determinism contract

Rollback only works if both machines compute the same thing from the same
inputs. Everything in the simulation obeys these from the first commit, even
before networking exists, because retrofitting determinism is a rewrite.

- Fixed timestep, 60 ticks per second. Rendering is decoupled
- The simulation is a function of its previous state plus each player's input
  for the tick. Nothing else enters
- No wall clock. No `std::time::Instant` or `SystemTime` in the simulation
- No unseeded randomness. One seeded PRNG, stored as simulation state so it
  rolls back with everything else
- No `f32::sin`, `cos`, `tan`, `powf` and friends from `std`. Their last bits
  can differ between platforms. Use `libm` or lookup tables. Plain `+ - * /` and
  `sqrt` are fine: they are exactly specified by IEEE 754
- No iteration over `HashMap` or `HashSet` in simulation logic. Their order is
  randomised. Use `BTreeMap`, `Vec`, or Bevy entities sorted by a stable id
- Explicit system ordering in the simulation schedule. Bevy runs systems in
  parallel and in an unspecified order unless told otherwise, and an ambiguous
  order is a determinism bug waiting to happen
- Every piece of simulation state is registered for rollback. State that is
  not registered is not restored, which is a desync

GGRS provides a **SyncTest session** that runs the game locally, rolls back
every frame and re-simulates, and reports any mismatch. It catches determinism
bugs with no network involved, so it is used from early on, not saved for
multiplayer. See `02-roadmap.md`.

### Simulation units

Celeste's hybrid. Every actor has:

- a **position** in whole pixels, `i32`
- a **remainder**, the fraction of a pixel not yet moved, `f32`
- a **velocity** in pixels per tick, `f32`

Each tick, velocity is added to the remainder. Whenever the remainder passes a
whole pixel, the actor steps one pixel and checks for collision.

One simulation pixel is one pixel of the 320x180 render target. Collision, art
and rendering share one grid.

Why this and not the alternatives:

- Integer positions make collision exact: an actor is either overlapping a
  solid or it is not
- `f32` remainder and velocity keep motion smooth, including a future grapple
  pendulum
- Basic `f32` arithmetic and `sqrt` are bit identical on x86_64, ARM and WASM,
  and Rust never fuses operations implicitly

Known gap: SyncTest runs on one machine, so it cannot catch differences
between machines, such as browser against native. The Phase 6 per tick state
hashes do. If floats ever cause a desync, the remainder and velocity can move
to fixed point without leaving `sim`.

### Intent and movement are separate systems

- **Intent systems** decide velocity: run, gravity, jump, dash, grapple
- **One move and collide system** turns velocity into pixel steps against
  solids

This is what keeps abilities cheap to add. The grapple starts as a straight
pull toward the hook point. A pendulum later is a different intent system
writing the same velocity (apply gravity, remove the velocity component along
the rope, correct back to rope length), using dot products and `sqrt`, no
trigonometry. The move and collide core never changes. Rope wrapping around
corners is a separate, harder feature and is not committed to.

## Movement and collision

Hand written, after Maddy Thorson's "Celeste and TowerFall Physics":

- **Actors** move and get stopped. **Blockers** stop others. These are separate
  roles, so a box can be both: it falls, and you can stand on it and stack it
- Actors move one pixel at a time and stop when a step would overlap a
  blocker or a solid tile
- Gravity, jumping, coyote time, jump buffering, dash and the rest are rules
  written on top

The components, the bottom up movement order for stacks, and the snapshot
approach are in `05-data-model.md`.

A physics library (Avian or Rapier) is not used for the character. If a later
mechanic genuinely needs simulated physics, such as ropes or stacks of toppling
objects, a library can be added for those objects while the character keeps its
own controller. The cost to weigh at that point is rollback: a physics engine's
full state must be saved and restored every frame.

### Influence is gathered by the influenced

From aarthificial's "Walking on Walls" devlog. A jump pad does not push the
player. The player's movement system gathers every influence in range (jump
pads, grapple points, surfaces), works out the result, and applies it to itself.

In ECS terms: one system queries the player alongside each influencer
component. Adding a new kind of influencer means adding a query, not editing
every other object. Abilities gate influences: an influence is skipped if the
player lacks the ability, which is a fact lookup.

### Character state and animation

The controller is one state machine, `CharacterState`, as in Celeste, written
as a Rust enum whose variants carry their own data.

There is no animation state in the simulation. `game` derives the animation
from `CharacterState`, `Velocity` and `Contacts`. The animation can never
disagree with the physics, and nothing visual is rolled back. That is all the
preparation art needs.

### Control is separate from the body

A `Driver` component says who controls a character: a player slot, a script,
an AI, or nobody. Each produces the same input value, and one controller
executes it. After the driver, an ordered chain of input middleware
(auto navigation, input locks, accessibility assists) can modify the input,
after aarthificial's "Stealing Control from Players". Cutscenes, AI takeover,
companions and control swapping all come from this. See `05-data-model.md`.

## Levels

### Structure

The game is made of **chapters**, each made of several **rooms**.

- One LDtk project file per chapter, under `levels/`
- One LDtk level per room, placed in the chapter's world layout
- The whole chapter is loaded into `sim` at once, in one world coordinate
  space. In co-op the players can be in different rooms, and both machines
  simulate every room

### Authoring

- **IntGrid layers** hold meaning: solid, one way, hazard, crumbling,
  breakable. The look is chosen by auto-tiling rules, so placeholder squares
  and real tiles use the same level data
- **Entity layers** hold spawns, doors, boxes, interactables, triggers, enemies,
  grapple points and camera zones
- **Entity fields** carry data, for example a trigger with
  `event = "on_enter_lab"`, which is how levels hook into dialogue and puzzles
- **Entity references** link things, including across rooms: a switch in one
  room points at the door it opens in another

### Crossing the boundary

`sim` defines its own plain level format and never knows LDtk exists. `game`
loads the LDtk file as a Bevy asset and converts it. Roughly (illustration):

    ChapterData
      rooms: Vec<RoomData>
        origin, width, height        pixels, world space
        tiles: Vec<TileKind>         Empty | Solid | OneWay | Hazard
      spawns: Vec<Spawn>
        Spawn::Player { pos }
        Spawn::Box { pos }
        Spawn::Door { rect, link: LinkId }
        Spawn::Crumble { rect }
        Spawn::Trigger { rect, event: EventId }
        ...

Presentation only data (camera zones, decoration) goes to `game`'s own
structures, not to `sim`.

Rules:

- **The tile grid is static** and is a plain array per room, not an entity per
  tile. Collision asks "what is at this pixel" constantly, and an array answers
  in one step. Because it never changes, it is never part of rollback state
- **Anything that changes is an entity.** Crumbling blocks, breakable walls,
  doors and moving platforms are solid entities checked alongside the grid.
  Their timers and state are components and roll back like everything else.
  They are painted in LDtk as IntGrid values for convenience, and the converter
  turns each connected group of painted cells into one entity
- **Stable ids, never Bevy `Entity`.** LDtk references are UUID strings. The
  converter replaces them with numeric `LinkId`s assigned in a sorted, fixed
  order, so both machines produce identical data. Logic never stores a Bevy
  `Entity` as a link, because entity numbers depend on spawn order
- **Wait for loading.** Bevy loads assets in the background. The simulation
  does not start until conversion has finished. In co-op, both players compare
  a hash of the chapter data before the first tick
- **Hot reload.** Saving in LDtk re-converts the chapter and restarts the room.
  This is the Phase 1 iteration loop

`sim` tests build `ChapterData` from small ASCII grids written in the test, so
collision and movement are tested without the editor.

### Parser

Hand written with `serde`, reading only the layers needed. Chosen over
`bevy_ecs_ldtk` because the conversion step exists anyway, it is a contained
first Rust exercise, and it removes a crate that must track Bevy versions. Tile
art rendering is decided when art arrives.

## Dialogue and world state

Based on aarthificial's Typewriter, which implements Valve's dynamic dialogue
system from Elan Ruskin's 2012 GDC talk.

- **Facts**: integers describing the world. "Times visited this room", "has the
  grappling hook", "chose to help the engineer". Each fact has a **scope** that
  decides how it is saved: global, area, room, player, or temporary
- **Events**: named by id, never by string at runtime
- **Rules**: listen for an event, test criteria over facts, and if chosen, do
  something (show a line, set facts) and fire a follow up event

When an event fires, its rules are tested most specific first and the first
match wins. Events and rules are themselves facts, counting how often they have
fired, which gives "only once" behaviour and conversation resuming for free.

The fact store is the backbone of game state, not only dialogue: abilities,
puzzle progress and story choices all live in it, and the save system saves it.

Determinism notes:

- The fact store and rule selection are simulation. They are plain integers in
  a stable order, so they are cheap to roll back
- Displaying a line (text crawl, portraits, voice) is presentation
- A dialogue choice is player input. It travels in the per player input stream
  like a jump does, or the two machines disagree about the story

Decided:

- Readable keys in data files, numeric typed ids at runtime, and a load time
  check that every referenced key exists
- Rule actions are commands (`SetFact`, `Say`, `Choice`, `SetDriver`, ...),
  which is why rules live in data files
- One line at a time per player. Each player can be in their own
  conversation at once, and a shared conversation occupies both slots.
  Conversations outrank barks
- The world never pauses for one player's conversation, because both machines
  run the same simulation. The talker's input is locked and the other player
  keeps playing. Only shared scenes and the pause vote stop both
- In co-op, the player who started the conversation makes its choices
- Advancing lines is a player setting: auto advance is the client pressing
  confirm for you, so the simulation only ever sees presses
- Bark durations come from the line table, never from audio length
- Saves store facts by key, not by id

Puzzle wiring uses **signals**, a separate double buffered store for what is
true right now, bridged to facts and events by latches. Signals describe
"while", events describe "when".

Full shapes are in `05-data-model.md`.

**Authoring format:** a Yarn style script, written as text files in neovim and
compiled by `game` at load time into rules (sim data, hashed before co-op) and
a strings table per language (presentation). The syntax and the compiler are
our own, because the runtime must be our rule and fact system, not Yarn's
virtual machine. Conditions appear in two places: a `when` clause on a
conversation, which becomes the rule's criteria and takes part in "most
specific first" selection, and `if` blocks and conditional options inside a
conversation, checked against facts when they are reached. Every line carries a
stable key (`#line:baker_hello`) shared by text, translations and voice. The
first few rules in Phase 3 are written in RON to learn the data model, and the
exact syntax is settled after that. `sim` is unaffected by the format.

## Input

Each tick, each player's input is a small fixed size value: a bitmask for
buttons plus whatever the grappling hook needs for aiming. The simulation
derives everything else, such as "ticks since jump was pressed".

Inside the simulation, each character's input carries **held** and **pressed**
as separate masks. `pressed` is computed from the raw input before any input
middleware runs, so lifting an input lock while a button is held cannot invent
a press.

From the first commit the simulation takes input **per player**, even with one
player. That single decision is what makes co-op an addition instead of a
rewrite. GGRS requires this shape anyway.

Grapple aim needs no aim data. The grapple only attaches to **grapple points**,
never to arbitrary surfaces. It fires in one of eight directions from the held
direction buttons (facing when none are held), with **auto targeting as
leeway**: the best grapple point in range, within a cone around that direction,
with clear line of sight, is chosen. Line of sight is a DDA ray through the
tile grid. Ties break by `LinkId`. `game` highlights the chosen point from a
`GrappleFocus` component before the button is pressed. The `aim` byte stays
reserved for a future ability.

## Networking

- **GGRS**: rollback netcode. Each machine predicts the remote player's input,
  and when the real input arrives late, rolls back and re-simulates
- **matchbox**: WebRTC peer to peer, in the browser and natively. Game traffic
  goes directly between the two players
- **Signaling server**: introduces the two players, then steps aside. Written
  by hand as a Cloudflare Worker with one Durable Object per lobby code, in
  `services/signaling/` (TypeScript, outside the Cargo workspace). It speaks
  matchbox's signaling protocol exactly, pinned to the matchbox version in use
- **STUN**: a free public server, so each player can learn its public address
  for hole punching
- **TURN**: Cloudflare's hosted TURN, used only when a direct connection is
  impossible. Not self hosted. Relayed input traffic is roughly tens of
  megabytes per hour of play, and Cloudflare's free monthly allowance
  (1,000 GB at the time of writing, shared with its SFU) makes this effectively
  free. The signaling Worker requests short lived TURN credentials and hands
  them to clients when they join, so no long lived key ships in the game
- **Durable Object relay, stretch goal**: WebRTC can only fall back to a real
  TURN server, so the Durable Object cannot fill that slot. It can be a second
  transport instead: if WebRTC fails to connect, both clients switch to
  WebSockets relayed by the lobby's Durable Object, through GGRS's pluggable
  socket interface. Worth building as a comparison (latency, delay spikes, cost
  against TURN), not as a replacement
- **Steam**: Steam's own networking may replace matchbox for the Steam build.
  Decided when Steam is reached

Decided:

- **Lobbies by code and by link.** A host gets a short code (4 to 6
  characters, no lookalikes) and a join link containing it. Opening the link
  drops a friend straight into the lobby
- **Disconnects return to the last checkpoint.** No mid session rejoin: both
  players reconnect with the code and resume from the checkpoint, which is
  just the saved facts and checkpoint id
- **The signaling server is dumb.** It relays handshakes only and never sees
  game data. Version and chapter hash checks happen between the two clients
  over the new connection, before the first tick
- **Phase 6 order:** GGRS against a local `matchbox_server`, then two machines
  on a home network, then the own server deployed, then TURN tested from a
  strict network

Each player has their own screen, so each client renders its own camera and
can hide what its player should not see. Cheating is irrelevant in co-op, so
hiding information in presentation is enough.

## Rendering

### Resolution

**320x180**, 8 pixel tiles, a player about 8 pixels wide. One screen shows 40
by 22.5 tiles, the same player to screen ratio as Celeste.

Chosen over 480x270 because it scales by a whole number to every common 16:9
screen: 4x at 720p, 6x at 1080p, 8x at 1440p, 12x at 4K, and 4x with thin bars
on the Steam Deck. 480x270 does not scale evenly to 720p, 1440p or the Steam
Deck. It is also half the pixels to draw, which matters while learning
Aseprite.

### Camera

The camera is presentation: floats, real time and easing are all fine, it never
rolls back, and in co-op each client runs its own.

Behaviour is driven by **camera zones**, rectangles placed as LDtk entities.
Each zone has a behaviour (follow with lookahead, lock to the room, follow a
rail, frame a point) and the camera eases between them as the player moves.
Large spaces use a following camera; Celeste style rooms use a locked one.

Smooth scrolling on a pixel grid: render the world to the 320x180 target plus a
one pixel border with the camera snapped to whole pixels, then offset the
upscaled image by the remaining fraction, in screen pixels. Crisp art, smooth
motion.

Zoom is the one hard camera effect. Pixel art only scales cleanly by whole
numbers, so a zoomed out view means rendering a larger target (such as 640x360)
at a smaller scale. It is planned as its own feature if it is ever wanted.

### Order of work

Start with Bevy's built in 2D rendering of coloured rectangles. Later, in
order of ambition:

1. The 320x180 render target, whole number scaling, and the smooth camera
   offset above
2. Sprites and animation from Aseprite
3. Lighting and custom WGSL shaders
4. aarthificial's UV lookup animation: animation frames store coordinates into a
   single appearance texture, so a character's look can change, even at
   runtime, without redrawing any animation

Text and UI are rendered at full resolution, separately from the pixel art, so
dialogue stays readable.

## UI

### Three kinds

| Kind     | Examples                                | Drawn          | State owned by |
| -------- | --------------------------------------- | -------------- | -------------- |
| In world | interact prompts, visible wiring        | 320x180 target | `sim`          |
| Overlays | dialogue, choices, terminal interfaces  | full res layer | `sim`          |
| Menus    | title, pause, settings, lobby, chapters | full res layer | `game` only    |

An interface that changes the simulation does so through input, never by
writing simulation state directly.

### Fonts and resolution

The UI layer renders at full window resolution, separate from the pixel
target. Menus and the HUD use a pixel font drawn at the game's own scale, so one
font pixel matches one game pixel. Long dialogue uses a finer, more readable
font.

### Dialogue text

A presentation system: character by character reveal with pauses on
punctuation, inline tags in the line table (`<shake>`, `<wave>`, colour,
pauses, speed), tag aware word wrapping measured before reveal, per speaker
blip sounds from the sound generator when there is no voice, and confirm to
finish the line then confirm to advance.

### Library

bevy_ui for everything a player sees: entities with flexbox and grid layout,
so CSS knowledge carries over. egui for everything only the developer sees.

### Input devices

Keyboard and gamepad from Phase 0. Menus use focus and directional
navigation. Physical buttons map to `PlayerInput` bits through a hand written
action mapping layer in `game`, with rebinding in settings.

### Pausing

Solo: stop advancing ticks, outside `sim`.

Online co-op: **vote to pause.** A pause request is an input bit. When one
player requests, the other is asked; when both agree, the simulation enters a
paused state on both machines. The pause state is simulation state and rolls
back. Menus that only affect one client (settings) open without pausing.

### Developer tools

Built with egui, each when its system first needs it:

- Fact viewer: every fact, its value and scope, editable live
- Rule trace: for each event, every rule tested, which criteria failed, which
  rule won
- Signal viewer, with optional in world overlay
- Tick controls: pause, step one tick, slow motion
- The entity inspector (`bevy-inspector-egui`)

## Audio

Presentation, behind a thin `audio` module in `game` so only one plugin touches
the library.

### Sound and rollback

A sound triggered by a mispredicted tick plays for something that never
happened, and re-simulated ticks would play sounds twice. So `sim` writes
**cues** tagged with tick, kind and source (see `05-data-model.md`), and `game`
plays each `(tick, kind, source)` at most once. A played cue that disappears
after a rollback is faded out. Voice lines follow the same rule through
`ActiveLines`.

### Mixer

Buses: Master, then Music, Ambience, Sfx, Voice and Ui. Settings sliders are bus
volumes. Music ducks under voice. Bus effects (low pass on pause, reverb in
caves) as wanted.

### Music

One track per area to start. Designed so a track can later become stems
(vertical layering) or sections switched on the beat (horizontal resequencing),
driven by the current room or a fact such as `music_intensity`.

### Positional sound and co-op

2D panning and falloff relative to this client's own camera. Whether a player
hears the other player's room is level data: a room can list rooms it hears,
muffled through a low pass filter. Otherwise other rooms are silent.

### Procedural sound effects

A hand written sfxr style generator: effects defined as parameters (oscillator,
pitch sweep, envelope, noise, filter) in a data file and synthesised at load.
It is the audio placeholder, the way shapes are the art placeholder, so no
audio files are needed until real ones are made.

### Voice recordings

Files named by line key, `voice/{line_key}.ogg`, text only when missing.
Loudness normalised as a build step (for example ffmpeg `loudnorm`) so
recordings from different friends match. Recording sheets per speaker are
generated from the line table.

### Library

bevy_seedling: an audio graph of buses, effects and custom nodes, which suits
building a synth node and the mixer by hand. Confirm its web backend and Bevy
version support when pinning; fall back to bevy_kira_audio if either is
lacking. FMOD is not used: the browser target and a proprietary authoring tool
cost more than they give at this stage.

## Debug tooling

Built early because it pays for itself:

- An inspector for live entity and component state (`bevy-inspector-egui`)
- Hot reload of tuning constants and dialogue rules through Bevy's asset system
- Input recording and replay, later
