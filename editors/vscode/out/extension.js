"use strict";
Object.defineProperty(exports, "__esModule", { value: true });
exports.deactivate = exports.activate = void 0;
const vscode_1 = require("vscode");
const node_1 = require("vscode-languageclient/node");
let client;
function activate(context) {
    // Use cpc lsp from PATH
    const serverCommand = 'cpc';
    const serverArgs = ['lsp'];
    const serverOptions = {
        run: { command: serverCommand, args: serverArgs, transport: node_1.TransportKind.stdio },
        debug: { command: serverCommand, args: serverArgs, transport: node_1.TransportKind.stdio }
    };
    const clientOptions = {
        documentSelector: [{ scheme: 'file', language: 'cp' }],
        synchronize: {
            fileEvents: vscode_1.workspace.createFileSystemWatcher('**/.clientrc')
        }
    };
    client = new node_1.LanguageClient('cpLanguageServer', 'CP-BEAM Language Server', serverOptions, clientOptions);
    client.start();
}
exports.activate = activate;
function deactivate() {
    if (!client) {
        return undefined;
    }
    return client.stop();
}
exports.deactivate = deactivate;
//# sourceMappingURL=extension.js.map