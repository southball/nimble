import express from "express";
import * as path from "path";
import { getFilePath } from "./env";
import { apiRouter } from "./api";

const app = express();

const appPath = path.join(__dirname, "../app/build");
const filesPath = getFilePath();
const port = parseInt(process.env["PORT"] ?? "3000");

app.use("/files", express.static(filesPath, { dotfiles: "allow" }));

app.use("/api", apiRouter());

app.use("/app", express.static(appPath));

app.get("/", (_req, res) => {
  res.redirect("/app");
});

console.log(`Listening on port ${port}`);
app.listen(port);
