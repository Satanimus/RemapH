// ======================================================
// 🚀 ui_App
// ======================================================

import { crearLayout } from "./ui_layout";

// ======================================================
// 🚀 CREAR APP
// ======================================================

export function crearApp(alGuardar: () => Promise<void>): HTMLElement {
  return crearLayout(alGuardar);
}
