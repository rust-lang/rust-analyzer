// @ts-check

/** @type { import('prettier').Options } */
module.exports = {
    bracketSpacing: true,
    jsxBracketSameLine: true,
    printWidth: 100,
    semi: true,
    singleQuote: true,
    tabWidth: 4,
    trailingComma: "all",
    useTabs: false,
    overrides: [
        {
            files: "*.ts",
            options: {
                parser: "typescript",
            },
        },
    ],
};
