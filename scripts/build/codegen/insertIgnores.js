"use strict";
var __importDefault = (this && this.__importDefault) || function (mod) {
    return (mod && mod.__esModule) ? mod : { "default": mod };
};
Object.defineProperty(exports, "__esModule", { value: true });
const promises_1 = require("fs/promises");
const path_1 = require("path");
const prepend_file_1 = __importDefault(require("prepend-file"));
void (async function () {
    const generatedTypesDir = (0, path_1.resolve)((0, path_1.join)(__dirname, '../../types/generated'));
    const typeFiles = await getFiles(generatedTypesDir);
    for (const file of typeFiles) {
        await (0, prepend_file_1.default)(file, '// @ts-nocheck\n');
    }
})();
async function getFiles(dir) {
    const dirents = await (0, promises_1.readdir)(dir, { withFileTypes: true });
    const files = await Promise.all(dirents.map((dirent) => {
        const res = (0, path_1.resolve)(dir, dirent.name);
        return dirent.isDirectory() ? getFiles(res) : res;
    }));
    return Array.prototype.concat(...files);
}
//# sourceMappingURL=insertIgnores.js.map