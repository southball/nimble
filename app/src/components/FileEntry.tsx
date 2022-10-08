import { File } from "../api";
import { FontAwesomeIcon } from "@fortawesome/react-fontawesome";
import {
  faFolder,
  faFileLines,
  faFilePdf,
  faFileImage,
  faFileAudio,
  faFileVideo,
  faFileWord,
  IconDefinition,
} from "@fortawesome/free-regular-svg-icons";

interface FileEntryProps {
  file: File;
  href?: string;
  onClick?: (file: File) => any;
  showFullPath?: boolean;
}

const getIcon = (rawFilename: string): IconDefinition => {
  const mappings: [IconDefinition, string[]][] = [
    [faFilePdf, [".pdf"]],
    [faFileImage, [".jpg", ".png", ".bmp"]],
    [faFileAudio, [".mp3", ".aac", ".flac"]],
    [faFileVideo, [".mp4", ".wmv"]],
    [faFileWord, [".doc", ".docx", ".odt"]],
  ];
  const filename = rawFilename.toLowerCase();
  return (
    mappings.find(([_icon, extensions]) =>
      extensions.some((extension) => filename.endsWith(extension))
    )?.[0] ?? faFileLines
  );
};

export const FileEntry = ({
  file,
  href,
  onClick,
  showFullPath,
}: FileEntryProps) => (
  <div>
    <a
      href={href}
      onClick={() => {
        onClick?.(file);
      }}
      style={{
        textDecoration: "none",
        color: "black",
        cursor: "pointer",
        display: "inline-flex",
      }}
    >
      <div>
        {file.type === "directory" ? (
          <FontAwesomeIcon icon={faFolder} fixedWidth />
        ) : (
          <FontAwesomeIcon icon={getIcon(file.filename)} fixedWidth />
        )}
      </div>
      <div className="ps-1" style={{ wordBreak: "break-all" }}>
        {showFullPath
          ? `${file.directory}${file.directory ? "/" : ""}${file.filename}`
          : file.filename}
      </div>
    </a>
  </div>
);
