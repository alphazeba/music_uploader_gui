import { useState, useEffect } from "react";
import { listen } from "@tauri-apps/api/event";
import { generate_guid, get_valid_extensions, upload_album } from "../backendApi";

export const s_selecting = 69;
export const s_in_queue = 419;
export const s_uploading = 420;
export const s_upload_success = 8008;
export const s_upload_fail = 1134;

export const fs_not_uploaded = 69;
export const fs_uploading = 123;
export const fs_uploaded = 420;
export const fs_failed_upload = 333;


function newAlbumWithIndex(index) {
    return {
        id: index,
        name: "",
        artist: "",
        files: [],
        state: s_selecting,
    }
}
const defaultAlbumKey = 0;
// this is duplicated because using a function to initialize it caused the used value to be an empty object
const defaultAlbumsState = {
    0: {
        id: defaultAlbumKey,
        name: "",
        artist: "",
        files: [],
        state: s_selecting,
    }
}

export function useUploaderState(guiLog) {
    const [albums, setAlbums] = useState(defaultAlbumsState);
    const [selectedAlbumKey, setSelectedAlbumKey] = useState(defaultAlbumKey);
    const [nextAlbumIndex, setNextAlbumIndex] = useState(1);
    const [uploadQueueKeys, setUploadQueueKeys] = useState([]);
    const [currentlyUploading, setCurrentlyUploading] = useState(false);

    function findFileIndexWithPath(files, path) {
        for (let i in files) {
            let file = files[i];
            if (file.path == path) {
                return i;
            }
        }
        return null;
    }

    useEffect(() => {
        const unlistenFileReport = listen("music_uploader://file_report", (event) => {
            let fileReport = JSON.parse(event.payload);
            let nextState = fileReport.success ? fs_uploaded : fs_failed_upload;
            let targetPath = fileReport.file;
            setFileState(fileReport.album_id, targetPath, nextState);
            guiLog(fileReport.message);
        });
        const unlistenAlbumReport = listen("music_uploader://album_report", (event) => {
            let albumReport = JSON.parse(event.payload);
            if (albumReport.success) {
                setAlbumState(albumReport.album_id, s_upload_success);
                guiLog(albumReport.message);
            } else {
                setAlbumState(albumReport.album_id, s_upload_fail);
                guiLog(albumReport.message);
            }
        });
        const unlistenAlbumIsUploading = listen("music_uploader://album_is_uploading", (event) => {
            setAlbumState(event.payload, s_uploading);
        });
        const unlistenFileIsUploading = listen("music_uploader://file_is_uploading", (event) => {
            let payload = JSON.parse(event.payload);
            setFileState(payload.album_id, payload.file, fs_uploading);
        });
        return () => {
            unlistenFileReport.then(f => f());
            unlistenAlbumReport.then(f => f());
            unlistenAlbumIsUploading.then(f => f());
            unlistenFileIsUploading.then(f => f());
        };
    }, []);

    useEffect(() => {
        if (!currentlyUploading && uploadQueueKeys.length > 0) {
            setCurrentlyUploading(true);
            let nextUploadKey = uploadQueueKeys[0];
            setUploadQueueKeys(prevKeys => {
                let mutableKeys = clone(prevKeys);
                mutableKeys.shift();
                return mutableKeys;
            });
            let album = albums[nextUploadKey];
            let songs = album.files.map(file => fileToSong(file));
            upload_album(album, songs).then(result => {
                setCurrentlyUploading(false);
                guiLog(result);
            });
        }
    }, [currentlyUploading, uploadQueueKeys])

    function createAndSelectNewAlbum() {
        setAlbums(prevAlbums => {
            let mutableAlbums = clone(prevAlbums);
            mutableAlbums[nextAlbumIndex] = newAlbumWithIndex(nextAlbumIndex);
            return mutableAlbums;
        });
        setSelectedAlbumKey(nextAlbumIndex);
        setNextAlbumIndex(nextAlbumIndex + 1);
    }

    async function uploadAlbum(albumId) {
        let album = albums[albumId];
        let failToUpload = false;
        if (album === undefined) {
            guiLog("Cannot upload album, there was an error the album id does not seem to exist. embarassing");
            return;
        }
        if (album.files.length == 0) {
            guiLog("you must select files to upload");
            failToUpload = true;
        }
        if (album.name === "") {
            guiLog("You must provide an album");
            failToUpload = true;
        }
        if (album.artist === "") {
            guiLog("You must provide an artist");
            failToUpload = true;
        }
        if (failToUpload) {
            return;
        }
        setAlbumState(albumId, s_in_queue);
        setUploadQueueKeys(prevKeys => {
            let mutableKeys = clone(prevKeys);
            mutableKeys.push(albumId);
            return mutableKeys;
        });
    }

    function setAlbum(albumName) {
        setAlbums(prevAlbums => {
            let mutableAlbums = clone(prevAlbums);
            mutableAlbums[selectedAlbumKey].name = albumName;
            return mutableAlbums;
        })
    }

    function setArtist(artist) {
        setAlbums(prevAlbums => {
            let mutableAlbums = clone(prevAlbums);
            mutableAlbums[selectedAlbumKey].artist = artist;
            return mutableAlbums;
        });
    }

    function setAlbumState(albumId, state) {
        setAlbums(prevAlbums => {
            let mutableAlbums = clone(prevAlbums);
            mutableAlbums[albumId].state = state;
            return mutableAlbums;
        })
    }

    function setFileState(albumId, filePath, state) {
        setAlbums(prevAlbums => {
            let mutableAlbums = clone(prevAlbums);
            let mutableAlbum = mutableAlbums[albumId];
            let fileIndex = findFileIndexWithPath(mutableAlbum.files, filePath);
            if (fileIndex !== null) {
                mutableAlbum.files[fileIndex].state = state;
            }
            return mutableAlbums;
        });
    }

    function clone(thing) {
        return JSON.parse(JSON.stringify(thing));
    }

    const fileToSong = (fileState) => {
        return {
            song_name: fileState.name,
            path: fileState.path,
        };
    }

    const addSongPaths = async(paths) => {
        const validExtensions = await get_valid_extensions();
        if (paths && paths.length > 0) {
            let newFiles = [];
            for (let path of paths) {
                if (!validExtensions.includes(getExtension(path))) {
                    guiLog("skipping " + path + " because it does not have a valid extension");
                    continue;
                }
                if (findFileIndexWithPath(albums[selectedAlbumKey].files, path) != null) {
                    guiLog("skipping " + path + " becasue it is already listed");
                    continue;
                }
                newFiles.push(await pathToFilePathState(path));
            }
            setAlbums(prevAlbums => {
                let mutableAlbums = clone(prevAlbums);
                let prevFiles = mutableAlbums[selectedAlbumKey].files;
                mutableAlbums[selectedAlbumKey].files = [...prevFiles, ...newFiles];
                return mutableAlbums;
            })
        }
    }

    const pathToFilePathState = async (path) => {
        return {
            id: await generate_guid(),
            name: getFileName(path),
            path: path,
            state: fs_not_uploaded,
        }
    }

    const getExtension = (filename) => {
        return filename.split(".").pop();
    }

    const getFileName = (path) => {
        return path.split("\\").pop().split("/").pop();
    }

    function selectAlbumId(albumId) {
        setSelectedAlbumKey(albumId);
    }

    function closeAlbumId(albumId) {
        let album = albums[albumId];
        if (album.state == s_in_queue || album.state == s_uploading) {
            guiLog("Cannot close album, it is uploading");
            return;
        }
        setAlbums(prevAlbums => {
            let mutableAlbums = clone(prevAlbums);
            delete mutableAlbums[albumId];
            if (Object.keys(mutableAlbums).length == 0) {
                return defaultAlbumsState;
            }
            return mutableAlbums;
        });
        if (albumId == selectedAlbumKey) {
            selectADifferentAlbumOrDefault(albumId);
        }
    }

    function selectADifferentAlbumOrDefault(notAlbumId) {
        for (let id in albums) {
            if (id != notAlbumId) {
                setSelectedAlbumKey(id);
                return;
            }
        }
        setSelectedAlbumKey(defaultAlbumKey);
    }

    function getAlbumKey(key, defaultValue) {
        if (albums[selectedAlbumKey] !== undefined) {
            return albums[selectedAlbumKey][key];
        }
        return defaultValue;
    }

    return {
        state: getAlbumKey("state", s_upload_fail),
        album: getAlbumKey("name", ""),
        artist: getAlbumKey("artist", ""),
        files: getAlbumKey("files", []),

        albums,
        selectedAlbumKey,

        uploadAlbum,
        addSongPaths,
        setAlbum,
        setArtist,

        selectAlbumId,
        closeAlbumId,
        createAndSelectNewAlbum,
    }
}