import * as assert from 'assert';
import { sharedStateService } from '../../src/shared_state';

import { log } from '../../src/util';

suite('Shared state', () => {
    log.setEnabled(true);
    const testPipeId = "test_42c55ec3-0d97-4cb7-9a2d-e4e1abbd42e4";

    test('Single server works', async () => {
        const server = await sharedStateService(testPipeId);
        assert.equal(undefined, server.onDidServerLost);

        server.set('progress', 55);
        assert.equal(await server.get('progress'), 55);
        server.dispose();
    });

    test('Server is disposable', async () => {
        const server = await sharedStateService(testPipeId);
        assert.equal(undefined, server.onDidServerLost);
        server.dispose();

        const server2 = await sharedStateService(testPipeId);
        assert.equal(undefined, server2.onDidServerLost);
        server2.dispose();
    });

    test('A client can connect to the server', async () => {
        const server = await sharedStateService(testPipeId);
        assert.equal(undefined, server.onDidServerLost);

        const client = await sharedStateService(testPipeId);
        assert.notEqual(undefined, client.onDidServerLost);

        server.dispose();

    });

    test('Several clients can connect to the server', async () => {
        const server = await sharedStateService(testPipeId);
        assert.equal(undefined, server.onDidServerLost);

        const client = await sharedStateService(testPipeId);
        assert.notEqual(undefined, client.onDidServerLost);

        const client2 = await sharedStateService(testPipeId);
        assert.notEqual(undefined, client2.onDidServerLost);

        server.dispose();
        client.dispose();
        client2.dispose();
    });

    test('Server and clients share variables', async () => {
        const server = await sharedStateService(testPipeId);
        assert.equal(undefined, server.onDidServerLost);

        const client = await sharedStateService(testPipeId);
        assert.notEqual(undefined, client.onDidServerLost);

        const client2 = await sharedStateService(testPipeId);
        assert.notEqual(undefined, client2.onDidServerLost);

        await server.set('active', 33);
        assert.equal(await server.get('active'), 33);
        assert.equal(await client.get('active'), 33);
        assert.equal(await client2.get('active'), 33);

        await client.set('active', "!");
        assert.equal(await server.get('active'), "!");
        assert.equal(await client.get('active'), "!");
        assert.equal(await client2.get('active'), "!");

        server.dispose();
        client.dispose();
        client2.dispose();
    });

});
