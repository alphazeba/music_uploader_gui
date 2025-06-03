import { useState } from "react";
import { album_search } from "../backendApi";

export function useSearchState(guiLog) {
    const [searching, setSearching] = useState(false);
    const [albumSearch, setAlbumSearch] = useState("");
    const [foundAlbum, setFoundAlbum] = useState("");
    const [uploader, setUploader] = useState("");

    const searchForAlbum = () => {
        if (!canSearch()) {
            return;
        }
        // query the backend.
        setSearching(true);
        guiLog("searching for " + albumSearch);
        album_search(albumSearch)
            .then((result) => {
                if (result.album == undefined) {
                    guiLog("there was some issue");
                    guiLog(result);
                    return;
                }
                guiLog("found " + result.album);
                setFoundAlbum(result.album);
                setUploader(result.uploader);
            })
            .finally(() => setSearching(false));
    }

    const canSearch = () => {
        return !searching;
    }

    return {
        albumSearch,
        foundAlbum,
        uploader,

        setAlbumSearch,
        searchForAlbum,
        canSearch,
    }
}