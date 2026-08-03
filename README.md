# O.R.B.S.

**Operational Relic Bewitching System** — Blackhearth Studios

A wizard stares into his scrying orb and finds a computer terminal inside it.

O.R.B.S. is a text-only game played entirely through a fantasy command line. You
tend a wizard's tower by navigating a filesystem that *is* your duties — brewing
in `/alembic`, warding in `/battlements`, spying in `/lens` — and you progress by
writing scripts that teach the orb to do your work without you. When you are
ready, you descend into a siege, where an intelligent enemy attacks the automation
you built and you must diagnose and repair it under pressure.

Artless by design. No sprites, no characters, no illustrations. A single curved
CRT glowing in the dark.

```
O.R.B.S. v0.9.3  —  cold start

scrying lens ................ [ ok ]
ley-line uplink ............. [ ok ]
grimoire index .............. [ 2841 ]
alembic ..................... [ ok ]
battlements ................. [ DEGRADED ]
  east_wall integrity 34%
menagerie ................... [ not found ]
archive ..................... [ ok ]

3 warnings. the orb warms to your touch.

orbs:~$ _
```

---

**Status: pre-production.** No implementation code yet.

| | |
|---|---|
| [docs/DESIGN.md](docs/DESIGN.md) | The design — authoritative |
| [docs/ROADMAP.md](docs/ROADMAP.md) | Phase status |
| [docs/SETUP.md](docs/SETUP.md) | Toolchain, skills, build and release |
| [CLAUDE.md](CLAUDE.md) | Conventions and architectural rules |

Rust · Bevy 0.19 · Windows, macOS, Linux
