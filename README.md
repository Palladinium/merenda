# Merenda - Byte-sized snacks over SSH.

A tiny server/client application to synchronize clipboards over SSH in Rust, inspired by similar tools like [Lemonade](https://github.com/pocke/lemonade).

# Installation

On Arch Linux, you can install `merenda` from the AUR. For example, with yay:
```
yay merenda
```

Or you can install it through cargo:
```
cargo install merenda
```

## Local setup

After installation, run `merenda server` on your main machine - the one you'll use to SSH onto other systems. It will listen for connections requesting to read/write your clipboard, and you'll generally want to have it running in the background. On its own, it also works as a roundabout way to read/write your clipboard, by using `merenda get` and `merenda set`.

The Arch package comes with a systemd user unit, which you can activate with `systemctl --user start merenda.service` and enable at boot with `systemctl --user enable merenda.service`.
By default, the server will listen on `127.0.0.1`, port 3660. You can change either with the `-H/--address` and `-p/--port` arguments.

*CAUTION: Think carefully before changing the default listen address*. Merenda has absolutely no authentication, so listening on anything other than loopback will allow anyone able to reach that IP address to read/write to your clipboard at will. A safer alternative is to set up an SSH local reverse port forwarding, so that the port is exposed only on the other end of the SSH connection. More on that later.

## Remote setup

After installation, set up your preferred editors/tools to use merenda to store data to the clipboard when an SSH connection is available.
For example, I use NeoVim and my config looks like this:
```
.
```

## Forwarding the 

