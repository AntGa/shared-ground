# Vision

Shared Ground is a working title. It is a 2D pixel-art platformer with a
branching story, built as a long-running passion project with no deadline. It
ships in two parts:

1. **A solo prequel.** One character. This is where the movement, abilities and
   puzzles are learned, by the player and by the author. It is the first thing
   that ships.
2. **A two-player online co-op campaign.** The main game. It builds on
   everything the prequel teaches and adds a partner.

The prequel being first is deliberate: it is a complete, shippable game that
needs no networking, and it forces the core of the game to exist before the
hardest technical problem is touched.

## The game

### Movement and abilities

Tight, authored platforming in the spirit of Celeste. Movement is the
foundation and gets the most attention.

On top of running and jumping, the character gains tools. The current list:

- **Dash**
- **Grappling hook**
- More to be decided in a dedicated abilities spar

Abilities are unlocked over the course of the game, and the world, the
dialogue and the puzzles all react to which ones you have.

### Puzzles and interactables

- Physical puzzles, such as moving a box onto something
- Interactables that open their own small interface, like a terminal or a
  lock mechanism, which are puzzles in their own right
- Enemies that exist as obstacles and threats, but with **no combat**. You
  avoid, outsmart or use them, you do not fight them

### Co-op

Each player has their own screen. Depending on the scene, the two players are
either:

- **In the same room**, cooperating directly, or
- **In different rooms that affect each other.** A key, a switch or a puzzle in
  one player's room changes something in the other's, so players have to
  communicate what they can see and do.

The solo prequel uses the same mechanics with one character, which is why the
co-op campaign is an addition rather than a different game.

### Story

Characters who talk, and a story that branches on player choices. Roughly two
hours or more of play across both parts.

Dialogue happens in two ways, mixed per scene:

- **While you play.** Characters comment and react without stopping you
- **Stop-and-talk conversations**, for the scenes that need them

Characters react to what you actually did, not only to where you are. See the
dialogue section of `01-architecture.md` for how.

In co-op, each player has their own cast of NPCs and makes their own story
choices. Choices come in three kinds:

- **Private.** Affects only your own character and relationships
- **Echo.** One player's choice changes the other player's world. The other
  player often does not know it was a choice, and whether to tell them over
  voice chat is up to the player
- **Joint.** Both players choose and the combination matters. Kept for big
  moments, especially the ending

Neither player sees the other's dialogue by default. What players choose to
tell each other is part of the story.

Branching budget: many small choices stored as facts, reactive lines instead of
branches, and only a few structural forks in the whole campaign. Every voiced
line multiplies across branches, so this keeps recording sessions possible.

The story is provisional and nothing in it is locked, including who the two
characters are to each other. Working premise: a game about grief and moving
on, with a hopeful ending. Two people were in an accident; one died and one is
in a coma. The game is set in a dream world built from their shared life, one
chapter per memory, each chapter an everyday place scaled up and made strange.
The lost one wants the survivor to wake up, which means letting go. Early
chapters are warm and cozy, later ones eerie.

Ability ideas that came out of this, also provisional: a shared core (run,
jump, dash), the grapple as the survivor's "holding on", a fading ability for
the lost one that grows as they disappear, and one small mechanic per chapter.

Inspirations: It Takes Two, Up, Orpheus and Eurydice, Eternal Sunshine of the
Spotless Mind, Brothers: A Tale of Two Sons, Spiritfarer, To the Moon. A literal
past and present premise (one place, two times) was considered and set aside
because realistic settings limit level design.

None of the systems depend on these choices. Story is revisited once the
mechanics are playable.

The author does not consider themselves a writer, so the plan is to get
movement on screen first and develop the story alongside the mechanics, not
before them.

### Voice acting

Friends will voice the characters. Every line of dialogue has a stable id, and
recordings are looked up by that id. A line with no recording shows as text, so
voice work can happen in any order, whenever people are free, without blocking
anything.

## Art direction

**Final target: pixel art**, rendered at a low fixed resolution and scaled up by
a whole number. The exact resolution (320x180 or 480x270 are the candidates) is
decided in the architecture spar.

**Until then: shapes.** Coloured rectangles and lines, in the spirit of Thomas
Was Alone. These are placeholders, but they are placeholders on purpose: the
gameplay never depends on the art, so the shapes can stay for as long as
learning the art takes.

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

## The shippable targets

**Target 1, the solo prequel on the web.** Movement, abilities, puzzles, a
story with dialogue and at least one meaningful choice, start to finish, in the
browser.

**Target 2, the co-op campaign.** Two players online, same-room and cross-room
puzzles, the main story. Browser and Steam.

Everything past those is optional.

## Non-goals

Deliberately out of scope, so they stop being open questions:

- Combat. Enemies are obstacles, not opponents
- More than two players
- Competitive or versus play
- Mobile or touch controls
- Procedurally generated levels. Levels are handcrafted. Procedural _art_ and
  effects are in scope
- Writing an ECS from scratch. Bevy provides it

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
- **Systems serve content.** A system is built when a level needs it, not
  because building it is fun. See the rule at the bottom of `02-roadmap.md`
