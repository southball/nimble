import { FontAwesomeIcon } from "@fortawesome/react-fontawesome";
import {
  faCommentDots,
  faCircleExclamation,
} from "@fortawesome/free-solid-svg-icons";

interface FileEntryInfoProps {
  type: "error" | "info" | "loading";
  message: string;
}

export const FileEntryInfo = (props: FileEntryInfoProps) => (
  <div style={{ display: "flex" }}>
    <div style={{ width: "20px" }}>
      {props.type === "info" ? (
        <FontAwesomeIcon icon={faCommentDots} fixedWidth />
      ) : props.type === "error" ? (
        <FontAwesomeIcon icon={faCircleExclamation} fixedWidth />
      ) : props.type === "loading" ? (
        <div
          className="spinner-border"
          style={{ width: "16px", height: "16px", marginLeft: "1.68px" }}
        ></div>
      ) : undefined}
    </div>
    <div className="ps-1">{props.message}</div>
  </div>
);
