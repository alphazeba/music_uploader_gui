import { PageChooser } from "../bits/pageChooser";

export function Search({pageState, searchState}) {

    const renderOutput = (title, content) => {
        if (content === "") {
            return null;
        }
        return <div className="searchResult">
            {title}: {content}
        </div>
    }

    return (
        <div>
            <div className="uploaderTabContainer settingsTabBar">.</div>
            <div className="container">
                <h1 className="title">Search</h1>
                <p>who uploaded this?</p>
                <PageChooser pageState={pageState} />
                <form
                    onSubmit={(e) => {
                        e.preventDefault();
                        searchState.searchForAlbum();
                    }}
                >
                    <div className="row buttspace">
                            <span className="emoji">
                                💽
                            </span>
                            <input
                                className="interactable"
                                onChange={(e) => searchState.setAlbumSearch(e.currentTarget.value)}
                                placeholder="Enter album..."
                                value={searchState.albumSearch}
                            />
                            <span className="emoji">
                                📁
                            </span>
                    </div>
                    <button
                        className="interactable"
                        disabled={!searchState.canSearch()}
                    >{searchState.canSearch() ? "check" : "searching"}</button>
                </form>
                <div className="searchOutput">
                    {renderOutput("found album", searchState.foundAlbum)}
                    {renderOutput("uploader", searchState.uploader)}
                </div>
            </div>
        </div>
    )
}