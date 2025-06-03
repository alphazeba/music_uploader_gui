
import { invoke } from "@tauri-apps/api/core";

export async function upload_album(album, songs) {
    return await invoke("upload_album", {
        albumName: album.name,
        albumId: album.id.toString(),
        artist: album.artist,
        songs,
    });
}

export async function generate_guid() {
    return await invoke("generate_guid");
}

export async function get_valid_extensions() {
    return await invoke("get_valid_extensions");
}

export async function get_startup_message() {
    return await invoke("get_startup_message");
}

export async function reload_settings() {
    return await invoke("reload_settings");
}

export async function run_settings_checks() {
    return await invoke("run_settings_checks")
}

export async function get_settings() {
    return await invoke("get_settings");
}

export async function save_settings(user, password, url) {
    return await invoke("save_settings", {
        user,
        password,
        url,
    });
}

export async function album_search(album) {
    return await invoke("album_search", {
        album,
    });
}