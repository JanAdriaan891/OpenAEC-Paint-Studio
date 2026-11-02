# Project storage and object model decision

Decision date: 2 November 2026. Implements the storage decision in #1 and the design deliverable in #4. The API v1 document and existing IFCX/IFCXB raster contracts remain compatible.

## Editable storage

Save / Ctrl+S writes `name.paint.json`. Open Project / Ctrl+Shift+O opens this format; the ordinary Open Image chooser continues to show only image extensions. Filename arguments use the same validated reader. The envelope is JSON:

```json
{"format":"openaec-paint-project","version":1,"document":{"format":"openaec-paint","version":1}}
```

The abbreviated `document` above is the existing full v1 document, including name, dimensions, background, keep-vectors setting, ordered operations, embedded PNGs, multiline text, style and matrices. Assets are embedded; there are no external file references. The 64 MB input/output limit and canvas limits apply. Readers validate the complete document and render its embedded content before replacing current work. Unknown envelope versions are rejected. A future schema needs an explicit, tested migration; readers must not silently downgrade it. Undo stacks, selection, credentials and UI preferences are not portable content.

There is no `.oapaint` extension. IFCX and IFCXB continue to contain one PNG raster; Save now preserves editability, while Export deliberately produces PNG/JPEG/WebP/SVG/IFCX/IFCXB. Raster export does not clear the unsaved-project flag. In the browser a cancellable Download dialog hands the file to the browser; browsers do not expose disk-write completion or cancellation of their own download manager. Cancelling our dialog keeps the unsaved flag. Native saving writes a temporary sibling, syncs it and atomically replaces the target; a failed write keeps the existing target. A cancelled or failed project save cannot clear it either; completing a save only marks the exact saved document clean. Existing v1 API documents and local drafts remain readable without conversion. A recovered draft is marked unsaved until exported as a portable project; a late draft-recovery result cannot overwrite a file already opened during startup.

## Options for overlapping content

| Option | Selection, order, hide and lock | Brush, eraser and fill | Cost / compatibility |
| --- | --- | --- | --- |
| Object panel | Select operations; reorder them; per-object hidden/locked metadata in a future schema | Brush adds a stroke; eraser affects preceding composition; fill depends on preceding visible operations | Smallest extension, but reordering/hiding can change later fill/eraser results. Needs explicit semantics and stable IDs. |
| Raster layers | Select named raster surfaces; reorder/hide/lock a surface | Brush, eraser and fill affect the active surface; alpha reveals lower layers | Predictable painting, but loses individually editable text unless another model is added. Changes storage, undo, exports and API. |
| Hybrid | Named surfaces containing retained text/shapes, plus pixels | Painting applies to active surface; flattening resolves that surface only | Most flexible, largest model/memory/migration burden; not justified as the first implementation. |

Choose the **object-panel direction first**, with a separately versioned design before visibility, locking or reordering ships. This issue does not build a full layer system. The implemented first step is selecting subsets in the existing inspector, Shift+click on canvas, and a compact Objects and Alignment dialog available through F6 and the ribbon at 560 px. Six alignment actions use the union of the selected text/shape bounds and preserve individual transforms. There are no pretend raster layers or hidden/locked controls backed only by UI state.

Interaction proposal for photo + three labels + overlapping shape:

1. Open the photo, add labels and a shape; a future object list shows topmost first with a stable name/ID.
2. Tick only the three labels and align them; selecting via checkboxes remains possible when objects overlap.
3. Move a selected object one step forward/back using order controls. Explain that procedural eraser/fill results may change; alternatively resolve these into a raster before permitting that change.
4. Eye hides an object from rendering/export; lock prevents hit selection, transforms and deletion while leaving it visible. Hidden/locked metadata is persisted and undoable. Panel selection can still inspect a locked item.
5. On a 560 px screen use the F6/ribbon modal with vertical scrolling; the ribbon continues to collapse groups without horizontal scrolling. Desktop uses the same commands in the inspector. Tab/Space/Enter operate selection and alignment.

## Current pixel semantics

Rectangular Pixels mode samples the **transparent composition**, not one image operation. The document background remains separate. Selecting or copying does not flatten content. Moving, deleting or cutting flattens the composition into one PNG operation, with an explicit mode-entry explanation. Cleared pixels reveal the configured background/alpha; moved pixels are clipped at the canvas boundary and source-over composited at the destination. One action is one undo step, including restoration of all original vectors. Escape cancels the gesture and returns to object selection. Whole-canvas Copy includes the visible background; pixel-selection Copy/Cut preserves the composition's alpha. Cut changes the document only after clipboard write succeeds.

## Follow-up implementation tasks from #4

These are design outputs, not capabilities claimed by this change:

- **O1 Stable object identity and metadata:** design document v2 names, visibility and locking; migrate v1 drafts/projects and decide whether API v2 is needed. Old API clients must not silently discard metadata.
- **O2 Ordering and procedural paint:** choose whether fill/eraser operations remain procedural or are resolved before reordering. Add overlap/alpha and undo acceptance cases before adding order buttons.
- **O3 Persistent object panel:** implement eye/lock/order with the decided model; preserve the F6/560 px flow. Verify photo + three labels + overlapping shape end to end.
- **O4 Export and migration:** specify SVG hidden/locked metadata, IFCX visible raster output, draft migration and API/MCP schema negotiation. Project roundtrips must preserve all metadata and appearance.

Full raster layers, groups and blend modes require another product decision. No current feature claim here is based on competitors' planned roadmap functionality.
