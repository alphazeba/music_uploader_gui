import { useEffect, useState } from "react";
import { get_settings, run_settings_checks, save_settings } from "../backendApi"
import { PageChooser } from "../bits/pageChooser";

export function Settings({guiLog, pageState}) {
    const [user, setUser] = useState("");
    const [password, setPassword] = useState("");
    const [url, setUrl] = useState("");

    useEffect(() => {
        loadSettings();
    }, []);

    const handleGetSettingsResult = (result) => {
        if (!result.success) {
            guiLog("getting settings failed");
            return;
        }
        let settings = result.settings;
        setUser(settings.user);
        setPassword(settings.password);
        setUrl(settings.server_url);
    }

    const handleSave = () => {
        save_settings(user, password, url)
            .then(result => {
                guiLog(result);
                validateAndLoadSettings();
            });
    }

    const validateAndLoadSettings = () => {
        run_settings_checks()
            .then(result => guiLog(result));
        loadSettings();
    }

    const loadSettings = () => {
        get_settings()
            .then(result => handleGetSettingsResult(result));
    }

    return <div>
        <div className="uploaderTabContainer settingsTabBar">.</div>
        <div className="container">
            <h1 className="title">Settings</h1>
            <p>...</p>
            <PageChooser pageState={pageState} />
            <form
                onSubmit={(e) => {
                    e.preventDefault();
                    handleSave();
                }}
            >
                <div className="row buttspace">
                    <span>
                        <span className="settingsLabel">
                            user
                        </span>
                        <input
                            className="interactable"
                            onChange={(e) => setUser(e.currentTarget.value)}
                            placeholder="enter username..."
                            value={user}
                        />
                    </span>
                    
                </div>
                <div className="row buttspace">
                    <span>
                        <span className="settingsLabel">
                            password
                        </span>
                        <input 
                            className="interactable"
                            onChange={(e) => setPassword(e.currentTarget.value)}
                            placeholder="enter password..."
                            value={password}
                            type="password"
                        />
                    </span>
                </div>
                <div className="row buttspace">
                    <span>
                        <span className="settingsLabel">
                            server url
                        </span>
                        <input
                            className="interactable"
                            onChange={(e) => setUrl(e.currentTarget.value)}
                            placeholder="enter server url..."
                            value={url}
                        />
                    </span>
                </div>
                <div className="buttspace">
                    <button type="submit">
                        save
                    </button>
                </div>
            </form>
        </div>
    </div>
}