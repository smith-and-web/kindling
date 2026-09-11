import { openSync, closeSync, writeFileSync, unlinkSync, readFileSync, mkdirSync } from "node:fs";
import { dirname } from "node:path";
import { randomUUID } from "node:crypto";

// Exclusive creation prevents two drivers from sharing one hidden app. A crash
// leaves evidence rather than allowing a second run to adopt an unfinished fixture.
export function acquireLock(path) {
  mkdirSync(dirname(path), { recursive: true });
  let fd;
  try {
    fd = openSync(path, "wx", 0o600);
  } catch (error) {
    if (error.code !== "EEXIST") throw error;
    throw new Error(
      `QA lock exists: ${path}. Check its PID; remove only after that process has stopped and fixture recovery is complete.`
    );
  }
  const record = JSON.stringify({ pid: process.pid, id: randomUUID() });
  try {
    writeFileSync(fd, record);
  } finally {
    closeSync(fd);
  }
  return () => {
    try {
      if (readFileSync(path, "utf8") === record) unlinkSync(path);
    } catch (error) {
      if (error.code !== "ENOENT") throw error;
    }
  };
}
