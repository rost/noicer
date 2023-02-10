# nuice

A simple file browser for the terminal. Inspired by `noice`.

- [x] basic navigation using `h/j/k/l`
  - [x] store cursor position for each directory
  - [x] when moving up the tree, move cursor to the directory we came from
- [ ] graceful handling of empty directories
- [ ] support for opening files
  - [ ] config file with supported file types and programs to execute
- [ ] show/hide hidden files using `.`
- [ ] search/filter using `/`
- [ ] add more vim keybindings
  - [ ] `gg`, `G`, `2j`, `5k`, etc.
- [ ] support for opening directories
  - [ ] set PWD for underlying shell?
  - [ ] spawn new shell?
- [ ] support seamlessly opening archive files
  - [ ] `tar`, `gz`, `zip`, etc.
