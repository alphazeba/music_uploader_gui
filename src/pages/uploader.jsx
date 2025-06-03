import { useState, useEffect } from "react";
import { listen } from "@tauri-apps/api/event";
import { open } from '@tauri-apps/plugin-dialog';
import { fs_failed_upload, fs_not_uploaded, fs_uploaded, fs_uploading, s_in_queue, s_selecting, s_upload_fail, s_upload_success, s_uploading } from "../state/uploaderState";
import { PageChooser } from "../bits/pageChooser";

export function Uploader({uploaderState, pageState}) {
    const [filesHovering, setFilesHovering] = useState(false);

    useEffect(() => {
        const unlistenFileDropHover = listen("tauri://drag-enter", (event) => {
            setFilesHovering(true);
        });
        const unlistenFileDropCancel = listen("tauri://drag-leave", (event) => {
            setFilesHovering(false);
        });
        const unlistenFileDrop = listen("tauri://drag-drop", (event) => {
            setFilesHovering(false);
            const paths = event.payload.paths;
            console.log(JSON.stringify(paths));
            console.log(uploaderState.selectedAlbumKey);
            uploaderState.addSongPaths(paths);
        });
        return () => {
            unlistenFileDropHover.then(f => f());
            unlistenFileDropCancel.then(f => f());
            unlistenFileDrop.then(f => f());
        };
    }, [uploaderState.selectedAlbumKey]);

    const handleOpenFileSelector = async () => {
        const paths = await open({
            multiple: true,
            directory: false,
        });
        console.log(JSON.stringify(paths));
        console.log(uploaderState.selectedAlbumKey);
        uploaderState.addSongPaths(paths);
    }

    const renderFileStatus = (status) => {
        switch (status) {
            case fs_not_uploaded: return "⚪️";
            case fs_uploading: return "😤";
            case fs_uploaded: return "✅";
            case fs_failed_upload: return "❌";
        }
        return "error💀";
    }

    const renderAlbumStatus = (status) => {
        switch (status) {
            case s_selecting: return "⚪️";
            case s_in_queue: return "💤";
            case s_uploading: return "😤";
            case s_upload_success: return "✅";
            case s_upload_fail: return "❌";
        }
        return "error💀";
    }

    const renderMainPanel = () => {
        return (
            <div className="leftColumn">
                <main className="container">
                    <h1 className="title"><img className="logo logoLeft" src="cloud.svg"/> Music Uploader <img className="logo logoRight" src="cloud.svg"/></h1>
                    <p>Put your music all the way into me.</p>
                    <PageChooser pageState={pageState} />
                    <form
                        className="uploadForm"
                        onSubmit={(e) => {
                            e.preventDefault();
                            uploaderState.uploadAlbum(uploaderState.selectedAlbumKey);
                        }}
                    >
                        <div className="row buttspace">
                            <span className="emoji">
                                💽
                            </span>
                            <input
                                className="interactable"
                                onChange={(e) => uploaderState.setAlbum(e.currentTarget.value)}
                                placeholder="Enter album..."
                                value={uploaderState.album}
                            />
                            <span className="emoji">
                                📁
                            </span>
                        </div>
                        <div className="row buttspace">
                            <span className="emoji">
                                🧜‍♂️
                            </span>
                            <input
                                className="interactable"
                                onChange={(e) => uploaderState.setArtist(e.currentTarget.value)}
                                placeholder="Enter artist..."
                                value={uploaderState.artist}
                            />
                            <span className="emoji">
                                👩‍🎨
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
                        <button className="interactable" disabled={!canPressUpload()} type="submit">{(canPressUpload() ? "upload" : "yeehaw")}</button>
                    </form>
                </main>
            </div>
        );
    }

    const canPressUpload = () => {
        let state = uploaderState.state;
        return state != s_uploading && state != s_in_queue;
    }

    const renderFileView = () => {
        let files = uploaderState.files;
        return (
            <div className="rightColumn">
                <h3 className="songItem">files to upload</h3>
                {(files.length == 0 ? <p>looks a bit empty</p> : <div/>)}
                {files.map(f =>
                    <div className="songItem" key={f.id}>
                        {renderFileStatus(f.state)} {f.name}
                    </div>)}
            </div>
        );
    }

    const renderTabBar = () => {
        return <div className="uploaderTabContainer">
            {renderAlbumTabs()}
        </div>
    }

    const renderAlbumTabs = () => {
        let output = []
        for (let key in uploaderState.albums) {
            let album = uploaderState.albums[key];
            let className = "uploaderTab" + (
                (album.id == uploaderState.selectedAlbumKey) ? " selected" : "");
            output.push(<span
                key={key}
                className={className}
                onClick={()=>uploaderState.selectAlbumId(album.id)}
            >
                {renderAlbumStatus(album.state)} {(album.name == "") ? "New Album" : album.name} {renderCloseAlbum(album.id)}
            </span>)
        }
        output.push(renderNewTabButton());
        return output;
    }

    const renderCloseAlbum = (albumId) => {
        return <span
            className="uploaderTabExit"
            onClick={(e) => {
                e.stopPropagation();
                uploaderState.closeAlbumId(albumId)
            }}
        >🆇</span>;
    }

    const renderNewTabButton = () => {
        return <span
            className="uploaderTab"
            onClick={uploaderState.createAndSelectNewAlbum}
        >🐣</span>
    }

    return (
        <div>
            {renderTabBar()}
            <div className="columnContainer">
                {renderFileView()}
                {renderMainPanel()}
            </div>
        </div>
    );
}