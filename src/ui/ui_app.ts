// ======================================================
// 🚀 ui_App RemapH V3
// ======================================================

import { crearLayout } from "./ui_layout";

// ======================================================
// 🚀 CREAR APP
// ======================================================

export function crearApp(alGuardar: () => Promise<void>): HTMLElement {
  return crearLayout(alGuardar);
}
