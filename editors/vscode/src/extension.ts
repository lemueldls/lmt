import { workspace, ExtensionContext, window, commands } from "vscode";
import {
  LanguageClient,
  LanguageClientOptions,
  ServerOptions,
  TransportKind,
  Executable,
} from "vscode-languageclient/node";

let client: LanguageClient;

export function activate(context: ExtensionContext) {
  let disposable = commands.registerCommand("lmt-vscode.helloWorld", () => {
    window.showInformationMessage("Hello World from LMT VSCode Extension!");
  });
  context.subscriptions.push(disposable);

  const command = process.env.LMT_LSP_PATH || "lmt-lsp";

  const run: Executable = {
    command,
    options: {
      env: {
        ...process.env,
        RUST_LOG: "debug",
      },
    },
    transport: TransportKind.stdio,
  };

  const traceOutputChannel = window.createOutputChannel("LMT Language Server Trace");

  const serverOptions: ServerOptions = {
    run,
    debug: run,
  };

  const clientOptions: LanguageClientOptions = {
    documentSelector: [{ scheme: "file", language: "lmt" }],
    synchronize: {
      fileEvents: workspace.createFileSystemWatcher("**/*.lmt"),
    },
    traceOutputChannel,
  };

  client = new LanguageClient(
    "lmtLanguageServer",
    "LMT Language Server",
    serverOptions,
    clientOptions,
  );

  client.start();
}

export function deactivate(): Thenable<void> | undefined {
  if (!client) {
    return undefined;
  }
  return client.stop();
}
