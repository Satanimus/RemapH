// ======================================================
// ui_Layout
// ======================================================

import {
  crearToolbar,
  marcarPerfilEditado,
  refrescarEstadoDesdeBackend,
} from "./ui_toolbar";

import { crearTabla } from "./ui_tabla";

import { crearStatusbar, actualizarStatusbar } from "./ui_statusbar";

import { crearContenedorPopup } from "../componentes/comp_popup_contenedor";

import { obtenerPerfilUi } from "../core/core_perfil_ui";

import { registrarActualizacionConflictos } from "./ui_tabla_control";

import { esSeparador } from "../core/core_separadores";

import type { FilaPerfil } from "../core/core_perfil";

// ======================================================
// CREAR LAYOUT
// ======================================================

export function crearLayout(alGuardar: () => Promise<void>): HTMLElement {
  const toolbar = crearToolbar(alGuardar);

  const statusbar = crearStatusbar(() => {
    void refrescarEstadoDesdeBackend(toolbar);
  });

  registrarActualizacionConflictos(() => {
    actualizarStatusbar(
      obtenerPerfilUi().filas.filter(
        (item): item is FilaPerfil => !esSeparador(item),
      ),
    );
  });

  const tabla = crearTabla(() => {
    marcarPerfilEditado(toolbar);
  });

  const fragment = document.createDocumentFragment();

  fragment.append(
    toolbar,

    tabla,

    statusbar,

    crearContenedorPopup(),
  );

  const contenedor = document.createElement("div");

  contenedor.className = "layout";

  contenedor.append(fragment);

  return contenedor;
}
