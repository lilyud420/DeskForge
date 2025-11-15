
# DeskForge

A simple TUI launcher editor for Linux no one asked for 

## Usage
```htpt
Usage: deskforge [COMMANDS] [OPTIONS]

Options:
  -n, --new [<OPTIONAL: FILE_NAME>]   Create a new launcher
  -e, --edit [<REQUIRED: FILE_NAME>]  Edit an existing launcher
  -l, --list                          List all exisiting launcher
  -h, --help                          Print help
  -V, --version                       Print version
```

## Installation
#### Cargo (required)
```bash
curl https://sh.rustup.rs -sSf | sh
```

#### Makefile:
```bash
git clone https://github.com/lilyud420/DeskForge.git
cd DeskForge
make install
``` 

#### Build from source:
```bash
git clone https://github.com/lilyud420/DeskForge.git
cd DeskForge
cargo build --release
``` 
#### Uninstall
```htpt
make uninstall
``` 
or
```htpt
rm -f ~/.local/bin/deskforge
```


## Screenshots

![App Screenshot](https://via.placeholder.com/468x300?text=App+Screenshot+Here)


## License

[MIT](https://choosealicense.com/licenses/mit/)

