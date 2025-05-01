
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