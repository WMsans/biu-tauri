import _generator from "@babel/generator";
import * as babelParser from "@babel/parser";
import _template from "@babel/template";
import _traverse from "@babel/traverse";
import * as t from "@babel/types";
import fs from "fs";
import path from "path";

import { LANGUAGES } from "../shared/locales/index.ts";

// Deal with esm/cjs interop
const traverse = _traverse.default;
const generate = _generator.default;
const template = _template.default;

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
  if (!str || /^[0-9\s.,!?"'()-_:/%x+]+$/.test(str)) {
    return false;
  }
  if (str.length === 1 && !/[\u4e00-\u9fa5]/.test(str)) {
    return false;
  }
  return true;
}

async function replaceStringsInFile(filePath: string, stringToKey: Map<string, string>) {
  const content = await fs.promises.readFile(filePath, "utf-8");
  const ast = babelParser.parse(content, {
    sourceType: "module",
    plugins: ["typescript", "jsx"],
  });

  let needsTFunction = false;
  let useTranslationImported = false;

  traverse(ast, {
    ImportDeclaration(path) {
      if (path.node.source.value === "react-i18next") {
        path.node.specifiers.forEach(spec => {
          if (
            spec.type === "ImportSpecifier" &&
            spec.imported.type === "Identifier" &&
            spec.imported.name === "useTranslation"
          ) {
            useTranslationImported = true;
          }
        });
      }
    },
    JSXText(path) {
      const value = path.node.value.trim().replace(/\s+/g, " ");
      if (stringToKey.has(value)) {
        needsTFunction = true;
        const key = stringToKey.get(value)!;
        const replacement = template.expression(`t('${key}')`)();
        path.replaceWith(t.jsxExpressionContainer(replacement));
      }
    },
    JSXAttribute(path) {
      const attributeName = path.node.name.name;
      if (typeof attributeName === "string" && TARGET_ATTRIBUTES.includes(attributeName)) {
        const valueNode = path.node.value;
        if (valueNode?.type === "StringLiteral") {
          const value = valueNode.value.trim();
          if (stringToKey.has(value)) {
            needsTFunction = true;
            const key = stringToKey.get(value)!;
            const replacement = template.expression(`t('${key}')`)();
            path.get("value").replaceWith(t.jsxExpressionContainer(replacement));
          }
        }
      }
    },
  });

  if (needsTFunction) {
    let tFunctionInjectedInFile = false;
    traverse(ast, {
      "FunctionDeclaration|ArrowFunctionExpression"(path) {
        if (tFunctionInjectedInFile) return;

        let isComponent = false;
        if (path.isFunctionDeclaration() && path.node.id && /^[A-Z]/.test(path.node.id.name)) {
          isComponent = true;
        } else if (path.isArrowFunctionExpression()) {
          const variableDeclarator = path.findParent(p => p.isVariableDeclarator());
          if (variableDeclarator && variableDeclarator.isVariableDeclarator()) {
            const id = variableDeclarator.node.id;
            if (t.isIdentifier(id) && /^[A-Z]/.test(id.name)) {
              isComponent = true;
            }
          }
        }

        if (isComponent) {
          const body = path.get("body");
          if (body.isBlockStatement()) {
            if (!path.scope.hasBinding("t")) {
              const useTranslationHook = template.statement("const { t } = useTranslation();")();
              body.unshiftContainer("body", useTranslationHook);
              tFunctionInjectedInFile = true; // Assume one component per file for simplicity
            }
          }
        }
      },
    });

    if (!useTranslationImported) {
      const importDeclaration = template.statement(`import { useTranslation } from 'react-i18next';`)();
      const program = ast.program;
      program.body.unshift(importDeclaration);
    }

    const { code } = generate(ast, {
      retainLines: true,
      jsescOption: { minimal: true },
    });
    await fs.promises.writeFile(filePath, code);
  }
}

async function main() {
  const replace = process.argv.includes("--replace");

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

  const localeFiles = LANGUAGES.map(lang =>
    path.join(process.cwd(), "shared", "locales", lang.value, "translation.json"),
  );

  const translations: Record<string, any> = {};
  const existingValues = new Set<string>();
  const existingKeys = new Set<string>();
  const stringToKey = new Map<string, string>();

  for (const file of localeFiles) {
    if (fs.existsSync(file)) {
      const content = await fs.promises.readFile(file, "utf-8");
      translations[file] = JSON.parse(content);
      Object.entries(translations[file]).forEach(([key, value]) => {
        existingKeys.add(key);
        if (typeof value === "string") {
          existingValues.add(value);
          stringToKey.set(value, key);
        }
      });
    }
  }

  let newStringsCount = 0;
  for (const [value, filePath] of allStrings.entries()) {
    if (!existingValues.has(value)) {
      const key = generateKey(filePath, value, existingKeys);
      existingKeys.add(key);
      stringToKey.set(value, key);
      newStringsCount++;

      for (const file of localeFiles) {
        if (translations[file]) {
          translations[file][key] = value;
        }
      }
    }
  }

  if (newStringsCount > 0) {
    for (const file of localeFiles) {
      if (fs.existsSync(file)) {
        await fs.promises.writeFile(file, JSON.stringify(translations[file], null, 2) + "\n");
      }
    }
  }

  console.log(`Extraction complete. Found ${allStrings.size} unique translatable strings.`);
  console.log(`Added ${newStringsCount} new strings to the translation files.`);

  if (replace) {
    console.log("Replacing hardcoded strings with i18n keys...");
    const filesToProcess = new Set<string>();
    allStrings.forEach(filePath => {
      filesToProcess.add(filePath);
    });

    for (const file of filesToProcess) {
      await replaceStringsInFile(file, stringToKey);
    }
    console.log("Replacement complete.");
  } else {
    console.log("To replace hardcoded strings, run the script with the --replace flag.");
  }

  console.log("Please review the changes.");
}

main().catch(console.error);
