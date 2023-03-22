# noicer

A simple file browser for the terminal. Inspired by `noice`.

- [x] basic navigation using `h/j/k/l`
  - [x] store cursor position for each directory
  - [x] when moving up the tree, move cursor to the directory we came from
- [x] graceful handling of empty directories
- [ ] support for opening files
  - [x] pager support for text files (default behaviour for files)
    - defaults to bat using `--paging=always` in config to enable paging
  - [x] editor support for text files
  - [ ] config file with supported file types and programs to execute
- [x] show/hide hidden files using `.`
- [x] sort by dir, name, size, time using `d`, `n`, `s`, `t`
- [x] case sensitive sorting using `i`
- [x] search using `/`
- [ ] filter using `:g/term`
- [ ] add more vim keybindings
  - [x] `gg`
  - [x] `G`
  - [x] `2j`
  - [x] `5k`
- [x] support for opening directories
  - [ ] ~~set PWD for underlying shell?~~
  - [x] spawn new shell?
- [ ] support seamlessly opening archive files
  - [ ] `tar`, `gz`, `zip`, etc.
