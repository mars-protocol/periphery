"use strict";
var __importDefault = (this && this.__importDefault) || function (mod) {
    return (mod && mod.__esModule) ? mod : { "default": mod };
};
Object.defineProperty(exports, "__esModule", { value: true });
const ts_codegen_1 = __importDefault(require("@cosmwasm/ts-codegen"));
const path_1 = require("path");
const chalk_js_1 = require("../utils/chalk.js");
const promises_1 = require("fs/promises");
void (async function () {
    const schemasDir = (0, path_1.resolve)((0, path_1.join)(__dirname, '../../../schemas'));
    const schemas = await (0, promises_1.readdir)(schemasDir);
    for (const schema of schemas) {
        try {
            await (0, ts_codegen_1.default)({
                contracts: [`${schemasDir}/${schema}`],
                outPath: `./types/generated/${schema}`,
                options: {
                    types: {
                        enabled: true,
                    },
                    client: {
                        enabled: true,
                    },
                    reactQuery: {
                        enabled: true,
                        optionalClient: true,
                        version: 'v4',
                        mutations: true,
                        queryKeys: true,
                    },
                    messageComposer: {
                        enabled: false,
                    },
                },
            });
            (0, chalk_js_1.printGreen)(`Success ✨ ${schema} types generated`);
        }
        catch (e) {
            (0, chalk_js_1.printRed)(`Error with ${schema}: ${e}`);
        }
    }
})();
//# sourceMappingURL=index.js.map