Wanna `ls`?
===========

I don't wanna `ls` when:

- the directory is on network storage because it can be slow.
- the directory contains too many files because it can clutter the terminal.

`wanna_ls` is a command that returns `EXIT_FAILURE` when I don't wanna `ls`.

More specifically, return code will be:
- `0`: No error, allowed filesystem type, and not too many files
- `1`: Generic error such as IO error
- `2`: Unallowed filesystem
- `> 2`: Too many files with the code being the number of files

## Examples
```bash
cdls () {
    \cd "$@" && wanna_ls && ls
}
```

See the command in action:
```bash
echo "Wanna ls?" && wanna_ls && echo yes || echo no
```

Debug output:
```bash
RUST_LOG=debug wanna_ls
```

## Compatibility
- linux: yes
- macos: ?
- windows: no

## Installation
```bash
cargo install cargo install --git https://github.com/ykszk/wanna_ls
```

### Generate default configuration
```bash
mkdir -p ~/.config/wanna_ls
wanna_ls --default-config > ~/.config/wanna_ls/config.toml
```