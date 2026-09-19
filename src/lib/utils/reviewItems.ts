import type { EditorialMessage } from "./editorial";
export interface ReviewItem {
  id: string;
  kind: "comment" | "suggestion";
  state: string;
  author: string;
  excerpt: string;
  messages: EditorialMessage[];
  before: string;
  after: string;
  current?: string;
  conflict?: boolean;
  locked?: boolean;
  unavailable?: string;
  decide?: boolean;
  withdraw?: boolean;
  resolve?: boolean;
  reanchor?: boolean;
}
