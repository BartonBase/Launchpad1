"use client";
/**
 * Optional "host my token image + description" step of the Launch form. Uploads to Arweave (public, permanent) only
 * after the creator ticks the disclosure and clicks the button; the resulting URI fills the metadata URI field.
 */
import { useState } from "react";
import { UploadError, downscaleImage, uploadTokenMetadata } from "@/lib/metadata/arweave";

export function MetadataUpload({ name, symbol, onUploaded }: { name: string; symbol: string; onUploaded: (uri: string) => void }) {
  const [file, setFile] = useState<File | null>(null);
  const [description, setDescription] = useState("");
  const [ok, setOk] = useState(false);
  const [busy, setBusy] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const [done, setDone] = useState<string | null>(null);
  const ready = name.trim() !== "" && symbol.trim() !== "";

  const upload = async () => {
    setBusy(true);
    setError(null);
    try {
      const imageBytes = file ? await downscaleImage(file) : undefined;
      const r = await uploadTokenMetadata({ name, symbol, description, imageBytes });
      setDone(r.uri);
      onUploaded(r.uri);
    } catch (e) {
      setError(e instanceof UploadError ? e.message : "Upload failed. You can paste your own metadata URI instead.");
    } finally {
      setBusy(false);
    }
  };

  return (
    <div className="bg-surface-2 rounded-panel border-border space-y-3 border p-4" data-testid="launch-metadata-upload">
      <p className="text-sm font-medium">Token image and description</p>
      <div className="grid gap-3 sm:grid-cols-2">
        <div>
          <label htmlFor="md-image" className="mb-1 block text-xs font-medium">Image (PNG, JPG or WebP)</label>
          <input id="md-image" type="file" accept="image/png,image/jpeg,image/webp,image/gif" className="text-xs" onChange={(e) => setFile(e.target.files?.[0] ?? null)} data-testid="launch-metadata-image" />
          <p className="text-dim mt-1 text-xs">Resized to 512 px before upload.</p>
        </div>
        <div>
          <label htmlFor="md-desc" className="mb-1 block text-xs font-medium">Description</label>
          <textarea id="md-desc" className="input text-xs" rows={3} maxLength={500} value={description} onChange={(e) => setDescription(e.target.value)} data-testid="launch-metadata-description" />
        </div>
      </div>
      <label className="flex items-start gap-2 text-xs">
        <input type="checkbox" className="mt-0.5" checked={ok} onChange={(e) => setOk(e.target.checked)} data-testid="launch-metadata-consent" />
        <span className="text-muted">I understand the image, name and description are uploaded to Arweave, where they are public and permanent. They can&apos;t be deleted or edited.</span>
      </label>
      <button type="button" className="btn btn-sm" disabled={!ok || !ready || busy} onClick={() => void upload()} data-testid="launch-metadata-upload-btn">
        {busy ? "Uploading…" : "Upload to Arweave"}
      </button>
      {!ready && <p className="text-dim text-xs">Enter the name and ticker first.</p>}
      {done && <p className="text-positive-text text-xs" role="status" data-testid="launch-metadata-done">Uploaded. The metadata URI below is filled in.</p>}
      {error && <p className="text-warning text-xs" role="alert" data-testid="launch-metadata-error">{error}</p>}
    </div>
  );
}
