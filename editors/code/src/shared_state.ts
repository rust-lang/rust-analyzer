import * as path from 'path';
import * as os from 'os';
import * as net from 'net';
import { Disposable, window, EventEmitter } from 'vscode';
import { log } from './util';

import { Event } from 'vscode-languageclient';

function getChannelName(id: string): string {
    const commonName = `rust-analyzer-${id}-sock`;

    if (process.platform === 'win32') {
        return `\\\\.\\pipe\\${commonName}`;
    }

    if (process.env['XDG_RUNTIME_DIR']) {
        // to make SystemD happy
        return path.join(process.env['XDG_RUNTIME_DIR'], commonName);
    }

    return path.join(os.tmpdir(), commonName);
}

export interface SharedStateService extends Disposable {
    onDidValueChanged: Event<{ id: string, value: any }>;
    onDidServerLost?: Event<void>;

    get(id: string): Thenable<any>;
    set(id: string, value: any): Thenable<void>;
}

interface SharedStateServer extends SharedStateService { }
interface SharedStateClient extends SharedStateService {
}

const debugOutput = window.createOutputChannel("Update");

function println(message?: any, ...optionalParams: any[]) {
    log.debug(message, optionalParams);

    debugOutput.appendLine(`${message} ${optionalParams}`);
}

class PipeServer implements SharedStateServer {
    values = new Map<string, any>();
    clients: net.Socket[];

    private eventEmitter = new EventEmitter<{ id: string; value: any; }>();
    onDidValueChanged: Event<{ id: string; value: any; }> = this.eventEmitter.event;

    constructor(readonly server: net.Server) {
        server.on('connection', this.handleNewClient.bind(this));
        this.clients = [];

        println('Server ready');
    }

    get(id: string): Thenable<any> {
        return Promise.resolve(this.values.get(id));
    }

    set(id: string, value: any): Thenable<void> {
        return this.set_internal(id, value);
    }

    dispose() {
        (async () => await new Promise((resolve) => this.server.close(resolve)))();
    }

    private handleNewClient(stream: net.Socket) {
        this.clients.push(stream);

        stream.on('close', (_hadError) => {
            this.clients.forEach((it, index) => {
                if (it === stream) this.clients.splice(index, 1);
            });

            println("Client is gone.");
        });

        stream.on('data', (data) => {
            this.handleClientMessage(data, stream);
        });
    }

    private set_internal(id: string, value: any, filter?: net.Socket): Thenable<void> {
        this.values.set(id, value);
        this.eventEmitter.fire({ id, value });
        this.clients.forEach(async (it) => {
            if (it !== filter) {
                await sendAndWait(it, { action: "notify", id, value });
            };
        });

        return Promise.resolve();
    }

    private handleClientMessage(data: Buffer, stream: net.Socket) {
        const obj = JSON.parse(data.toString());
        println(`  Query: '${data.toString()}'`);

        if (obj.action === 'set') {
            (async () => {
                await this.set_internal(obj.id, obj.value, stream)
                    .then(() => stream.write(JSON.stringify({ id: obj.id, ok: true })))
            })();
        } else {
            this.get(obj.id).then((v) => {
                const value = JSON.stringify({ id: obj.id, value: v });
                println(`  - S: '${value}'`);
                stream.write(value);
            })
        }
    }
}

function sendAndWait(socket: net.Socket, request: any): Thenable<any> {
    return new Promise((resolve, reject) => {
        let cleanup = () => {
            socket.removeListener('error', ups);
            socket.removeListener('data', resolveData);
        };
        let resolveData = (data: Buffer) => {
            println(`S->C: '${data.toString()}'`)
            cleanup();
            let result = JSON.parse(data.toString());
            resolve(result);
        };
        let ups = (err: Error) => {
            cleanup();
            socket.removeListener('error', ups);
            socket.removeListener('data', resolveData);
            reject(err);
        };
        socket.on('error', ups);
        socket.on('data', resolveData);
        socket.write(JSON.stringify(request));
    });
}

class PipeClient implements SharedStateClient {
    isLocal = false;

    private serverLost: EventEmitter<void> = new EventEmitter<void>();
    onDidServerLost: Event<void> = this.serverLost.event;

    private eventEmitter = new EventEmitter<{ id: string; value: any; }>();
    onDidValueChanged: Event<{ id: string; value: any; }> = this.eventEmitter.event;

    constructor(readonly socket: net.Socket, readonly clientId: string | number) {
        socket.on('close', () => {
            println(`Client ${this.clientId} disconnected`);
            this.serverLost.fire();
        })
    }

    get(id: string): Thenable<any> {
        return sendAndWait(this.socket, { action: 'get', id: id }).then((reply) => reply.value);
    }

    set(id: string, value: any): Thenable<void> {
        return sendAndWait(this.socket, { action: 'set', id: id, value: value }).then((v) => {
            println(`Server set reply: '${v}'`);
        });
    }

    dispose() {
        this.serverLost.dispose();
        this.socket.destroy();
    }
}

function createServer(id: string): Promise<SharedStateServer> {
    const pipeName = getChannelName(id);

    return new Promise((resolve, reject) => {
        var server = net.createServer();
        server.on('error', reject);
        server.listen(pipeName, () => {
            server.removeListener('error', reject);
            resolve(new PipeServer(server));
        });
    });
}

function connectTo(id: string, clientId: string | number): Promise<SharedStateClient> {
    const pipeName = getChannelName(id);

    return new Promise((resolve, reject) => {
        const client = net.createConnection(pipeName, () => {
            client.removeListener('error', reject);
            resolve(new PipeClient(client, clientId));
        });
        client.on('error', reject);
    });
}

export async function sharedStateService(id: string = 'update'): Promise<SharedStateService> {
    const service = await createServer(id).catch((_err) => {
        // A server already exists
        return connectTo(id, process.ppid);
    });

    return service;
}