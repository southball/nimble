import React from "react";
import { useSearchApi } from "../api";
import { generateURL } from "../utils";
import { FileEntriesContainer } from "./FileEntriesContainer";
import { FileEntry } from "./FileEntry";
import { FileEntryInfo } from "./FileEntryInfo";

interface SearchDialogProps {
  updatePath: (path: string[]) => any;
  foldAfterDirectoryClicked?: boolean;
}

export const SearchDialog = ({
  updatePath,
  foldAfterDirectoryClicked,
}: SearchDialogProps) => {
  const [query, setQuery] = React.useState("");
  const [folded, setFolded] = React.useState(true);
  const searchResult = useSearchApi(query);

  return (
    <div className="accordion">
      <div className="accordion-item">
        <h2 className="accordion-header">
          <button
            className={`accordion-button py-2 ${folded ? "collapsed" : ""}`}
            type="button"
            onClick={() => setFolded((folded) => !folded)}
          >
            Search
          </button>
        </h2>
        <div className={`accordion-collapse collapse ${!folded ? "show" : ""}`}>
          <div className="accordion-body py-2">
            <input
              className="form-control form-control-sm mb-2"
              type="text"
              placeholder="Search query"
              onChange={(event) => setQuery(event.target.value)}
              value={query}
            />

            {!query ? (
              <FileEntryInfo
                type="info"
                message="Please enter search query above."
              />
            ) : searchResult.error ? (
              <FileEntryInfo type="error" message="Search failed." />
            ) : !searchResult.data ? (
              <FileEntryInfo type="loading" message="Searching..." />
            ) : searchResult.data.length === 0 ? (
              <FileEntryInfo type="info" message="No search result." />
            ) : (
              <FileEntriesContainer>
                {searchResult.data.map((entry) => (
                  <FileEntry
                    showFullPath
                    key={entry.directory + "/" + entry.filename}
                    file={entry}
                    href={
                      entry.type === "file"
                        ? generateURL("/files", entry.directory, entry.filename)
                        : undefined
                    }
                    onClick={(file) => {
                      if (file.type === "directory") {
                        updatePath([
                          ...file.directory
                            .split("/")
                            .filter((fragment) => fragment.length > 0),
                          file.filename,
                        ]);
                        if (foldAfterDirectoryClicked) {
                          setFolded(true);
                        }
                      }
                    }}
                  />
                ))}
              </FileEntriesContainer>
            )}
          </div>
        </div>
      </div>
    </div>
  );
};
