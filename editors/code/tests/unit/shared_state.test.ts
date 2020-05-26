import * as assert from 'assert';
import { sharedStateService } from '../../src/shared_state';

import { log } from '../../src/util';

suite('Shared state', () => {
    log.setEnabled(true);
    const testPipeId = "test_42c55ec3-0d97-4cb7-9a2d-e4e1abbd42e4";

    test('Single server works', async () => {
        const server = await sharedStateService(testPipeId);
        assert.deepEqual(undefined, server.onDidServerLost);

        server.set('progress', 55);
        assert.deepEqual(await server.get('progress'), 55);
        server.dispose();
    });

    test('Server is disposable', async () => {
        const server = await sharedStateService(testPipeId);
        assert.deepEqual(undefined, server.onDidServerLost);
        server.dispose();

        const server2 = await sharedStateService(testPipeId);
        assert.deepEqual(undefined, server2.onDidServerLost);
        server2.dispose();
    });

    test('A client can connect to the server', async () => {
        const server = await sharedStateService(testPipeId);
        assert.deepEqual(undefined, server.onDidServerLost);

        const client = await sharedStateService(testPipeId);
        assert.notDeepEqual(undefined, client.onDidServerLost);

        server.dispose();

    });

    test('Several clients can connect to the server', async () => {
        const server = await sharedStateService(testPipeId);
        assert.deepEqual(undefined, server.onDidServerLost);

        const client = await sharedStateService(testPipeId, 11);
        assert.notDeepEqual(undefined, client.onDidServerLost);

        const client2 = await sharedStateService(testPipeId, 22);
        assert.notDeepEqual(undefined, client2.onDidServerLost);

        server.dispose();
        client.dispose();
        client2.dispose();
    });

    test('Server and clients share variables', async () => {
        const server = await sharedStateService(testPipeId);
        assert.deepEqual(undefined, server.onDidServerLost);

        const client = await sharedStateService(testPipeId, 1);
        assert.notDeepEqual(undefined, client.onDidServerLost);

        const client2 = await sharedStateService(testPipeId, 2);
        assert.notDeepEqual(undefined, client2.onDidServerLost);

        let value: { name: string; value: any } | undefined;
        let value2: { name: string; value: any } | undefined;

        client.onDidValueChanged((v) => { log.debug(`Client 1 value changed: ${v.name} -> ${v.value}`); value = v; });
        client2.onDidValueChanged((v) => { log.debug(`Client 2 value changed: ${v.name} -> ${v.value}`); value2 = v; });

        await server.set('active', 33);
        assert.deepEqual(await server.get('active'), 33);
        assert.deepEqual(await client.get('active'), 33);
        assert.deepEqual(await client2.get('active'), 33);

        await server.set('active', "!");
        assert.deepEqual(await server.get('active'), "!");
        assert.deepEqual(await client.get('active'), "!");
        assert.deepEqual(await client2.get('active'), "!");

        await server.set('obj', { ok: true });
        assert.deepEqual(await server.get('obj'), { ok: true });
        assert.deepEqual(await client.get('obj'), { ok: true });
        assert.deepEqual(await client2.get('obj'), { ok: true });

        assert.deepEqual({ name: 'obj', value: { ok: true } }, value);
        assert.deepEqual(value, value2);

        server.dispose();
        client.dispose();
        client2.dispose();
    });
});
