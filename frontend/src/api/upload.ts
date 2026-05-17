// XHR-based uploader so we can surface progress events. fetch() can't.

import { getToken } from "@/lib/token";
import type { UploadMediaResp, UploadSubtitleResp } from "./types";

export interface UploadHandlers {
  onProgress?: (loaded: number, total: number | null) => void;
  signal?: AbortSignal;
}

export function uploadMedia(file: File, h: UploadHandlers = {}): Promise<UploadMediaResp> {
  return doUpload<UploadMediaResp>("/api/library/upload-media", file, h);
}

export function uploadSubtitle(file: File, h: UploadHandlers = {}): Promise<UploadSubtitleResp> {
  return doUpload<UploadSubtitleResp>("/api/library/upload-subtitle", file, h);
}

function doUpload<T>(url: string, file: File, h: UploadHandlers): Promise<T> {
  return new Promise((resolve, reject) => {
    const xhr = new XMLHttpRequest();
    const form = new FormData();
    form.append("file", file, file.name);

    if (h.signal) {
      h.signal.addEventListener("abort", () => xhr.abort());
    }

    xhr.open("POST", url, true);
    const token = getToken();
    if (token) xhr.setRequestHeader("Authorization", `Bearer ${token}`);
    xhr.upload.addEventListener("progress", (e) => {
      if (h.onProgress) {
        h.onProgress(e.loaded, e.lengthComputable ? e.total : null);
      }
    });
    xhr.onerror = () => reject(new Error("network error"));
    xhr.onabort = () => reject(new Error("aborted"));
    xhr.onload = () => {
      if (xhr.status >= 200 && xhr.status < 300) {
        try {
          resolve(JSON.parse(xhr.responseText) as T);
        } catch (e) {
          reject(e instanceof Error ? e : new Error(String(e)));
        }
      } else {
        reject(new Error(`HTTP ${xhr.status}: ${xhr.responseText}`));
      }
    };
    xhr.send(form);
  });
}
