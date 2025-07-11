# dir-nuke

<p align="center">
  <img src="logo.png" width="300" alt="Locker Bun Logo" style="border-radius: 25px; box-shadow: 0 4px 8px rgba(0, 0, 0, 0.2); padding: 10px;"/>
</p>


## Table of Contents
- [About](#about)
- [Important Information](#important-information)

## About
This project is a directory nuker. It is designed to safely and efficiently remove specified directories and their contents.

## Important Information
- **Purpose**: To provide a reliable tool for directory deletion.
- **Safety**: Always double-check the directories you are targeting for deletion, as this operation is irreversible.
- **Development**: This project is written in Rust.


## Usage

### Cloning the project

```bash
cargo run ~/Projects
```

### TUI Keybindings

| Keybinding        | Action                               |
|-------------------|--------------------------------------|
| `Down`, `j`, `Tab`| Move down on the list                |
| `Up`, `k`, `BackTab`| Move up on the list                  |
| `Space`           | Toggle selection of a directory      |
| `h`               | Unselect current item                |
| `l`               | Select current item                  |
| `Enter`           | Confirm and delete selected directories |
| `Esc`, `q`        | Cancel application and exit          |
