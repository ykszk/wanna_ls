Wanna `ls`?
===========
![Build status](https://github.com/ykszk/wanna_ls/actions/workflows/rust.yml/badge.svg?branch=main)

I don't wanna `ls` when:

- I'm on network storage because it can be slow.
- too many entries are in the directory because it can clutter the terminal.

`wanna_ls` is a command that returns `EXIT_FAILURE` when I don't wanna `ls`.

More specifically, return code will be:
- `0`: No error, allowed filesystem type, and not too many entries
- `1`: Generic error such as IO error
- `2`: Unallowed filesystem
- `> 2`: Too many entries with the code being the number of entries

## Examples

Checking filesystem type:
```bash
$ cd /mnt/usb
$ wanna_ls && echo yes || echo no
yes

$ cd /mnt/nfs
$ wanna_ls && echo yes || echo no
no
```

Checking the number of entries:
```bash
$ cd /tmp
$ wanna_ls && echo yes || echo no
yes

$ touch $(seq 1 100)
$ wanna_ls && echo yes || echo no
no
```

### Integration with `cd` and `ls`
Bash
```bash
cdls () {
    \cd "$@" && wanna_ls && ls
}
```

Fish
```fish
function cdls
    builtin cd $argv[1]; and wanna_ls; and ls
end
```

### Enable debug output
```bash
RUST_LOG=debug wanna_ls
```

## Installation
```bash
cargo install --git https://github.com/ykszk/wanna_ls
```

### Generate default configuration
```bash
mkdir -p ~/.config/wanna_ls
wanna_ls --default-config > ~/.config/wanna_ls/config.toml
```

## Compatibility
- linux: yes
- macos: ?
- windows: no
