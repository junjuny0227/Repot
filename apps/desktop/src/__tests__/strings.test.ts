import { expect, test } from "vitest";
import { osLabel, TARGET_OPTIONS, S, categoryLabel, statusText, findingText, reasonText, warningText, nextSteps } from "../strings";
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
