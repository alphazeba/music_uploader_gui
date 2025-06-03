import { PAGE_SEARCH, PAGE_SETTINGS, PAGE_UPLOADER } from "../state/pageState"

export const PageChooser = ({pageState}) => {
    const renderButton = (page, emoji) => {
        if (pageState.page == page) {
            return (
                <div
                    className="pageChooserButton selected"
                >{emoji}</div>
            )
        }
        return (
            <div
                className="pageChooserButton"
                onClick={() => pageState.goToPage(page)}
            >{emoji}</div>
        )
    }
    return (
        <div className="pageChooser">
            {renderButton(PAGE_UPLOADER, "☁️")}
            {renderButton(PAGE_SEARCH, "🔎")}
            {renderButton(PAGE_SETTINGS, "⚙️")}
        </div>
    )
}