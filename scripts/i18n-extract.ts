import * as babelParser from "@babel/parser";
import _traverse from "@babel/traverse";
import fs from "fs";
import path from "path";

// Deal with esm/cjs interop
const traverse = _traverse.default;

const SRC_DIR = path.join(process.cwd(), "src");
const LOCALES_DIR = path.join(SRC_DIR, "locales");
const FILE_EXTENSIONS = [".ts", ".tsx"];
const IGNORED_DIRS = [LOCALES_DIR];

const TARGET_ATTRIBUTES = ["placeholder", "title", "alt", "label", "aria-label", "tooltip"];

async function findFiles(dir: string): Promise<string[]> {
  let files: string[] = [];
  const entries = await fs.promises.readdir(dir, { withFileTypes: true });

  for (const entry of entries) {
    const fullPath = path.join(dir, entry.name);
    if (IGNORED_DIRS.some(ignored => fullPath.startsWith(ignored))) {
      continue;
    }
    if (entry.isDirectory()) {
      files = files.concat(await findFiles(fullPath));
    } else if (FILE_EXTENSIONS.includes(path.extname(fullPath))) {
      files.push(fullPath);
    }
  }

  return files;
}

async function extractStringsFromFile(filePath: string): Promise<string[]> {
  const content = await fs.promises.readFile(filePath, "utf-8");
  const extractedStrings: string[] = [];

  try {
    const ast = babelParser.parse(content, {
      sourceType: "module",
      plugins: ["typescript", "jsx"],
      errorRecovery: true,
    });

    traverse(ast, {
      JSXText(path) {
        const value = path.node.value.trim().replace(/\s+/g, " ");
        if (value) {
          extractedStrings.push(value);
        }
      },
      JSXAttribute(path) {
        const attributeName = path.node.name.name;
        if (typeof attributeName === "string" && TARGET_ATTRIBUTES.includes(attributeName)) {
          const valueNode = path.node.value;
          if (valueNode?.type === "StringLiteral") {
            const value = valueNode.value.trim();
            if (value) {
              extractedStrings.push(value);
            }
          }
        }
      },
    });
  } catch (error) {
    console.error(`Error parsing file: ${filePath}`, error);
  }

  return extractedStrings;
}

function generateKey(filePath: string, value: string, keys: Set<string>): string {
  const relativePath = path.relative(SRC_DIR, filePath);
  const pathParts = relativePath.split(path.sep).map(part => part.replace(/\.(ts|tsx)$/, ""));
  const valuePart = value
    .toLowerCase()
    .replace(/[^a-z0-9\s-]/g, "")
    .trim()
    .replace(/\s+/g, "-")
    .substring(0, 20);

  let key = [...pathParts, valuePart].join(".");
  let counter = 1;
  while (keys.has(key)) {
    key = `${[...pathParts, valuePart].join(".")}.${counter}`;
    counter++;
  }
  return key;
}

function isTranslatable(str: string): boolean {
  // Basic filter for strings that are probably not for translation
  if (!str || /^[0-9\s.,!?'"()-_:/%x+]+$/.test(str)) {
    return false;
  }
  // Filter out single character strings unless they are chinese characters
  if (str.length === 1 && !/[\u4e00-\u9fa5]/.test(str)) {
    return false;
  }
  return true;
}

async function main() {
  console.log("Starting i18n extraction...");
  const files = await findFiles(SRC_DIR);
  const allStrings = new Map<string, string>(); // value -> filePath

  for (const file of files) {
    const strings = await extractStringsFromFile(file);
    strings.forEach(s => {
      if (isTranslatable(s) && !allStrings.has(s)) {
        allStrings.set(s, file);
      }
    });
  }

  const localeFiles = [
    path.join(LOCALES_DIR, "en", "translation.json"),
    path.join(LOCALES_DIR, "zh-CN", "translation.json"),
    path.join(LOCALES_DIR, "zh-TW", "translation.json"),
  ];

  const translations: Record<string, any> = {};
  const existingValues = new Set<string>();
  const existingKeys = new Set<string>();

  for (const file of localeFiles) {
    const content = await fs.promises.readFile(file, "utf-8");
    translations[file] = JSON.parse(content);
    Object.entries(translations[file]).forEach(([key, value]) => {
      existingKeys.add(key);
      existingValues.add(value as string);
    });
  }

  let newStringsCount = 0;
  for (const [value, filePath] of allStrings.entries()) {
    if (!existingValues.has(value)) {
      const key = generateKey(filePath, value, existingKeys);
      existingKeys.add(key);
      newStringsCount++;

      for (const file of localeFiles) {
        translations[file][key] = value;
      }
    }
  }

  for (const file of localeFiles) {
    await fs.promises.writeFile(file, JSON.stringify(translations[file], null, 2) + "\n");
  }

  console.log(`Extraction complete. Found ${allStrings.size} unique translatable strings.`);
  console.log(`Added ${newStringsCount} new strings to the translation files.`);
  console.log("Please review the changes in the `src/locales` directory.");
}

main().catch(console.error);
