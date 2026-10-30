import { invoke } from "@tauri-apps/api/core";
import { t } from "./i18n";
import type { ReferenceCopyPreview, ReferenceCopyRequest, ReferenceCopyResult } from "./types";

export function previewReferenceCopy(request: ReferenceCopyRequest) {
  return invoke<ReferenceCopyPreview>("preview_reference_copy", { request });
}
export function copyReferences(request: ReferenceCopyRequest, expectedRevision: string) {
  return invoke<ReferenceCopyResult>("copy_references_between_projects", {
    request,
    expectedRevision,
  });
}
export function copyErrorMessage(error: unknown): string {
  if (
    typeof error === "object" &&
    error !== null &&
    "message" in error &&
    typeof error.message === "string"
  )
    return error.message;
  return typeof error === "string" ? error : t("logic.referenceCopy.failed");
}
