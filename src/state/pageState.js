import { useState } from "react";

export const PAGE_UPLOADER = 69
export const PAGE_SETTINGS = 420;
export const PAGE_SEARCH = 777;
const VALID_PAGES = [
    PAGE_UPLOADER,PAGE_SETTINGS,PAGE_SEARCH,
];

export function usePageState() {
    const [page, setPage] = useState(PAGE_UPLOADER);
    const goToSettings = () => {
        setPage(PAGE_SETTINGS);
    }
    const goToUploader = () => {
        setPage(PAGE_UPLOADER);
    }
    const goToSearch = () => {
        setPage(PAGE_SEARCH);
    }

    const goToPage = (page) => {
        if (!VALID_PAGES.includes(page)) {
            console.error("tried to go to invalid page ", page);
        }
        setPage(page);
    }
    return {
        page,
        goToSettings,
        goToUploader,
        goToSearch,
        goToPage,
    }
}