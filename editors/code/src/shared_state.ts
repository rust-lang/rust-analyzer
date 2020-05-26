import * as path from 'path';
import * as os from 'os';
import * as fs from 'fs';
import * as net from 'net';

import { Disposable, EventEmitter } from 'vscode';
import { Event } from 'vscode-languageclient';
import { log } from './util';

function getChannelName(id: string): string {
    const commonName = `rust-analyzer-${id}.sock`;

    if (process.platform === 'win32') {
        return `\\\\.\\pipe\\${commonName}`;
    }

    if (process.platform === 'darwin') {
        return `/tmp/${commonName}`;
    }

    if (process.env['XDG_RUNTIME_DIR']) {
        // to make SystemD happy
        return path.join(process.env['XDG_RUNTIME_DIR'], commonName);
    }

    return path.join(os.tmpdir(), commonName);
}

export abstract class SharedStateService implements Disposable {
    protected valueChangedEmitter = new EventEmitter<{ name: string; value: any }>();
    onDidValueChanged: Event<{ name: string; value: any }> = this.valueChangedEmitter.event;

    abstract onDidServerLost?: Event<void>;
    abstract get(id: string): Thenable<any>;
    abstract set(id: string, value: any): Thenable<void>;
    abstract dispose(): any;

    get isLocal(): boolean {
        return this.onDidServerLost !== undefined;
    }
}

class PipeServer extends SharedStateService {
    values = new Map<string, any>();
    clients: net.Socket[];

    onDidServerLost: undefined;

    constructor(readonly server: net.Server, readonly ipcPath: string) {
        super();

        server.on('connection', this.handleNewClient.bind(this));
        this.clients = [];
    }

    get(id: string): Thenable<any> {
        return Promise.resolve(this.values.get(id));
    }

    set(id: string, value: any): Thenable<void> {
        return this.setInternal(id, value);
    }

    dispose() {
        (async () => await new Promise((resolve) => {
            this.clients.forEach(it => it.destroy());
            this.server.close(resolve);
        }))();
        try {
            fs.unlinkSync(this.ipcPath);
        } catch {
            // do nothing
        }
    }

    private handleNewClient(stream: net.Socket) {
        this.clients.push(stream);

        stream.on('close', (_hadError) => {
            this.clients.forEach((it, index) => {
                if (it === stream) this.clients.splice(index, 1);
            });
        });

        stream.on('data', (data) => {
            this.handleClientMessage(data, stream);
        });
    }

    private setInternal(name: string, value: any, filter?: net.Socket): Thenable<void> {
        this.values.set(name, value);
        this.valueChangedEmitter.fire({ name, value });
        this.clients.forEach(async (it) => {
            if (it !== filter) {
                send(it, { name, value }, 'notify');
            };
        });

        log.debug(`[U] Server set: ${name} -> ${value}`);

        return Promise.resolve();
    }

    private handleClientMessage(data: Buffer, stream: net.Socket) {
        const s = data.toString().trimEnd().split('\n');

        log.debug("C->S: " + s);

        s.forEach(it => {
            const request = JSON.parse(it);

            if (request.action === 'set') {
                (async () => {
                    await this.setInternal(request.name, request.value, stream).then(() => {
                        log.debug("S->C: " + JSON.stringify({ id: request.id, ok: true }));
                        send(stream, { ok: true }, request.id);
                    });
                })();
            } else if (request.action === 'get') {
                this.get(request.name).then((v) => {
                    log.debug("S->C: " + JSON.stringify({ id: request.id, value: v, action: "reply" }));
                    send(stream, { value: v }, request.id);
                });
            }
        });
    }
}

function send(socket: net.Socket, request: any, requestId: number | string) {
    request.id = requestId;
    socket.write(JSON.stringify(request) + '\n');
}

function sendAndWait(socket: net.Socket, request: any): Thenable<any> {
    let nextRequestId = 0;

    return new Promise((resolve, reject) => {
        const requestId = nextRequestId;
        nextRequestId += 1;

        const cleanup = () => {
            socket.removeListener('error', ups);
            socket.removeListener('data', resolveData);
        };
        const resolveData = (data: Buffer) => {
            const messages = data.toString().trimEnd().split('\n');
            messages.forEach(it => {
                const obj = JSON.parse(it);
                if (obj.id === requestId) {
                    cleanup();
                    resolve(obj);
                }
            });
        };
        const ups = (err: Error) => {
            cleanup();
            reject(err);
        };
        socket.on('error', ups);
        socket.on('data', resolveData);
        send(socket, request, requestId);
    });
}

class PipeClient extends SharedStateService {
    private serverLost: EventEmitter<void> = new EventEmitter<void>();
    onDidServerLost: Event<void> = this.serverLost.event;

    constructor(readonly socket: net.Socket, readonly clientId: string | number) {
        super();

        socket.on('close', () => this.serverLost.fire());
        socket.on('data', this.handleServerNotify.bind(this));
    }

    private handleServerNotify(data: Buffer) {
        const s = data.toString().trimEnd().split('\n');
        s.forEach(it => {
            const request = JSON.parse(it);
            if (request.id === 'notify') {
                this.valueChangedEmitter.fire({ name: request.name, value: request.value });
                send(this.socket, {}, request.id);
            }
        });
    }

    get(name: string): Thenable<any> {
        return sendAndWait(this.socket, { clientId: this.clientId, action: 'get', name }).then((obj) => obj.value);
    }

    set(name: string, value: any): Thenable<void> {
        return sendAndWait(this.socket, { clientId: this.clientId, action: 'set', name, value });
    }

    dispose() {
        this.serverLost.dispose();
        this.socket.destroy();
    }
}

function createServer(id: string): Promise<SharedStateService> {
    const pipeName = getChannelName(id);

    return new Promise((resolve, reject) => {
        var server = net.createServer();
        server.on('error', reject);

        try {
            fs.unlinkSync(pipeName);
        } catch {
            // do nothing
        }

        server.listen({ path: pipeName, exclusive: true }, () => {
            server.removeListener('error', reject);
            resolve(new PipeServer(server, pipeName));
        });
    });
}

function connectTo(id: string, clientId: string | number): Promise<SharedStateService> {
    const pipeName = getChannelName(id);

    return new Promise((resolve, reject) => {
        const client = net.createConnection(pipeName, () => {
            client.removeListener('error', reject);
            resolve(new PipeClient(client, clientId));
        });
        client.on('error', reject);
    });
}

export async function sharedStateService(id: string = 'update', clientId?: number): Promise<SharedStateService> {
    const service = await connectTo(id, clientId ?? process.ppid).catch((_err) => {
        return createServer(id);
    });

    return service;
}