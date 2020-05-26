import * as path from 'path';
import * as os from 'os';
import * as net from 'net';
import { Disposable, EventEmitter } from 'vscode';
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

export abstract class SharedStateService implements Disposable {
    protected valueChangedEmitter = new EventEmitter<{ id: string; value: any; }>();
    onDidValueChanged: Event<{ id: string; value: any; }> = this.valueChangedEmitter.event;

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

    private eventEmitter = new EventEmitter<{ id: string; value: any; }>();
    onDidValueChanged: Event<{ id: string; value: any; }> = this.eventEmitter.event;
    onDidServerLost: undefined;

    constructor(readonly server: net.Server) {
        super();

        server.on('connection', this.handleNewClient.bind(this));
        this.clients = [];

        log.debug('[U] Server ready');
    }

    get(id: string): Thenable<any> {
        return Promise.resolve(this.values.get(id));
    }

    set(id: string, value: any): Thenable<void> {
        return this.set_internal(id, value);
    }

    dispose() {
        (async () => await new Promise((resolve) => {
            this.clients.forEach(it => it.destroy());
            this.server.close(resolve);
        }))();

        log.debug('[U] Server disposed.');
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
        log.debug(`[U] C->S: '${data.toString()}'`);

        if (obj.action === 'set') {
            (async () => {
                await this.set_internal(obj.id, obj.value, stream)
                    .then(() => stream.write(JSON.stringify({ id: obj.id, ok: true })))
            })();
        } else {
            this.get(obj.id).then((v) => {
                const value = JSON.stringify({ id: obj.id, value: v });
                log.debug(`[U]    S->C: '${value}'`);
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

class PipeClient extends SharedStateService {
    private serverLost: EventEmitter<void> = new EventEmitter<void>();
    onDidServerLost: Event<void> = this.serverLost.event;

    constructor(readonly socket: net.Socket, readonly clientId: string | number) {
        super();

        socket.on('close', () => this.serverLost.fire());
        socket.on('data', (data) => {
            const obj = JSON.parse(data.toString());
            if (obj.action === 'notify') {
                this.valueChangedEmitter.fire({ id: obj.id, value: obj.value });
            }
        });
    }

    get(id: string): Thenable<any> {
        return sendAndWait(this.socket, { action: 'get', id: id }).then((reply) => reply.value);
    }

    set(id: string, value: any): Thenable<void> {
        return sendAndWait(this.socket, { action: 'set', id: id, value: value });
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
        server.listen(pipeName, () => {
            server.removeListener('error', reject);
            resolve(new PipeServer(server));
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

export async function sharedStateService(id: string = 'update'): Promise<SharedStateService> {
    const service = await createServer(id).catch((_err) => {
        // A server already exists
        return connectTo(id, process.ppid);
    });

    return service;
}