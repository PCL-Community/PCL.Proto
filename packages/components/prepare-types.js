import { copyFile, readFile, writeFile } from 'node:fs/promises'

const typesDir = 'packages/components/dist/types'
for await (const path of new Bun.Glob('**/*.d.ts').scan({ cwd: typesDir, absolute: true })) {
  const content = await readFile(path, 'utf8')
  const updated = content.replace(
    /((?:from\s*|import\s*\()\s*['"])(\.\.?\/[^'"]+)(['"])/g,
    (_, prefix, specifier, quote) =>
      `${prefix}${specifier.endsWith('.js') ? specifier : `${specifier}.js`}${quote}`,
  )
  if (updated !== content) await writeFile(path, updated)
}
await copyFile('LICENSE', 'packages/components/LICENSE')
