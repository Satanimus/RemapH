// ======================================================
// ui_Layout
// ======================================================

import {
  crearToolbar,
  marcarPerfilEditado,
  refrescarEstadoDesdeBackend,
  obtenerEstadoPerfilActual,
  aplicarResultadoPerfilEnToolbar,
} from "./ui_toolbar";

import { crearTabla } from "./ui_tabla";

import { crearStatusbar, actualizarStatusbar } from "./ui_statusbar";

import { crearContenedorPopup } from "../componentes/comp_popup_contenedor";

import { crearPanelLateral } from "../componentes/comp_panel_lateral";

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

  const panelLateral = crearPanelLateral(
    alGuardar,
    () => obtenerEstadoPerfilActual(toolbar),
    (resultado) => aplicarResultadoPerfilEnToolbar(toolbar, resultado),
  );

  const layoutCuerpo = document.createElement("div");

  layoutCuerpo.className = "layout-cuerpo";

  layoutCuerpo.append(panelLateral, tabla);

  const fragment = document.createDocumentFragment();

  fragment.append(
    toolbar,

    layoutCuerpo,

    statusbar,

    crearContenedorPopup(),
  );

  const contenedor = document.createElement("div");

  contenedor.className = "layout";

  contenedor.append(fragment);

  return contenedor;
}
