import { useState, useRef, useEffect } from "react";
import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";
import { open } from '@tauri-apps/plugin-dialog';
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
    const [filesHovering, setFilesHovering] = useState(false);

    // i want this to run just once
    useEffect(() => {
        const unlistenBackEndLog = listen("music_uploader://log", (event) => {
            guiLog(event.payload);
        });
        const unlistenFileDropHover = listen("tauri://drag-enter", (event) => {
            setFilesHovering(true);
        });
        const unlistenFileDropCancel = listen("tauri://drag-leave", (event) => {
            setFilesHovering(false);
        });
        const unlistenFileDrop = listen("tauri://drag-drop", (event) => {
            setFilesHovering(false);
            handleNewFilePaths(event.payload.paths);
        });
        invoke("get_startup_message")
            .then(message => guiLog(message));
        return () => {
            unlistenBackEndLog.then(f => f());
            unlistenFileDropHover.then(f => f());
            unlistenFileDropCancel.then(f => f());
            unlistenFileDrop.then(f => f());
        };
    }, []);

    useEffect(() => {
        if (state === s_selecting && allFilesSuccessfullyUploaded()) {
            trigger_scan();
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
                guiLog("skipping " + file.name + " as it was already uploaded");
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

    async function trigger_scan() {
        guiLog("Requesting a scan")
        let result = await invoke("trigger_scan");
        guiLog("finished requesting scan: " + result);
    }

    async function generate_guid() {
        return await invoke("generate_guid");
    }

    async function get_valid_extensions() {
        return await invoke("get_valid_extensions");
    }

    const fileToSong = async (fileState) => {
        return {
            song_name: fileState.name,
            path: fileState.path,
        };
    }

    const guiLog = (text) => {
        console.log(text);
        outputMessage.current = [{
            msg: text,
            ts: new Date(Date.now()).toLocaleTimeString(),
        }, ...outputMessage.current];
        setMessageCount(outputMessage.current.length);
    }

    const handleNewFilePaths = async(paths) => {
        const validExtensions = await get_valid_extensions();
        if (paths && paths.length > 0) {
            let newFiles = [];
            for (let path of paths) {
                if (!validExtensions.includes(getExtension(path))) {
                    guiLog("skipping " + path + " because it does not have a valid extension");
                    continue;
                }
                newFiles.push(await pathToFilePathState(path));
            }
            setFiles((prevFiles) => [...prevFiles, ...newFiles]);
        }
    }

    const fs_not_uploaded = 69;
    const fs_uploaded = 420;
    const fs_failed_upload = 333;
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

    const handleOpenFileSelector = async () => {
        const paths = await open({
            multiple: true,
            directory: false,
        });
        handleNewFilePaths(paths);
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
                            <div
                                className={"dropzone interactable item" + (filesHovering ? " dropzoneActive" : "")}
                                onClick={handleOpenFileSelector}
                            >
                                <p>
                                drop zone
                                </p>
                                <p className="fineprint">(or click to browse)</p>
                            </div>
                        </div>
                        <button className="interactable" disabled={state != s_selecting} type="submit">{(state == s_selecting ? "upload" : "yeehaw")}</button>
                    </form>
                    <div>{outputMessage.current.map((message, index) => 
                        <div className="logline" key={index}>{message.ts} : {message.msg}</div>)}</div>
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
                    <div className="songItem" key={f.name}>
                        {renderFileStatus(f.state)} {f.name}
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
            case fs_uploaded: return "✅";
        }
        return "error💀";
    }
    return (
        <div className="columnContainer">
            {renderFileView()}
            {renderMainPanel()}
        </div>
    );
}

export default App;
