# Vision

Shared Ground is a working title. It is a 2D pixel-art co-op puzzle platformer
for two players, built as a long-running passion project with no deadline.

It is the author's second game, so it is deliberately a normal one: two
characters who need each other to get through rooms, no plot. The story is
parked (see the end of this document) and can be layered on later, because no
system depends on it.

## The game

### Movement

Tight controls in the spirit of Celeste: coyote time, jump buffering, variable
jump height, a dash. Celeste describes how the movement _feels_, not how hard
the rooms are.

**Difficulty is It Takes Two level.** Forgiving jumps, generous checkpoints,
and the challenge is coordinating with your partner, not pixel-perfect inputs.

Both characters share the same base movement, so neither player feels like the
weaker one:

- Run, jump, wall jump, dash
- Interact: push and pull boxes, use switches and terminals
- Carry small objects, and hand them to your partner
- Boost your partner up, and pull them up a ledge

### Split tools

On top of the shared movement, the two characters get **complementary tools
that are useless alone**, in the way It Takes Two gives one player a hammer and
the other nails. Each pair is a prototype first and earns its place in a
playtest. Candidates:

| One character                                      | The other                                      |
| -------------------------------------------------- | ---------------------------------------------- |
| **Throw**: launches the partner far                | **Glide**: steers in the air after a throw     |
| **Light**: reveals paths that only exist when lit  | **Reach**: small and fast, fits gaps           |
| **Hold**: keeps heavy doors and platforms in place | **Retrieve**: carries keys and items through   |
| **Shield**: blocks hazards for a short time        | **Sneak**: unnoticed by enemies while shielded |

Throw and glide is prototyped first. The grappling hook is still a candidate
tool, but no longer the main focus.

One small new mechanic per level set is the rhythm: introduce it safely,
develop it, twist it, finish with a set piece that uses it fully.

### Puzzles and interactables

- Physical puzzles, such as moving a box onto something
- Interactables that open their own small interface, like a terminal or a
  lock mechanism, which are puzzles in their own right
- Enemies that exist as obstacles and threats, but with **no combat**. You
  avoid, outsmart or use them, you do not fight them

### Co-op

Each player has their own screen: split screen on one machine, their own
window online. Depending on the room, the two players are either:

- **In the same room**, cooperating directly, or
- **In different rooms that affect each other.** A key, a switch or a puzzle in
  one player's room changes something in the other's, so players have to
  communicate what they can see and do.

Voice chat helps but is never required. A ping ("here") and a countdown (act on
the same beat) are built in, after Portal 2's co-op.

When a player dies in a shared room, only they respawn and puzzle objects keep
their state. Both players can vote to reset the room.

### The demo level: dialogue and voice

Building systems is part of why this project exists, and the dialogue system is
one of the ones the author wants to build. It gets a fixed target so it does
not grow without content: **a demo level** with a few rooms, two or three NPCs
and a small self-contained situation, no plot.

The demo decides what the dialogue system has to do, and nothing more:

- An NPC who comments while you play, without stopping you
- A stop-and-talk conversation
- An NPC who reacts to something you actually did, not only to where you are
- One choice that changes something later in the level

It is built for one player first. The data model already gives each player
their own dialogue slot, so two players is a later step, not a rewrite.

**Voice acting, for fun.** Friends voice the demo's lines. Every line has a
stable id, and recordings are looked up by that id. The fallback chain is
recording, then a babble voice (a short pitched sound per syllable, as in
Celeste and Animal Crossing), then silent text. Recording can happen in any
order, whenever people are free, without blocking anything.

## Art direction

**Final target: pixel art** at 320x180 with 8 pixel tiles, scaled up by a whole
number (see `01-architecture.md`).

**The look: a few colours per level set.** Three or four colours each, after
Downwell, Return of the Obra Dinn and Minit. The palette carries the mood, and
a palette swap shader is the first shader to learn.

- **Each player's screen can have its own palette.** Same room, same
  collision, different colours. It is presentation only, so it never touches
  the simulation, and it is something almost no co-op game does
- **The gameplay layer never changes style.** Solids, hazards, interactables
  and the players stay high contrast and consistent everywhere. Mood lives in
  the background, the palette and later the lighting

**Until then: shapes.** Coloured rectangles and lines, in the spirit of Thomas
Was Alone. They are placeholders on purpose: the gameplay never depends on the
art, so the shapes can stay for as long as learning the art takes.

The art skills (Aseprite for pixel art and animation, shaders for lighting and
procedural effects) are learned in parallel with the game, not before it.

Inspirations for the rendering work, all from aarthificial: his custom pixel
renderer, lighting, and the UV lookup animation technique, which separates how a
character looks from how it moves. These are later goals, not starting points.

## Platforms

- **Browser** first. A link friends can click is the lowest-friction way to get
  playtesters
- **Steam** later, as a native download. Learning how to ship there is part of
  the point

Bevy builds to both from the same code.

## The shippable target

**A co-op puzzle platformer on the web.** A short set of co-op levels with
same-room and cross-room puzzles, two players online, plus the demo level with
dialogue and voices. A link two friends can open and finish together.

Everything past that is optional: Steam, more levels, the story.

## Non-goals

Deliberately out of scope, so they stop being open questions:

- Combat. Enemies are obstacles, not opponents
- More than two players
- Competitive or versus play
- Mobile or touch controls
- Procedurally generated levels. Levels are handcrafted. Procedural _art_ and
  effects are in scope
- Writing an ECS from scratch. Bevy provides it
- A plot, for now. See below

## Why this project exists

It is a learning project first, and a game second, and both matter.

The author's background is HTML, CSS and TypeScript. This project is a
deliberate step out of that: learning Rust, game development, systems design,
rendering, networking and neovim, over a long time, at their own pace. The
satisfying part is watching separate systems come together into a game.

## Lessons taken from aarthificial

His "A Little Reset" video describes the mistakes that cost him years on
Astortion. They are written here because this project is exposed to the same
ones, especially the love of building systems.

- **Playtest the core mechanic early.** He never did, and the one friend who
  played it could not understand it
- **Do not chase a perfect vertical slice** before designing the game
- **Make content early.** Fear of redoing levels is not a reason to delay them.
  "The rest of the game will never be ready"
- **Always have something playable**, even if most of it is rough
- **Systems serve content.** Building systems is allowed to be fun, but each one
  is built for a level that needs it. The demo level is that level for
  dialogue. See the rule at the bottom of `02-roadmap.md`

## Parked: the story

Set aside so the game can be made first. Kept here so the thinking is not lost.
Nothing in it is decided.

**Directions explored:**

- **Codependency and moving on.** The most recent direction. Two characters,
  "the one who leans" and "the one who holds". Leaning on your partner is
  rewarded early, then starts to cost the holder something that only shows on
  the holder's screen (colour draining, music thinning, weaker movement). The
  leaner's screen looks fine. Later the two learn to help each other from a
  distance, and moving on is a choice both players make, good for both
- **Grief and a coma dream.** The earlier premise: two people in an accident,
  one died, one in a coma, an abstract dream space built from their shared life,
  one chapter per memory. Set aside for the direction above

**Ideas that would come back with a story:**

- An abstract dream space, one level set per stage of the relationship, each
  with its own palette and shape language. Memory carried by a few recognisable
  objects rather than real places
- Per-player palettes that drift apart over the game, then briefly match
- A control mapping taught for hours, then used for emotion (Brothers)
- Letting go as an action the player chooses when to take (Spiritfarer)
- Swapping roles partway, so each player has been both characters
- Revisiting a level later, decayed: recognisable, changed, and cheap to build

**Dialogue design that would come back with a story:** in co-op, each player
has their own NPCs and choices, and neither sees the other's dialogue. Choices
are private (your own character), echo (changes the other player's world,
often without them knowing it was a choice) or joint (both choose, kept for big
moments). A small branching budget: many small choices stored as facts,
reactive lines instead of branches, few structural forks, so voice recording
stays possible.

**Set aside along the way:** everyday places scaled up (needs art for real
locations, and it is the premise of It Takes Two); a literal past and present
setting (realistic places limit level design); the grapple as the main ability;
a partner who becomes the ground the other stands on.

Inspirations: It Takes Two, Brothers: A Tale of Two Sons, Spiritfarer, Inside,
Gris, Journey, Portal 2's co-op, Unravel Two.
