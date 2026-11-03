import { expect, test } from "vitest";
import catalogRs from "../../../../crates/core/src/catalog.rs?raw";
import en from "../locales/en.json";
import { itemDescription } from "../strings";
import type { Item } from "../types";

// core keeps the English text (catalog.rs) and sends its id; the UI translates by id.
// en.json must say exactly what core says, so the English UI does not depend on which side wins.
const coreEntries = Object.fromEntries([...catalogRs.matchAll(/\(\s*"([^"]+)",\s*"([^"]+)"\s*\)/g)].map((m) => [m[1], m[2]]));

test("catalog in en.json matches the core catalog", () => {
  expect(Object.keys(coreEntries).length).toBeGreaterThan(10);
  expect(en.catalog).toEqual(coreEntries);
});

const fixture = (await import("../__fixtures__/scan.json")).default;
const items = fixture.scan.items as unknown as Item[];

test("catalog descriptions are looked up by id", () => {
  const zshrc = items.find((i) => i.id === "shell:file:.zshrc")!;
  expect(zshrc.catalogId).toBe("file:zshrc");
  expect(itemDescription(zshrc)).toBe(en.catalog["file:zshrc"]);
});

test("unknown catalog ids fall back to the core description, tool descriptions pass through", () => {
  const zshrc = items.find((i) => i.id === "shell:file:.zshrc")!;
  expect(itemDescription({ ...zshrc, catalogId: "file:not-yet-translated", description: "Core text." })).toBe("Core text.");
  expect(itemDescription({ ...zshrc, catalogId: "toString", description: "Core text." })).toBe("Core text.");
  const npm = items.find((i) => i.id === "npm:typescript")!;
  expect(npm.catalogId).toBeNull();
  expect(itemDescription(npm)).toBe("Typed JavaScript tooling");
  expect(itemDescription({ ...npm, description: null })).toBeNull();
});
