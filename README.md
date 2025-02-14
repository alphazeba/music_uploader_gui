# Web uploader
this is the front end for https://github.com/alphazeba/music_uploader_server.
Easily allow drag and drop simplicity for uploading songs to a plex server.  No plex server access, permissions, ssh, command line. (Simplicity for gui users, plex server owner will still need to setup the music uploader server)

Just drag, drop, click upload.

# User configuration
inside the app you will find a Settings.toml file.
Your server admin will need to provide you with a.
- user="your username"
- password="your password"
- server_url="https://website.com/api" (url pointing to music_uploader_server)
    - note: do NOT use a url that is insecure (http)

# how to build
## clone the repos
clone music_uploader_gui & music_uploader_server next to each other so you have a directory that looks like this.
```bash
ls
# music_uploader_gui    music_uploader_server
```
You have to do this because i have not published the libraries. Right now, they link to eachother locally.  If you know what you are doing, you can put these repos anywhere you like and modify the cargo.toml to find the server.

## setup rust
you will need rust in order to build this https://www.rust-lang.org/tools/install

## setup node
use node greater than 18. https://nodejs.org/en/download
If you are able, i would suggest using NVM to manage node. https://github.com/nvm-sh/nvm
```bash
nvm use node
nvm current
# v20.11.0
```

## how to test run
```bash
npm install
npm run tauri dev
```

## how to build installable/executable versions for distribution
```bash
npm install
npm run tauri build
```
