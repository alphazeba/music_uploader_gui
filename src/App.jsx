import "./App.css";
import { useState, useEffect } from "react";
import { getVersion } from "@tauri-apps/api/app";
import { useMessageState } from "./state/messageState";
import { PAGE_SEARCH, PAGE_SETTINGS, PAGE_UPLOADER, usePageState } from "./state/pageState";
import { Uploader } from "./pages/uploader";
import { useUploaderState } from "./state/uploaderState";
import { Settings } from "./pages/settings";
import { get_startup_message, run_settings_checks } from "./backendApi";
import { Search } from "./pages/search";
import { useSearchState } from "./state/searchState";

function App() {
    const messageState = useMessageState();
    const pageState = usePageState();
    const uploaderState = useUploaderState(messageState.guiLog);
    const searchState = useSearchState(messageState.guiLog);

    const [appVersion, setAppVersion] = useState("");

    useEffect(() => {
        getVersion()
            .then(version => setAppVersion(version));
        get_startup_message()
            .then(message => messageState.guiLog(message));
        run_settings_checks()
            .then(result => messageState.guiLog(result));
        return () => {};
    }, []);

    const renderPage = () => {
        switch (pageState.page) {
            case PAGE_UPLOADER:
                return <Uploader
                    uploaderState={uploaderState}
                    pageState={pageState}
                />;
            case PAGE_SETTINGS:
                return <Settings
                    guiLog={messageState.guiLog}
                    pageState={pageState}
                />;
            case PAGE_SEARCH:
                return <Search
                    pageState={pageState}
                    searchState={searchState}
                />;
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
