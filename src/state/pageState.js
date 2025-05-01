import { useState } from "react";

export const PAGE_UPLOADER = 69
export const PAGE_SETTINGS = 420;

export function usePageState() {
    const [page, setPage] = useState(PAGE_UPLOADER);
    const goToSettings = () => {
        setPage(PAGE_SETTINGS);
    }
    const goToUploader = () => {
        setPage(PAGE_UPLOADER);
    }
    return {
        page,
        goToSettings,
        goToUploader,
    }
}