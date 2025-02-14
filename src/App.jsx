import { useState, useRef, useEffect } from "react";
import { invoke } from "@tauri-apps/api/core";
import "./App.css";

const s_selecting = 69;
const s_uploading = 420;

function App() {
    const [state, setState] = useState(s_selecting);
    const outputMessage = useRef([]);
    const [messageCount, setMessageCount] = useState(0);
    const [album, setAlbum] = useState("");
    const [artist, setArtist] = useState("");
    const [files, setFiles] = useState([]);
    const [clearCounter, setClearCounter] = useState(0);

    // i want this to run just once
    useEffect(() => {
        let fn = async () => {
            let message = await invoke("get_startup_message");
            guiLog(message);
        }
        fn();
    }, []);

    useEffect(() => {
        if (state === s_selecting && allFilesSuccessfullyUploaded()) {
            guiLog("Requesting a scan")
            invoke("request_scan").then((result) => {
                guiLog("Succesfully triggered scan");
                guiLog(result);
            }).catch((error) => {
                guiLog("Scan request failed");
                guiLog(error);
            })
        }
    }, [files, state])

    async function upload() {
        let failToUpload = false;
        if (state == s_uploading) {
            guiLog("cannot upload, currently uploading");
            failToUpload = true;
        }
        if (files.length == 0) {
            guiLog("you must select files to upload");
            failToUpload = true;
        }
        if (album === "") {
            guiLog("You must provide an album");
            failToUpload = true;
        }
        if (artist === "") {
            guiLog("You must provide an artist");
            failToUpload = true;
        }
        if (failToUpload) {
            return;
        }
        setState(s_uploading);
        guiLog("beginning upload");
        for (let index in files) {
            let file = files[index];
            if (file.state == fs_uploaded) {
                guiLog("skipping " + file.file.name + " as it was already uploaded");
                continue;
            }
            let song = await fileToSong(file);
            let mutFiles = [...files];
            let result = await invoke("upload_song", { 
                album, artist, song,
            })
            guiLog(result);
            if (result.startsWith("Success")) {
                mutFiles[index].state = fs_uploaded;
            } else {
                mutFiles[index].state = fs_failed_upload;
            }
            setFiles(mutFiles);
        }
        guiLog("done");
        setState(s_selecting);
    }

    function allFilesSuccessfullyUploaded() {
        if (files.length == 0) {
            return false;
        }
        for (let file of files) {
            if (file.state !== fs_uploaded) {
                return false;
            }
        }
        return true;
    }

    async function generate_guid() {
        return await invoke("generate_guid");
    }

    async function get_valid_extensions() {
        return await invoke("get_valid_extensions");
    }

    // async function album_search(album) {
    //     guiLog("searching for " + album);
    //     let result = await invoke("album_search", {album});
    //     console.log("found the following albums: ", result);
    //     result;
    // }

    const fileToSong = async (fileState) => {
        let buffer = await fileState.file.arrayBuffer();
        return {
            song_name: fileState.file.name,
            data: new Uint8Array(buffer),
        };
    }

    const guiLog = (text) => {
        outputMessage.current = [{
            msg: text,
            ts: new Date(Date.now()).toLocaleTimeString(),
        }, ...outputMessage.current];
        setMessageCount(messageCount+1);
    }

    const handleFileChange = (event) => {
        handleNewFiles(event.target.files);
    };

    const handleDrop = (event) => {
        console.log("handling drop event");
        event.preventDefault();
        handleNewFiles(event.dataTransfer.files);
    };

    const handleNewFiles = async(incomingFiles) => {
        const validExtensions = await get_valid_extensions();
        if (incomingFiles && incomingFiles.length > 0) {
            let newFiles = [];
            for (let file of incomingFiles) {
                if (!validExtensions.includes(getExtension(file.name))) {
                    guiLog("skipping " + file.name + " because it does not have a valid extension");
                    continue;
                }
                newFiles.push(await fileToFileState(file));
            }
            setFiles((prevFiles) => [...prevFiles, ...newFiles]);
        }
    }

    const getExtension = (filename) => {
        return filename.split('.').pop();
    }

    const fs_not_uploaded = 69;
    const fs_uploaded = 420;
    const fs_failed_upload = 333;
    const fileToFileState = async (file) => {
        return {
            id: await generate_guid(),
            file: file,
            state: fs_not_uploaded,
        }
    }

    const handleSearchAlbum = () => {
        album_search(album).then((result) => {
            for (let item of result) {
                guiLog(item);
            }
        })
        .catch( e => {
            guiLog(e);
        });
    }

    const renderMainPanel = () => {
        return (
            <div className="leftColumn">
                <main className="container">
                    <h1 className="title"><img className="logo logoLeft" src="cloud.svg"/> Music Uploader <img className="logo logoRight" src="cloud.svg"/></h1>
                    <p>Put your music all the way into me.</p>
                    <form
                        onSubmit={(e) => {
                            e.preventDefault();
                            upload();
                        }}
                    >
                        <div className="row buttspace">
                            <span className="emoji">
                                🧜‍♂️
                            </span>
                            <input
                                className="interactable"
                                onChange={(e) => setArtist(e.currentTarget.value)}
                                placeholder="Enter artist..."
                                value={artist}
                            />
                            <span className="emoji">
                                👩‍🎨
                            </span>
                        </div>
                        <div className="row buttspace">
                            <span className="emoji">
                                💽
                            </span>
                            <input
                                className="interactable"
                                onChange={(e) => setAlbum(e.currentTarget.value)}
                                placeholder="Enter album..."
                                value={album}
                            />
                            <span className="emoji">
                                📁
                            </span>
                        </div>
                        <div className="row buttspace dropzoneContainer">
                            <div className="dropzone interactable item">
                                <p>
                                drop zone
                                </p>
                                <p className="fineprint">(or click to browse)</p>
                                <input
                                    className="fileInput"
                                    type="file"
                                    id="browse"
                                    onChange={handleFileChange}
                                    onDragOver={(event) => { 
                                        console.log("dragover triggered");
                                        event.stopPropagation();
                                        event.preventDefault(); }}
                                    onDrop={handleDrop}
                                    accept="audio/*"
                                    multiple
                                    key={clearCounter}
                                />
                            </div>
                        </div>
                        <button className="interactable" disabled={state != s_selecting} type="submit">{(state == s_selecting ? "upload" : "yeehaw")}</button>
                    </form>
                    <p>{outputMessage.current.map((message, index) => 
                        <div className="logline" key={index}>{message.ts} : {message.msg}</div>)}</p>
                </main>
            </div>
        );
    }

    const onClear = () => {
        setFiles([]);
        setArtist("");
        setAlbum("");
        outputMessage.current = [];
        // this sets the key on the input to a new one so react rebuilds it.
        // this causes the file contents to visibily clear.
        setClearCounter(clearCounter + 1);
    }

    const renderFileView = () => {
        return (
            <div className="rightColumn">
                <h3 className="songItem">files to upload</h3>
                {(files.length == 0 ? <p>looks a bit empty</p> : <div/>)}
                {files.map(f => 
                    <div className="songItem" key={f.file.name}>
                        {renderFileStatus(f.state)} {f.file.name}
                    </div>)}
                <div className="clearButtonContainer">
                    <button className="clearButton interactable" onClick={onClear}>clear</button>
                </div>
            </div>
        );
    }

    const renderFileStatus = (status) => {
        switch (status) {
            case fs_failed_upload: return "❌";
            case fs_not_uploaded: return "⚪️";
            case fs_uploaded: return "👍";
            default: "wtf is this state?";
        }
    }
    return (
        <div className="columnContainer">
            {renderFileView()}
            {renderMainPanel()}
        </div>
    );
}

export default App;
