# group-expenses-back

![Build](https://github.com/Monsieur-Wary/group-expenses-back/workflows/Rust%20build/badge.svg?branch=master)
![Security](https://github.com/Monsieur-Wary/group-expenses-back/workflows/Security%20audit/badge.svg?branch=master)

Backend part of the group-expenses app.
The front part can be found [here](https://github.com/chloeturchi/group-expenses-front).

## How to start the server

### Docker

You first need to build the image.
Then you run it with the following commands.

```Shell
docker build -t group-expenses:local .
docker run -it --rm -p 8000:8000 group-expenses:local
```

### Rust

You need to install Rust first.
Next just build and run the project.

```Shell
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
cargo run
```
