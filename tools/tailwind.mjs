import { execFileSync, spawn } from "node:child_process";
import { dirname, join } from "node:path";

// Cargo resolves the pinned rust-ui checkout; its classes live outside this project.
const metadata = JSON.parse(execFileSync("cargo", ["metadata", "--locked", "--format-version", "1"], {
  encoding: "utf8",
  maxBuffer: 32 * 1024 * 1024,
}));
const registry = metadata.packages.find((pkg) => pkg.name === "registry" && pkg.source?.includes("github.com/rust-ui/ui"));
if (!registry) throw new Error("rust-ui registry is missing from Cargo metadata");
const source = join(dirname(registry.manifest_path), "src/ui").replaceAll("\\", "/");
const cli = spawn("tailwindcss", ["-i", "-", "-o", "./assets/tailwind.css", ...process.argv.slice(2)], {
  stdio: ["pipe", "inherit", "inherit"],
  shell: process.platform === "win32",
});
cli.stdin.end(`@import "./input.css";\n@source "${source}";\n`);
cli.on("error", (error) => { console.error(error); process.exitCode = 1; });
cli.on("exit", (code) => { process.exitCode = code ?? 1; });
