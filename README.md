# nuice

A simple file browser for the terminal. Inspired by `noice`.

- [x] basic navigation using `h/j/k/l`
  - [x] store cursor position for each directory
  - [x] when moving up the tree, move cursor to the directory we came from
- [x] graceful handling of empty directories
- [ ] support for opening files
  - [ ] config file with supported file types and programs to execute
- [x] show/hide hidden files using `.`
- [x] search using `/`
- [ ] filter using `:g/term`
- [ ] add more vim keybindings
  - [x] `gg`
  - [x] `G`
  - [ ] `2j`
  - [ ] `5k`
- [ ] support for opening directories
  - [ ] set PWD for underlying shell?
  - [ ] spawn new shell?
- [ ] support seamlessly opening archive files
  - [ ] `tar`, `gz`, `zip`, etc.
