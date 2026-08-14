# Fallout 4 Terminal-Hacking Minigame
The iconic terminal-hacking minigame from Fallout 4, readily available inside your terminal (emulator) of choice.

(Only tested on Linux yet, though all dependencies should be cross-platform)

## Build
### Dependencies
- `python3`
- `cargo`
### Instructions
```bash
cargo build
```

## Interesting facts
- The lists of random words, which can appear in the minigame are generated at build-time, using the function `top_n_list` from the Python package [`wordfreq`](https://pypi.org/project/wordfreq/).
