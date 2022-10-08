import React, { useEffect } from "react";
import { useListApi } from "../api";
import { useSearchParams } from "react-router-dom";
import { SearchDialog } from "./SearchDialog";
import { FileEntry } from "./FileEntry";
import { useRef } from "react";
import { FileEntriesContainer } from "./FileEntriesContainer";
import { FontAwesomeIcon } from "@fortawesome/react-fontawesome";
import { faFolderOpen } from "@fortawesome/free-regular-svg-icons";
import { FileEntryInfo } from "./FileEntryInfo";
import { generateURL } from "../utils";

const encodePath = (pathFragment: string[]) =>
  pathFragment.map((fragment) => encodeURIComponent(fragment)).join("/");
const decodePath = (path: string) =>
  path
    .split("/")
    .map((fragment) => decodeURIComponent(fragment))
    .filter((fragment) => fragment.length > 0);

function App() {
  const [searchParams, setSearchParams] = useSearchParams();
  const [currentPath, setCurrentPath] = React.useState<string[]>(
    decodePath(searchParams.get("path") ?? encodePath([]))
  );
  console.log(currentPath);
  const listResult = useListApi(currentPath);

  const positionRef = useRef<HTMLDivElement>(null);

  const updateCurrentPath = (path: string[]) => {
    setCurrentPath(path);
    const newSearchParams = new URLSearchParams(searchParams);
    newSearchParams.set("path", encodePath(path));
    setSearchParams(newSearchParams);
  };

  useEffect(() => {
    if (encodePath(currentPath) !== searchParams.get("path")) {
      setCurrentPath(decodePath(searchParams.get("path") ?? ""));
    }
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [searchParams.get("path")]);

  return (
    <div className="container">
      <SearchDialog updatePath={updateCurrentPath} foldAfterDirectoryClicked />

      <div className="mt-3"></div>

      <div className="alert alert-secondary py-1 px-2">
        <div style={{ display: "flex" }}>
          <div>
            <FontAwesomeIcon icon={faFolderOpen} fixedWidth />
          </div>
          <div className="ps-1">
            <ol className="breadcrumb mb-0">
              <li
                className="breadcrumb-item"
                onClick={() => updateCurrentPath([])}
                style={
                  currentPath.length > 0
                    ? { textDecoration: "underline", cursor: "pointer" }
                    : {}
                }
              >
                Home
              </li>
              {currentPath.map((fragment, index) => (
                <li
                  className="breadcrumb-item"
                  onClick={() =>
                    updateCurrentPath(currentPath.slice(0, index + 1))
                  }
                  style={
                    index < currentPath.length - 1
                      ? { textDecoration: "underline", cursor: "pointer" }
                      : {}
                  }
                >
                  {fragment}
                </li>
              ))}
            </ol>
          </div>
        </div>
      </div>

      <div ref={positionRef} />

      <FileEntriesContainer>
        {currentPath.length > 0 && (
          <FileEntry
            file={{ directory: "", filename: "..", type: "directory" }}
            onClick={() =>
              updateCurrentPath(currentPath.slice(0, currentPath.length - 1))
            }
          />
        )}
        {listResult.error ? (
          <FileEntryInfo type="info" message="Failed to fetch directory." />
        ) : !listResult.data ? (
          <FileEntryInfo type="loading" message="Loading..." />
        ) : listResult.data?.length === 0 ? (
          <FileEntryInfo type="info" message="Directory is empty." />
        ) : (
          listResult.data.map((entry) => (
            <FileEntry
              key={entry.directory + "/" + entry.filename}
              file={entry}
              href={
                entry.type === "file"
                  ? generateURL("/files", entry.directory, entry.filename)
                  : undefined
              }
              onClick={(file) => {
                if (file.type === "directory")
                  updateCurrentPath([...currentPath, file.filename]);
              }}
            />
          ))
        )}
      </FileEntriesContainer>
    </div>
  );
}

export default App;
