import express from "express";
import { getFilePath } from "./env";
import fs from "fs";
import path from "path";
import { type } from "os";

type Directory = string;

type File = {
  directory: string;
  filename: string;
} & ({ type: "file" } | { type: "directory"; files: File[] });

function crawlDirectory(directory: string, prefix: string = ""): File[] {
  let entries: fs.Dirent[];
  try {
    entries = fs.readdirSync(directory, { withFileTypes: true });
  } catch (e) {
    entries = [];
  }
  const cache: File[] = [];
  for (const entry of entries) {
    if (entry.isFile()) {
      cache.push({
        directory: prefix,
        filename: entry.name,
        type: "file",
      });
    } else if (entry.isDirectory()) {
      cache.push({
        directory: prefix,
        filename: entry.name,
        type: "directory",
        files: crawlDirectory(
          path.join(directory, entry.name),
          path.join(prefix, entry.name)
        ),
      });
    }
  }
  return cache;
}

function searchDirectory(
  files: File[],
  query: string,
  limit: number = 100,
  prefix: string = ""
): Omit<File, "files">[] {
  let remaining = limit;
  let result: Omit<File, "files">[] = [];
  for (const file of files) {
    if (remaining <= 0) {
      break;
    }

    const filePath = path.join(file.directory, file.filename);
    if (filePath.includes(query)) {
      result.push({
        directory: file.directory,
        filename: file.filename,
        type: file.type,
      });
      remaining--;
      continue;
    }

    if (file.type === "directory") {
      const subResult = searchDirectory(
        file.files,
        query,
        remaining,
        path.join(prefix, file.filename)
      );
      result.push(...subResult);
      remaining -= subResult.length;
    }
  }
  return result;
}

const getCache = () => crawlDirectory(getFilePath());
let cache: File[] = crawlDirectory(getFilePath());

// Refresh cache every 15 minutes.
setInterval(() => {
  cache = getCache();
}, 15 * 60 * 1000);

export const apiRouter = () => {
  const router = express.Router();

  router.use((_req, res, next) => {
    res.header("Access-Control-Allow-Origin", "*");
    res.header("Access-Control-Allow-Methods", "GET,PUT,POST,DELETE");
    res.header(
      "Access-Control-Allow-Headers",
      "Content-Type, Authorization, access_token"
    );
    next();
  });

  router.get("/search", (req, res) => {
    if (typeof req.query["query"] !== "string") {
      res.status(400).json({ message: "query is not string" });
      return;
    }

    res.status(200).json(searchDirectory(cache, req.query["query"]));
  });

  router.get("/list", (req, res) => {
    if (typeof req.query["path"] !== "string") {
      res.status(400).json({ message: "path is not string" });
      return;
    }

    const result = req.query["path"]
      .split("/")
      .reduce((currentDirectory, path) => {
        if (path === "") {
          return currentDirectory;
        }

        let matchingEntry = currentDirectory.find(
          (entry) => entry.filename === path
        );
        if (matchingEntry && matchingEntry.type === "directory") {
          return matchingEntry.files;
        } else {
          return [];
        }
      }, cache)
      .map((entry) => {
        if (entry.type === "directory") {
          let { files, ...remainingEntry } = entry;
          return remainingEntry;
        } else {
          return entry;
        }
      });

    res.status(200).json(result);
  });

  return router;
};
