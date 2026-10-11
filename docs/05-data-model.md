# Data model

The shape of the simulation's data: components, resources, the order systems
run in each tick, and what rolls back. `01-architecture.md` says _why_ things
are split the way they are; this document says _what_ the pieces are.

Every code block here is **illustration**. Names, field types and exact
signatures are the author's to settle while implementing. What is fixed is the
shape and the reasoning.

Sources the design draws on:

- Maddy Thorson, "Celeste and TowerFall Physics": actors and solids
- Celeste's published player source: the controller as one state machine
- Elan Ruskin, GDC 2012, and aarthificial's Typewriter: facts, events, rules
- aarthificial, "Walking on Walls": influence gathered by the influenced
- aarthificial, "Stealing Control from Players": input providers and an input
  middleware chain
- Robert Nystrom, "Game Programming Patterns": Command, State, Double Buffer,
  Game Loop, Observer, Event Queue, Component

---

## 1. Ids

Data files use readable keys (`has_grapple`, `on_enter_lab`). At load time each
key is given a number, in sorted key order so both machines in co-op agree.

```rust
// illustration
struct LinkId(u16);    // an entity that other things refer to
struct SignalId(u16);  // a puzzle wire
struct FactId(u16);
struct EventId(u16);
struct RuleId(u16);
struct LineId(u16);
struct SpeakerId(u16);

#[derive(Resource)]
struct Registry {
    fact_keys: Vec<String>,                 // FactId -> key
    fact_by_key: BTreeMap<String, FactId>,  // key -> FactId
    // and the same for events, lines, speakers
}
```

- Ids are newtypes, never bare integers, so they cannot be mixed up
- The loader rejects any reference to a key that does not exist
- Logic never stores a Bevy `Entity` across ticks. Entity numbers depend on
  spawn order. Use `LinkId`

Facts that code needs are resolved once:

```rust
// illustration
#[derive(Resource)]
struct CoreFacts { has_dash: FactId, has_grapple: FactId /* ... */ }
```

---

## 2. Level data

`game` converts LDtk into this. `sim` never sees LDtk.

```rust
// illustration
struct ChapterData {
    rooms: Vec<RoomData>,
    spawns: Vec<Spawn>,
}

struct RoomData {
    origin: IVec2,          // top left, world pixels
    size_tiles: UVec2,
    tiles: Vec<TileKind>,   // flat: index = y * width + x
}

#[repr(u8)]
enum TileKind { Empty, Solid, OneWay, Hazard }

enum Spawn {
    Character { id: LinkId, pos: IVec2, driver: DriverSpec },
    Box       { id: LinkId, pos: IVec2 },
    Key       { id: LinkId, pos: IVec2, opens: LinkId },
    Door      { id: LinkId, rect: IRect, inputs: Vec<SignalId>, mode: Combine },
    Lever     { id: LinkId, pos: IVec2, out: SignalId },
    Plate     { id: LinkId, rect: IRect, out: SignalId },
    Crumble   { id: LinkId, rect: IRect },
    Breakable { id: LinkId, rect: IRect },
    Trigger   { rect: IRect, event: EventId },
    // ...
}
```

- The tile grid is static. Anything that changes during play is an entity
- Crumbling and breakable cells are painted in LDtk as IntGrid values; the
  converter turns each connected group into one `Spawn`
- Presentation only data (camera zones, decoration, visible wiring) goes to
  `game`'s own structures

The grids live in a resource with a lookup helper:

```rust
// illustration
#[derive(Resource)]
struct ChapterGrid { rooms: Vec<RoomData> }
// fn tile_at(&self, world_px: IVec2) -> TileKind
```

---

## 3. Bodies: movement and collision

Every physical thing is a body. Roles are separate markers, so one entity can
have several:

```rust
// illustration
#[derive(Component)] struct Actor;     // moves, and collision stops it
#[derive(Component)] enum Blocks { All, FromAbove }  // others collide with it
#[derive(Component)] struct Pushable { weight: u8 }  // actors walking into it shove it
#[derive(Component)] struct PushStrength(u8);        // on characters

#[derive(Component)] struct Position(IVec2);          // whole pixels
#[derive(Component)] struct PreviousPosition(IVec2);  // for render blending
#[derive(Component)] struct Remainder(Vec2);          // sub pixel leftover
#[derive(Component)] struct Velocity(Vec2);           // pixels per tick
#[derive(Component)] struct Collider { offset: IVec2, size: UVec2 }

#[derive(Component)]
struct Contacts {
    ground: bool, ceiling: bool, wall_left: bool, wall_right: bool,
    riding: Option<LinkId>,
}
```

| Thing              | Actor | Blocks | Pushable |
| ------------------ | ----- | ------ | -------- |
| Character          | yes   | above  | no       |
| Box                | yes   | yes    | yes      |
| Door, platform     | no    | yes    | no       |
| Crumble, breakable | no    | yes    | no       |

### How a tick of movement works

1. **Blockers that are not actors move first** (doors, platforms). Each one
   carries whatever rides on it, and pushes what it moves into. An actor with
   nowhere to go is squished
2. **A snapshot** of every blocker's rectangle is collected into a plain `Vec`
3. **Actors move one at a time, bottom of the stack first**, ordered by height
   then `LinkId`. Each one steps pixel by pixel against the tile grid and the
   snapshot, ignoring its own rectangle. The snapshot is updated after each
   actor moves, so a box on top sees where the box below went. An actor that
   also blocks carries its riders, recursively, so stacks move together
4. Each actor's `Contacts` are written as it moves, for next tick to read

The snapshot sidesteps Bevy's rule that two queries may not both access
`Position` on the same entity, which boxes (both `Actor` and `Blocks`) would
otherwise break.

### Standing on each other

Characters are `Blocks::FromAbove`: solid only to something landing on them
from above, like a one way platform. Players can stand on each other and ride
along, but never shove each other sideways or get wedged in a corridor. The
character underneath carries the one on top through the same riding logic as a
box.

### Pushing chains

Pushing a box into another box pushes both. The push succeeds only when the
pusher's `PushStrength` is at least the total `weight` of the chain. Two
characters pushing the same chain add their strengths, so heavy chains are a
co-op puzzle. Strength can grow through facts (an upgrade), like any ability.

Move functions return what they hit rather than taking callbacks. Closures that
borrow the world are painful in Rust; return values are not.

Bevy's `Transform` is not used in `sim`. `game` copies `Position` into
`Transform`, blending from `PreviousPosition` on high refresh displays.

---

## 4. Characters

Anything with a controller: the player, NPCs, companions, enemies.

```rust
// illustration
#[derive(Component)]
enum CharacterState {
    Normal,
    Dashing           { dir: IVec2, ticks_left: u8 },
    Grappling         { anchor: IVec2 },
    UsingInteractable { target: LinkId },
    Scripted          { path: PathId, tick: u16 },   // kinematic cutscene moves
    Dead              { ticks_until_respawn: u8 },
}

#[derive(Component)]
struct CharacterTimers {
    coyote_ticks: u8,
    jump_buffer_ticks: u8,
    dash_cooldown: u8,
    dashes_left: u8,
}

#[derive(Component)] struct Facing(i8);   // -1 or 1
```

Illegal states are unrepresentable: a dash direction only exists while dashing.

There is no `MotionState` in `sim`. `game` derives the animation from
`CharacterState`, `Velocity` and `Contacts`, so the animation can never
disagree with the physics and nothing visual is rolled back.

### Who drives a character

Control is a relationship, not identity. "Player" is a person; "character" is a
body.

```rust
// illustration
#[derive(Component)]
enum Driver {
    Player(PlayerSlot),
    Script { script: ScriptId, tick: u32 },
    Ai(AiKind),
    None,
}
```

Changing who controls a body is changing one component. Cutscenes, AI takeover,
companions and control swapping all fall out of it. This is the command and
strategy patterns: every driver produces the same input value, and one
controller executes it.

### Character input, and the input chain

```rust
// illustration
#[derive(Component)]
struct CharacterInput {
    held: u16,     // buttons down now
    pressed: u16,  // buttons that went down this tick
    aim: u8,
}

#[derive(Component)] struct RawInputHistory { prev_held: u16 }

// middleware, each present only when active
#[derive(Component)] struct AutoNavigate { target_x: i32, cancel_on_opposite: bool, on_cancel: EventId }
#[derive(Component)] struct InputLock { mask: u16 }
```

The chain, run as explicitly ordered systems:

1. **drive**: the `Driver` writes `held` and `aim`. `pressed` is computed here,
   from the driver's _raw_ input against `RawInputHistory`. A script may set
   `pressed` directly
2. **auto_navigate**: overrides direction; pushing the other way cancels it and
   fires `on_cancel`
3. **input_lock**: clears locked bits from both `held` and `pressed`
4. **accessibility**: optional assists the player has turned on
5. **character_control**: the state machine, which knows nothing about
   dialogue or cutscenes

`pressed` is computed before the chain on purpose. If it were computed after,
lifting an `InputLock` while a button is held would create a press that never
happened: the "jumps the moment the cutscene ends" bug.

### Influences

```rust
// illustration
#[derive(Component)] struct JumpPad { strength: f32 }
#[derive(Component)] struct GrapplePoint { range: u16 }
#[derive(Component)] struct GrappleFocus(Option<LinkId>);   // on characters: the target if grapple is pressed now
```

Influencers have no behaviour of their own. `character_control` gathers the ones
in range and decides what applies. Ability gating is a fact check at the top of
each branch.

### Tuning

```rust
// illustration
#[derive(Resource, Deserialize)]
struct MovementTuning {
    run_speed: f32, run_accel: f32, run_friction: f32,
    gravity_rise: f32, gravity_fall: f32, max_fall: f32,
    jump_speed: f32, coyote_ticks: u8, jump_buffer_ticks: u8,
    dash_speed: f32, dash_ticks: u8,
}
```

Loaded from RON by `game` and hot reloaded. Part of the data hashed before a
co-op session.

---

## 5. Interactables and carriables

```rust
// illustration
#[derive(Component)] struct Interactable { range: u16, kind: InteractKind }
#[derive(Component)] struct InteractFocus(Option<LinkId>);   // on characters

#[derive(Component)] struct Carriable;
#[derive(Component)] struct Carried { by: LinkId };           // only while held
#[derive(Component)] struct Key { opens: LinkId }
#[derive(Component)] struct RespawnAt(IVec2);                 // where it was found

#[derive(Component)]
struct CombinationLock { digits: [u8; 4], cursor: u8, solution: [u8; 4], out: SignalId }
```

- Each tick, each character's `InteractFocus` is set to the closest
  interactable in range. `game` draws the prompt from it, and only when
  interacting would do something
- Interacting either acts directly (a lever flips) or fires an event with the
  character as instigator
- **Carrying:** picking up adds `Carried`; the object then snaps above its
  carrier each tick. Throwing removes `Carried` and sets `Velocity`, and the
  object is a normal actor again. Throws across rooms need nothing special
- **Lost items:** a carriable that falls into a pit or a hazard returns to
  `RespawnAt`, the spot where it was found (its LDtk spawn). If that spot is
  occupied it waits until it is clear. Story items and keys are never lost for
  good
- **Interfaces:** `CharacterState::UsingInteractable` routes the character's
  input to the interactable's own state. `game` draws the interface. Each
  client chooses what its own player sees, which is how co-op communication
  puzzles work

---

## 6. Signals

Puzzle wiring. A signal describes what **is** true; events describe what
**happened**. Rule of thumb: "while" is a signal, "when" is an event.

```rust
// illustration
#[derive(Resource)]
struct Signals {
    current: Vec<u8>,   // read by receivers this tick
    next: Vec<u8>,      // written by emitters this tick
}

#[derive(Component)] struct PressurePlate { out: SignalId }
#[derive(Component)] struct Lever { out: SignalId, on: bool }
#[derive(Component)] struct Button { out: SignalId, hold_ticks: u16, ticks_left: u16 }

#[derive(Component)]
struct SignalInput { inputs: Vec<SignalId>, mode: Combine, invert: bool }
enum Combine { Any, All }

#[derive(Component)] struct Door { open_ticks: u8 }

#[derive(Component)]
struct Latch { input: SignalId, fired: bool, actions: Vec<Action> }
```

- **Double buffered.** Emitters write `next`, receivers read `current`, and the
  two swap at the end of the tick. System order cannot change the result. Each
  hop along a chain costs one tick
- **Cross room for free**, because the whole chapter is simulated
- **Latches** bridge signals to the fact and event world: the first time a
  signal turns on, run actions (usually `SetFact` and `FireEvent`). This is the
  "puzzle solved" moment
- **Facts can drive signals**, so a solved puzzle stays solved: on room load,
  the latch's fact switches its signal on
- **Visible wiring** is presentation. `game` gets a list of emitter and receiver
  positions per signal and draws the lines

---

## 7. Facts and dialogue

### Facts

```rust
// illustration
#[derive(Resource)]
struct Facts {
    shared: Vec<i32>,           // by FactId
    per_player: [Vec<i32>; 2],  // by FactId, per player slot
}

struct FactDef { key: String, scope: Scope }   // static

enum Scope { Global, Chapter, Room, Player, Temporary }
```

| Scope     | Lifetime                          | Example             |
| --------- | --------------------------------- | ------------------- |
| Global    | the whole save                    | story choices       |
| Chapter   | one chapter                       | `lab_door_solved`   |
| Room      | one room, saved                   | `times_visited_lab` |
| Player    | one copy per player slot          | `heard_intro`       |
| Temporary | reset on room reload, never saved | `boss_is_watching`  |

A small `world_facts` system copies a few live values (grounded, current room,
held ability) into temporary facts before rules run. Keep that list short.

### Events

```rust
// illustration
struct EventFired { event: EventId, instigator: Option<LinkId> }

#[derive(Resource)]
struct PendingEvents(Vec<EventFired>);   // empty at the end of every tick
```

### Rules

```rust
// illustration
struct Rule {
    listens: EventId,
    criteria: Vec<Criterion>,
    actions: Vec<Action>,
    once: bool,
    interruptible: bool,
}

struct Criterion { fact: FactRef, op: Op, value: i32 }
enum FactRef { Shared(FactId), Instigator(FactId) }
enum Op { Eq, Ne, Lt, Le, Gt, Ge }
```

- Rules are grouped by event at load time, sorted most criteria first, ties in
  file order. First match wins
- Rules and events are facts counting how often they fired. `once` means "only
  while my count is 0", which makes conversations resume where they left off

### Actions

The command pattern: data describing what to do, which is why rules live in
data files.

```rust
// illustration
enum Action {
    SetFact   { fact: FactRef, value: i32 },
    AddFact   { fact: FactRef, delta: i32 },
    FireEvent(EventId),
    Say       { line: LineId, speaker: SpeakerId, mode: SayMode },
    Choice    { options: Vec<ChoiceOption> },
    SetDriver { character: LinkId, driver: DriverSpec },
    AddInputLock    { character: LinkId, mask: u16 },
    RemoveInputLock { character: LinkId },
    AutoNavigate    { character: LinkId, target_x: i32 },
    Wait      { ticks: u16 },
}

enum SayMode { Bark, Conversation }
```

Getting an ability is `SetFact { has_grapple, 1 }`. Cutscenes are rules whose
actions change drivers, add locks and say lines.

### Conversations in progress

```rust
// illustration
struct ActiveLine {
    rule: RuleId,
    line: LineId,
    speaker: SpeakerId,
    ticks_left: u16,
    next_action: u8,
}

const MAX_PLAYERS: usize = 2;
#[derive(Resource)] struct ActiveLines([Option<ActiveLine>; MAX_PLAYERS]);  // one slot per player, indexed by PlayerSlot

#[derive(Resource)]
struct PendingChoice { options: Vec<ChoiceOption>, chooser: PlayerSlot, cursor: u8 }
```

- **One line at a time per player.** Each player has their own NPCs, so both can
  be in a conversation at once. A conversation with both players fills both
  slots with the same line. A conversation outranks a bark; a new bark arriving
  while a line plays is dropped
- **The world keeps running.** A conversation locks only its participants'
  input. The simulation is shared, so it cannot pause for one player
- **Conversations** lock the participants' input with `InputLock`. Lines advance
  on a confirm press
- **Auto advance** is a player setting in `game`: the client presses confirm
  when the line's duration ends. The simulation only ever sees presses, so
  players with different settings cannot desync
- **Barks** run on `ticks_left` while play continues
- **Choices** are picked by the event's **instigator**. The other player sees
  them read only. The chooser's input moves the cursor and confirms
- **Interruptible** rules stop if the player walks away and resume later through
  `once` counts. Non interruptible ones hold the lock until done

### Lines: the simulation never sees words

`sim` knows `LineId` and a duration in ticks. Everything else is presentation:

```
// illustration: line table, keyed by LineId
line 812:
  text:     "..."
  voice:    voice/architect_812.ogg     optional, text only if missing
  portrait: architect_suspicious
  ticks:    150                          read by sim, for barks
```

A bark's duration comes from this table, never from the length of a voice
recording, so a re-recorded line cannot desync anything.

The table is split in two:

- **Line data**, the same for every language: key, speaker, portrait,
  `ticks`. `sim` reads only this part. Two co-op players reading in different
  languages must agree on every duration, so `ticks` can never come from the
  translated text
- **Strings**, one table per language (`strings/en.csv`, `strings/fr.csv`):
  key, text, a hash of the source text, and a note for the translator. A
  translation whose hash no longer matches the source is out of date. Voice
  files follow the same keys per language (`voice/en/baker_hello.ogg`)

---

## 8. Saving

A save is the saved scopes of the fact store (Global, Chapter, Room, Player),
the current chapter, and the last checkpoint. Nothing else. Rooms reset on
re-entry, and anything that must persist is a fact.

Facts are saved **by key, not by id**, because ids are reassigned whenever a
fact is added. Unknown keys are dropped with a warning in development builds.

---

## 9. One tick, in order

Every step is an explicitly ordered system in the simulation schedule.

     0. store_previous_positions
     1. drive                  Driver -> CharacterInput, pressed from raw input
     2. auto_navigate          input chain
     3. input_lock             input chain
     4. accessibility          input chain
     5. world_facts            live values into temporary facts
     6. interact_focus         closest interactable per character
     7. character_control      state machine, influences, interact, carry, throw
     8. apply_gravity
     9. move_blockers          doors, platforms, crumbles: carry, push, squish
    10. move_actors            snapshot, bottom up, writes Contacts
    11. follow_carrier         carried objects snap to their carrier
    12. world_reactions        plates, crumbles, breakables, hazards, triggers
    13. emit_signals           write Signals.next
    14. receive_signals        read Signals.current: doors and the like
    15. latches                rising signals run their actions
    16. run_rules              drain PendingEvents; rules may fire more events
    17. advance_dialogue       ActiveLines, confirm presses, PendingChoice
    18. end_of_tick            swap signal buffers; in debug builds, assert
                               PendingEvents is empty

---

## Simulation output: sound cues

Not state, and not rolled back. Written during a tick and collected by `game`
after every simulated tick, including re-simulated ones.

```rust
// illustration
struct Cue { tick: u32, kind: CueKind, source: LinkId }

#[derive(Resource)]
struct TickCues(Vec<Cue>);   // cleared at the start of each tick
```

`game` keeps a short memory of `(tick, kind, source)` keys already played:
seen before is skipped, new is played, played but missing after a rollback is
faded out.

---

## 10. What rolls back

Rollback saves the simulation every tick and restores it when a late input
arrives. Anything that changes during play and is not registered is not
restored, which is a desync.

**Rule:** if a component or resource is ever inserted, removed or changed after
the chapter loads, it is registered for rollback.

**Rule:** no spawning or despawning during play. All of a chapter's entities
exist from load. A broken wall or a used key is deactivated, not despawned. This
removes a whole class of rollback bugs around entities appearing and
disappearing.

### Registered: changes during play

| Area        | State                                                                           |
| ----------- | ------------------------------------------------------------------------------- |
| Bodies      | `Position`, `PreviousPosition`, `Remainder`, `Velocity`, `Contacts`             |
| Characters  | `CharacterState`, `CharacterTimers`, `Facing`, `Driver`                         |
| Input       | `CharacterInput`, `RawInputHistory`, `AutoNavigate`, `InputLock`                |
| Interaction | `InteractFocus`, `Carried`, `CombinationLock`, active or inactive flags         |
| World       | `Door`, `Lever`, `Button`, `Latch`, crumble and breakable state, enemy AI state |
| Signals     | `Signals`                                                                       |
| Facts       | `Facts`                                                                         |
| Dialogue    | `ActiveLines`, `PendingChoice`                                                  |
| Session     | `SessionPause` (requests from each player, paused or not)                       |
| Core        | the seeded PRNG, the tick counter                                               |

### Static: loaded once, hashed before co-op

`ChapterGrid`, `Registry`, `FactDef`s, `CoreFacts`, rules (grouped and sorted),
line durations, `MovementTuning`, script timelines and paths, and the
components that never change after spawn (`Actor`, `Blocks`, `Pushable`,
`Collider`, `Interactable`, `Key`, `SignalInput`, emitter outputs, `JumpPad`,
`GrapplePoint`, `LinkId`).

### Within one tick only: nothing to restore

`PendingEvents`, the blocker snapshot, any command list built and consumed in
the same tick. Debug builds assert they are empty at the end of the tick.

### Never in the simulation

`Transform`, sprites, animation, camera, particles, lighting, audio, voice,
text, portraits, prompts, visible wiring, menus, settings such as auto advance.

The cost is small: a few hundred entities and a few kilobytes of facts, saved
for a handful of ticks of prediction. SyncTest catches anything that changes but
was not registered.

---

## Open

- The exact script syntax, settled after the first RON rules in Phase 3
