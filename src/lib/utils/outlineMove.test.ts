import { describe, expect, it } from "vitest";
import { stepMoveOrder } from "./outlineMove";

type Row = { id: string; locked?: boolean };
const rows = (spec: string): Row[] =>
  spec.split(" ").map((token) => ({ id: token.replace("*", ""), locked: token.endsWith("*") }));
const locked = (row: Row) => !!row.locked;
const all = (items: Row[]) => items.map((row) => row.id);

describe("stepMoveOrder", () => {
  it("swaps with the neighbour when nothing is locked", () => {
    const items = rows("a b c");
    expect(stepMoveOrder(items, all(items), "b", -1, locked)).toEqual(["b", "a", "c"]);
    expect(stepMoveOrder(items, all(items), "b", 1, locked)).toEqual(["a", "c", "b"]);
  });

  it("refuses at either end, for unknown or hidden items, and for a locked item", () => {
    const items = rows("a b* c");
    expect(stepMoveOrder(items, all(items), "a", -1, locked)).toBeNull();
    expect(stepMoveOrder(items, all(items), "c", 1, locked)).toBeNull();
    expect(stepMoveOrder(items, all(items), "missing", 1, locked)).toBeNull();
    expect(stepMoveOrder(items, ["a", "c"], "b", 1, locked)).toBeNull();
    expect(stepMoveOrder(items, all(items), "b", -1, locked)).toBeNull();
    // A visible id that isn't in the list can't be a destination.
    expect(stepMoveOrder(items, ["a", "ghost"], "a", 1, locked)).toBeNull();
  });

  it("refuses a move that would shift a locked neighbour out of its slot", () => {
    const items = rows("a l* c");
    expect(stepMoveOrder(items, all(items), "a", 1, locked)).toBeNull();
    expect(stepMoveOrder(items, all(items), "c", -1, locked)).toBeNull();
    // Moves that leave the locked row where it is are still offered.
    const longer = rows("a b l* c d");
    expect(stepMoveOrder(longer, all(longer), "a", 1, locked)).toEqual(["b", "a", "l", "c", "d"]);
    expect(stepMoveOrder(longer, all(longer), "d", -1, locked)).toEqual(["a", "b", "l", "d", "c"]);
  });

  it("steps over hidden rows in the visible order, keeping theirs", () => {
    const items = rows("a n1 n2 b c");
    const visible = ["a", "b", "c"];
    expect(stepMoveOrder(items, visible, "a", 1, locked)).toEqual(["n1", "n2", "b", "a", "c"]);
    expect(stepMoveOrder(items, visible, "b", -1, locked)).toEqual(["b", "a", "n1", "n2", "c"]);
    // Stepping over a hidden row still counts as moving it when it is locked.
    const pinned = rows("a n* b");
    expect(stepMoveOrder(pinned, ["a", "b"], "a", 1, locked)).toBeNull();
  });
});
