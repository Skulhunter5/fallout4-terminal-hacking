# Fallout 4 Terminal-Hacking Minigame
The iconic terminal-hacking minigame from Fallout 4, readily available inside your terminal (emulator) of choice.

(Only tested on Linux yet)

## WIP
It's technically playable already, although the difficulty can't be changed yet. It's always 15 words with 4 letters each and 4 attempts for now.
A menu and command line arguments to change the difficulty are WIP.

## Interesting facts
- The lists of random words, which can appear in the minigame are generated at build-time, using the function `top_n_list` from the Python package [`wordfreq`](https://pypi.org/project/wordfreq/).
