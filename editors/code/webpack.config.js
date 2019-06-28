//@ts-check

'use strict';

const path = require('path');

/**
 * @type {import('webpack').Configuration}
 */

const config = {
    devtool: "source-map",
    entry: "./src/extension.ts",
    externals: {
        vscode: "commonjs vscode"
    },
    module: {
		rules: [{
			exclude: /node_modules/,
			test: /\.ts$/,
			use: [{
				loader: "ts-loader",
			}],
		}],
    },
    output: {
        path: path.resolve(__dirname, 'dist'),
        filename: 'extension.js',
        libraryTarget: 'commonjs2',
        devtoolModuleFilenameTemplate: '../[resource-path]'
    },
    resolve: {
        extensions: ['.ts', '.js']
    },
    target: "node",
}

module.exports = config;
