import { expect, test } from "vitest";
import { osLabel, TARGET_OPTIONS, S, categoryLabel, statusText, findingText, reasonText, warningText, nextSteps, selectedCount, revealSecretLine } from "../strings";
test("labels all targets and structured reports", () => {
  expect(TARGET_OPTIONS.map(osLabel)).toEqual(["macOS", "Windows", "Linux (Debian/Ubuntu)", "Linux (Fedora)", "Linux (Arch)"]);
  expect(categoryLabel("globalPackage")).toBe("Global packages");
  expect(statusText({ ok: { count: 3 } })).toContain("3");
  expect(statusText({ failed: { message: "failed", detail: "raw detail" } })).toBe("failed");
  expect(findingText({ line: 2, kind: { secret: { name: "API_TOKEN" } } })).toContain("API_TOKEN");
  expect(reasonText({ differentOs: { source: "macOs" } })).toContain("macOS");
  expect(warningText({ readFailed: { itemId: "git:file" } }, "Git config")).toContain("Git config");
});
test("next steps use destination format and OS", () => {
  expect(nextSteps("macOs", "script")).toEqual(["Copy this folder to the new machine.", "Run: ./restore.sh"]);
  expect(nextSteps("windows", "script")).toContain("Run: powershell -ExecutionPolicy Bypass -File restore.ps1");
  expect(nextSteps("windows", "manifest")).toEqual(["Open README.md in this folder and follow the steps."]);
});

test("scan timeout copy matches the 60-second provider limit", () => {
  expect(S.scanIntro).toContain("60 seconds");
  expect(statusText("timedOut")).toBe("Timed out (60 s)");
});

test("counts use the plural form for the total", () => {
  expect(selectedCount(1, 1)).toBe("1 of 1 item selected");
  expect(selectedCount(2, 12)).toBe("2 of 12 items selected");
  expect(revealSecretLine(4)).toBe("Export the original value of line 4");
});

test("open errors map to messages; unknown errors pass through", async () => {
  const { openErrorText, openedFromText, skippedItemsText } = await import("../strings");
  expect(openErrorText("newerVersion:2")).toBe("This file was made by a newer version of Repot. Update Repot to open it.");
  expect(openErrorText("notRepotFile")).toContain("repot.json");
  expect(openErrorText("readFailed")).toBe("The file couldn’t be read.");
  expect(openErrorText(new Error("boom"))).toBe("boom");
  expect(openErrorText("hasOwnProperty")).toBe("hasOwnProperty");
  expect(openedFromText({ source: "macOs", createdAt: "2026-11-04T10:00Z" })).toBe("From a Repot file made on macOS, Nov 4, 2026");
  expect(skippedItemsText(1)).toBe("1 item this version of Repot can’t read was skipped.");
});

test("secrets from an opened Repot file are described as already masked", async () => {
  const { findingText } = await import("../strings");
  const f = { line: 2, kind: { secret: { name: "API_TOKEN" } } };
  expect(findingText(f, true)).toBe("Line 2: Masked in the Repot file (API_TOKEN). Fill in the value after restoring.");
  expect(findingText(f)).toContain("unless you allow the original value");
});
