import "./App.css";
import { useState, useEffect } from "react";
import { getVersion } from "@tauri-apps/api/app";
import { invoke } from "@tauri-apps/api/core";
import { useMessageState } from "./state/messageState";
import { PAGE_SETTINGS, PAGE_UPLOADER, usePageState } from "./state/pageState";
import { Uploader } from "./pages/uploader";
import { useUploaderState } from "./state/uploaderState";

function App() {
    const messageState = useMessageState();
    const pageState = usePageState();
    const uploaderState = useUploaderState(messageState.guiLog);

    const [appVersion, setAppVersion] = useState("");

    useEffect(() => {
        getVersion()
            .then(version => setAppVersion(version));
        invoke("get_startup_message")
            .then(message => messageState.guiLog(message));
        return () => {};
    }, []);

    const renderPage = () => {
        switch (pageState.page) {
            case PAGE_UPLOADER:
                return <Uploader
                    uploaderState={uploaderState}
                />;
            case PAGE_SETTINGS:
                return <div>settings</div>;
        }
    }

    const renderMessages = () => {
        return messageState.message.map((message, index) =>
            <div className="logline" key={index}><span className="logTs">({message.ts})</span> {message.msg}</div>
        );
    }

    return <div className="container">
        <div className="version">v{appVersion}</div>
        {renderPage()}
        <div className="logSection">
            {renderMessages()}
        </div>
    </div>
}

export default App;
