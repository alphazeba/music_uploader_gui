# Tauri + React

This template should help get you started developing with Tauri and React in Vite.

## Recommended IDE Setup

- [VS Code](https://code.visualstudio.com/) + [Tauri](https://marketplace.visualstudio.com/items?itemName=tauri-apps.tauri-vscode) + [rust-analyzer](https://marketplace.visualstudio.com/items?itemName=rust-lang.rust-analyzer)

# how to build
## clone the repos
clone music_uploader_gui, music_uploader_client, & music_uploader_server next to each other so you have a directory that looks like this.
```bash
❯ ls
music_uploader_client music_uploader_gui    music_uploader_server
```

You have to do this because i have not published the server & client libraries. so they link to eachother locally.  If you know what you are doing, you can put these repos anywhere you like and modify their cargo.toml to find each other.
gui relies on client relies on server.

# how to run 
use node greater than 18.  I use nvm to managed node
```bash
❯ nvm current
v20.11.0
```
```bash
❯ npm run tauri dev
```
will build the app for you.
