import { $ } from "automation";

// 1. Query crates.io for newest typstyle-core
const res = await fetch("https://crates.io/api/v1/crates/typstyle-core");
const data = await res.json();
const latestVersion = data.crate.max_version;

console.log(`Latest typstyle-core on crates.io: ${latestVersion}`);

// 2. Check current Cargo.lock
const cargoLock = await Deno.readTextFile("Cargo.lock");
if (
  cargoLock.includes(`name = "typstyle-core"\nversion = "${latestVersion}"`)
) {
  console.log("Already on latest version.");
  Deno.exit(0);
}

// 3. Cargo update
if (latestVersion.startsWith("0.15.")) {
  await $`cargo update -p typstyle-core`;
} else {
  // Major/minor bump requires updating Cargo.toml requirement
  let cargoToml = await Deno.readTextFile("Cargo.toml");
  cargoToml = cargoToml.replace(
    /typstyle-core = "[^"]+"/,
    `typstyle-core = "${latestVersion}"`,
  );
  await Deno.writeTextFile("Cargo.toml", cargoToml);
  await $`cargo update -p typstyle-core --precise ${latestVersion}`;
}

// 4. Verification pipeline
await $`cargo test --locked`;
await $`cargo clippy --locked --all-targets --all-features -- -D warnings`;
await $`cargo build --locked --target wasm32-unknown-unknown --features wasm --release`;

console.log("Update succeeded!");
