const vscode = require("vscode");
const { execFile } = require("child_process");
const fs = require("fs");
const os = require("os");
const path = require("path");
const { promisify } = require("util");

const execFileAsync = promisify(execFile);
let lastRepository = "";
let warningShown = false;
let updateTimer;
let output;
let preferredContext = "editor";
let extensionContext;

const roasts = [
  "That command had one job and still filed for unemployment. 💀🥀",
  "Your terminal watched that command trip over its own syntax. Spectacular. 💀🪦",
  "That command failed so confidently it probably thinks the error is a feature. 💀🤡",
  "Even the error handler is embarrassed to be associated with that command. 💀😭",
  "A bold strategy: ask the computer to do something and immediately disappoint it. 💀🔥",
];

function showPixelSkull(roast, exitCode) {
  const panel = vscode.window.createWebviewPanel(
    "ht32Panel.terminalRoast",
    "Terminal Roast 💀",
    vscode.ViewColumn.Beside,
    { enableScripts: false },
  );
  const skull = panel.webview.asWebviewUri(
    vscode.Uri.joinPath(extensionContext.extensionUri, "media", "pixel-skull.svg"),
  );
  const safeRoast = roast.replace(/[&<>"']/g, (character) => ({
    "&": "&amp;", "<": "&lt;", ">": "&gt;", "\"": "&quot;", "'": "&#39;",
  })[character]);
  panel.webview.html = `<!doctype html>
    <html><head><meta charset="utf-8"><meta name="viewport" content="width=device-width, initial-scale=1">
    <style>
      body { color: var(--vscode-foreground); background: var(--vscode-editor-background); font-family: var(--vscode-font-family); display: grid; place-items: center; min-height: 90vh; text-align: center; }
      main { max-width: 440px; padding: 24px; } img { width: 192px; height: 192px; image-rendering: pixelated; } h1 { color: var(--vscode-errorForeground); } p { font-size: 1.15rem; line-height: 1.6; } small { opacity: .75; }
    </style></head><body><main><img src="${skull}" alt="Pixel art skull"><h1>Command eliminated 💀</h1><p>${safeRoast}</p><small>Exit code: ${exitCode} · The terminal remains undefeated.</small></main></body></html>`;
}

function roastFailedCommand(exitCode) {
  const roast = roasts[Math.floor(Math.random() * roasts.length)];
  const configuredCliPath = vscode.workspace
    .getConfiguration("ht32Panel")
    .get("cliPath", "ht32panelctl");
  const localCliPath = path.join(os.homedir(), ".local", "bin", "ht32panelctl");
  const cliPath = configuredCliPath === "ht32panelctl" && fs.existsSync(localCliPath)
    ? localCliPath
    : configuredCliPath;
  vscode.window.showErrorMessage(`Terminal command failed: ${roast}`, "Show pixel skull").then((choice) => {
    if (choice === "Show pixel skull") showPixelSkull(roast, exitCode);
  });
  output.appendLine(`Failed terminal command (exit ${exitCode}): ${roast}`);
  execFileAsync(cliPath, ["roast", roast, "--seconds", "8"], { timeout: 5000 })
    .then(() => output.appendLine("Sent roast and pixel skull to the HT32 panel for 8 seconds."))
    .catch((error) => {
      output.appendLine(`Could not show roast on HT32 panel via ${cliPath}: ${error.message}`);
      if (error.stderr) output.appendLine(error.stderr.trim());
    });
}

function activeRepositoryPath() {
  if (preferredContext === "terminal") {
    const terminal = vscode.window.activeTerminal;
    const terminalCwd = terminal?.shellIntegration?.cwd;
    if (terminalCwd?.scheme === "file") {
      output.appendLine(`Using active terminal directory: ${terminalCwd.fsPath}`);
      return terminalCwd.fsPath;
    }
  }

  const editor = vscode.window.activeTextEditor;
  if (editor) {
    const folder = vscode.workspace.getWorkspaceFolder(editor.document.uri);
    if (folder) {
      output.appendLine(`Terminal directory unavailable; using active file workspace: ${folder.uri.fsPath}`);
      return folder.uri.fsPath;
    }
  }
  const folder = vscode.workspace.workspaceFolders?.[0];
  if (folder) output.appendLine(`Terminal and active file unavailable; using workspace: ${folder.uri.fsPath}`);
  return folder?.uri.fsPath;
}

function scheduleUpdate(forceRefresh = false) {
  clearTimeout(updateTimer);
  updateTimer = setTimeout(() => updateRepository(forceRefresh), 500);
}

async function updateRepository(forceRefresh = false) {
  const repositoryPath = activeRepositoryPath();
  if (!repositoryPath) {
    output.appendLine("No workspace folder is open.");
    return;
  }

  let repository;
  try {
    const result = await execFileAsync("git", ["-C", repositoryPath, "rev-parse", "--show-toplevel"]);
    repository = result.stdout.trim();
  } catch (error) {
    output.appendLine(`Could not detect a Git repository: ${error.message}`);
    if (!warningShown) {
      vscode.window.showWarningMessage(
        `HT32 Panel could not detect Git in ${repositoryPath}. Check that Git is installed and this folder is a repository.`,
      );
      warningShown = true;
    }
    return;
  }

  if (!repository || (repository === lastRepository && !forceRefresh)) {
    output.appendLine(repository ? `Already tracking ${repository}` : "Git returned an empty repository path.");
    return;
  }

  const configuredCliPath = vscode.workspace
    .getConfiguration("ht32Panel")
    .get("cliPath", "ht32panelctl");
  const localCliPath = path.join(os.homedir(), ".local", "bin", "ht32panelctl");
  const cliPath = configuredCliPath === "ht32panelctl" && fs.existsSync(localCliPath)
    ? localCliPath
    : configuredCliPath;
  try {
    await execFileAsync(cliPath, ["git", "repo", repository], { timeout: 10000 });
    lastRepository = repository;
    warningShown = false;
    output.appendLine(`Panel now tracks ${repository} (CLI: ${cliPath})`);
  } catch (error) {
    output.appendLine(`CLI failed (${cliPath}): ${error.message}`);
    if (error.stderr) output.appendLine(error.stderr.trim());
    if (!warningShown) {
      vscode.window.showWarningMessage(
        `HT32 Panel could not switch to ${repository}. Is ht32paneld running, and is ht32panelctl available?`,
      );
      warningShown = true;
    }
    console.error("HT32 Panel Git workspace update failed:", error);
  }
}

function activate(context) {
  extensionContext = context;
  output = vscode.window.createOutputChannel("HT32 Panel Git Workspace");
  context.subscriptions.push(output);
  context.subscriptions.push(vscode.commands.registerCommand("ht32Panel.refreshGitRepository", scheduleUpdate));
  context.subscriptions.push(
    vscode.window.onDidChangeActiveTextEditor(() => {
      preferredContext = "editor";
      scheduleUpdate();
    }),
    vscode.window.onDidChangeActiveTerminal((terminal) => {
      preferredContext = terminal ? "terminal" : "editor";
      scheduleUpdate();
    }),
    vscode.window.onDidChangeTerminalShellIntegration(scheduleUpdate),
    vscode.workspace.onDidChangeWorkspaceFolders(scheduleUpdate),
    vscode.workspace.onDidChangeConfiguration((event) => {
      if (event.affectsConfiguration("ht32Panel.cliPath")) scheduleUpdate();
    }),
  );
  // This event fires after commands finish. Checking then catches `cd` even
  // when shell integration does not emit a separate cwd property change.
  if (typeof vscode.window.onDidEndTerminalShellExecution === "function") {
    context.subscriptions.push(
      vscode.window.onDidEndTerminalShellExecution((event) => {
        const command = event.execution.commandLine?.value || "(command not reported)";
        output.appendLine(`Terminal command finished with exit code ${event.exitCode}: ${command}`);
        if (event.exitCode !== undefined && event.exitCode !== 0) {
          roastFailedCommand(event.exitCode);
        }
        if (event.terminal === vscode.window.activeTerminal && preferredContext === "terminal") {
          // Refresh even in the same repository: checkout/switch/branch
          // commands change the branch while leaving the repository path intact.
          scheduleUpdate(true);
        }
      }),
    );
    output.appendLine("Terminal command and failure detection is active.");
  } else {
    output.appendLine("Terminal failure detection unavailable; update VS Code to version 1.93 or newer.");
  }
  output.appendLine("Extension activated.");
  scheduleUpdate();
}

function deactivate() {
  clearTimeout(updateTimer);
}

module.exports = { activate, deactivate };
