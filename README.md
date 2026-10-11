# Shared Ground

Working title. A 2D pixel-art platformer with tight, Celeste-style movement,
tools like a dash and a grappling hook, puzzles, and a branching story voiced
by friends. It ships as a solo prequel first, then a two-player online co-op
campaign.

Built in Rust on Bevy. Movement, collision, dialogue and rendering are hand
written, because building those systems is the point.

## Status

Phase 0. Docs only so far; the Cargo workspace comes next. See [docs/02-roadmap.md](docs/02-roadmap.md).

## Stack

Rust, Bevy (version pinned), LDtk for levels, GGRS rollback netcode over
matchbox WebRTC. Runs in the browser through WASM, and natively for Steam.

## Docs

| Document                                      | What is in it                                               |
| --------------------------------------------- | ----------------------------------------------------------- |
| [00-vision.md](docs/00-vision.md)             | The game, the shippable targets, non-goals                  |
| [01-architecture.md](docs/01-architecture.md) | Stack, the simulation boundary, determinism, open questions |
| [02-roadmap.md](docs/02-roadmap.md)           | Phases with exit criteria                                   |
| [03-conventions.md](docs/03-conventions.md)   | Units, input model, naming, testing, commits                |
| [04-neovim.md](docs/04-neovim.md)             | Working in this repo with neovim                            |
| [05-data-model.md](docs/05-data-model.md)     | Components, resources, tick order, what rolls back          |
| [CLAUDE.md](CLAUDE.md)                        | How AI assistants should work in this repo                  |

## Why it is built this way

It is a learning project. The author comes from web development and is
learning Rust, game development, systems design and neovim at the same time.

The constraints that shaped the decisions:

- **Determinism from day one.** Fixed timestep, input driven, no wall clock, no
  unseeded randomness. It is the precondition for rollback co-op, and it makes
  every bug reproducible
- **Simulation and presentation are separate.** Gameplay never depends on art,
  so placeholder shapes can stay while the art skills are learned
- **Content led.** Systems are built when a level needs them, so there is
  always something playable

## Licence

All rights reserved. No part of this repository may be used, copied or
redistributed without written permission. See [LICENSE](LICENSE).
