import { generateChangeLog } from "automation";

const version = Deno.args[0];
const changelog = await generateChangeLog({
  versionTo: version,
});

const text = `${changelog}

## Install

Add to your \`dprint.json\`:

\`\`\`json
{
  "plugins": [
    "https://plugins.dprint.dev/apcamargo/typstyle-${version}.wasm"
  ]
}
\`\`\`

Powered by \`typstyle-core\` 0.15.1.
`;

console.log(text);
