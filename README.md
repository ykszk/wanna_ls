Wanna `ls`?
===========
![Build status](https://github.com/ykszk/wanna_ls/actions/workflows/rust.yml/badge.svg?branch=main)

I don't wanna `ls` when:

- it takes too long to list the directory because the filesystem is slow (e.g., NFS or Samba)
- too many entries are in the directory because it can clutter the terminal

`wanna_ls` is a command that returns `EXIT_FAILURE` when I don't wanna `ls`.

More specifically, return code will be:
- `0`: No error, it is safe to `ls`
- `1`: Generic error such as IO error
- `2`: Time limit exceeded
- `> 2`: Too many entries with the code being the number of entries

Default configuration:
```toml
time_limit_ms = 50
max_entries = 32
```

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

### Shell completions
```bash
wanna_ls --completions bash > ~/.config/bash_completion.d/wanna_ls
wanna_ls --completions fish > ~/.config/fish/completions/wanna_ls.fish
```
