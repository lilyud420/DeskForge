
# DeskForge

A simple TUI launcher editor for Linux no one asked for 

## Usage
```htpt
Usage: deskforge [COMMANDS] [OPTIONS]

Options:
  -n, --new [<OPTIONAL: FILE_NAME>]   Create a new launcher
  -e, --edit [<REQUIRED: FILE_NAME>]  Edit an existing launcher
  -r, --remove [<REQUIRED: FILE_NAME>]  Remove an exisiting launcher
  -l, --list                          List all exisiting launcher
  -h, --help                          Print help
  -V, --version                       Print version
```

## Keymaps
```htpt
Mode: NORMAL

gg                                    Go to name
G                                     Go to save
j                                     Go down
k                                     Go up
dd                                    Delete line
i                                     Insert 
q                                     Quit

Mode: INSERT
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


## Screenshot

![App Screenshot](https://raw.githubusercontent.com/lilyud420/DeskForge/refs/heads/logic/showcase/tui.png)

## File Structure
```htpt
src/    
├── app/
│   ├── event.rs        # Handle events & key input
│   ├── mod.rs          
│   ├── state.rs        # App state management
│   └── ui.rs           # UI rendering
│
├── commands/           # CLI commands
│   ├── edit.rs
│   ├── list.rs
│   ├── mod.rs
│   ├── new.rs 
│   └── remove.rs
│
├── utils/
│   ├── constants.rs    # Constant declaration
│   └── mod.rs
│
├── main.rs
└── cli.rs  
```
## Notes

- Make sure your **PATH** includes **$HOME/.local/bin**. You can add this to your shell configuration:
  - Bash (**~/.bashrc**): 
      ```htpt
      export PATH="$HOME/.local/bin:$PATH"
      ```
  - Zsh (**~/.zshrc**):
      ```htpt
      export PATH="$HOME/.local/bin:$PATH"
      ```  
- The executable is installed at: 
  ```htpt
  $HOME/.local/bin/deskforge 
  ```
- Optional .desktop file:
  ```htpt
  $HOME/.local/share/applications
  ```
## License

[MIT](https://choosealicense.com/licenses/mit/)

