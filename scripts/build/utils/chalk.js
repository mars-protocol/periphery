"use strict";
var __importDefault = (this && this.__importDefault) || function (mod) {
    return (mod && mod.__esModule) ? mod : { "default": mod };
};
Object.defineProperty(exports, "__esModule", { value: true });
exports.printYellow = exports.printGreen = exports.printBlue = exports.printRed = void 0;
const chalk_1 = __importDefault(require("chalk"));
const printRed = (text) => {
    console.log(chalk_1.default.red(text));
};
exports.printRed = printRed;
const printBlue = (text) => {
    console.log(chalk_1.default.blue(text));
};
exports.printBlue = printBlue;
const printGreen = (text) => {
    console.log(chalk_1.default.green(text));
};
exports.printGreen = printGreen;
const printYellow = (text) => {
    console.log(chalk_1.default.yellow(text));
};
exports.printYellow = printYellow;
//# sourceMappingURL=chalk.js.map