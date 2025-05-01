import { listen } from "@tauri-apps/api/event";
import { useState, useRef, useEffect } from "react";

export function useMessageState() {
    const outputMessage = useRef([]);
    const [message, setMessageCount] = useState(0);

    useEffect(() => {
        const unlistenBackEndLog = listen("music_uploader://log", (event) => {
            guiLog(event.payload);
        });
        return () => {
            unlistenBackEndLog.then(f => f());
        };
    }, []);

    const guiLog = (text) => {
        console.log(text);
        outputMessage.current = [{
            msg: text,
            ts: new Date(Date.now()).toLocaleString(),
        }, ...outputMessage.current];
        setMessageCount(outputMessage.current.length);
    }
    const clear = () => {
        outputMessage.current = [];
        setMessageCount(0);
    }
    return {
        message: outputMessage.current,
        guiLog,
        clear,
    }
}