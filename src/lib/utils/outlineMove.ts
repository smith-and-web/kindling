/**
 * One-step keyboard moves (Move up / Move down) for an ordered list whose locked
 * rows are pinned: the backend refuses any reorder that changes a locked row's
 * index, so a move that would push a locked neighbour out of its slot is not
 * offered at all. It is not "skipped over" either, because jumping past a locked
 * row would shift that row by one and be refused just the same.
 *
 * The step is taken in the order the writer can see (`visibleIds`, e.g. with
 * sidebar filters on). Hidden rows between the item and its visible neighbour
 * keep their relative order; the item lands directly before (up) or after (down)
 * that neighbour.
 */
export function stepMoveOrder<T extends { id: string }>(
  items: readonly T[],
  visibleIds: readonly string[],
  id: string,
  step: -1 | 1,
  isLocked: (item: T) => boolean
): string[] | null {
  const moving = items.find((item) => item.id === id);
  const visibleIndex = visibleIds.indexOf(id);
  const neighbour = visibleIds[visibleIndex + step];
  if (!moving || visibleIndex === -1 || neighbour === undefined || isLocked(moving)) return null;

  const order = items.map((item) => item.id).filter((itemId) => itemId !== id);
  const at = order.indexOf(neighbour);
  if (at === -1) return null;
  order.splice(step < 0 ? at : at + 1, 0, id);

  const shiftsLockedRow = items.some(
    (item, index) => isLocked(item) && order.indexOf(item.id) !== index
  );
  return shiftsLockedRow ? null : order;
}
