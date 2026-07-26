import { spawnSync } from "node:child_process";
import { readFileSync } from "node:fs";

export function runCLI() {
    const args = process.argv.slice(2);

    if (args[0] === "lex" && args[1]) {
        const file = args[1];
        const src = readFileSync(file, "utf8");

        const result = spawnSync("cargo", ["run", "-p", "helion-lexer-demo"], {
            input: src,
            stdio: ["pipe", "inherit", "inherit"],
        });

        process.exit(result.status ?? 0);
        return;
    }

    console.log("Helion CLI");
    console.log("Usage:");
    console.log("  helion lex <file>");
}