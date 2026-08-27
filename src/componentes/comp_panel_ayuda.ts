// ======================================================
// ❔ comp_Panel_Ayuda
// ------------------------------------------------------
// Panel lateral derecho, persistente (toggle ❔), mismo
// patrón de referencias a nivel de módulo que
// comp_panel_lateral.ts. Muestra el contenido de ayuda.txt
// (backend) para el objeto bajo el mouse (ver Etapa F).
// ======================================================

import { invoke } from "@tauri-apps/api/core";

import { renderizarAyuda } from "../util/util_texto_ayuda";

// ======================================================
// MÓDULO: referencias del panel activo
// ------------------------------------------------------
// Un solo panel por app (mismo patrón que comp_panel_lateral.ts).
// ======================================================

let panelElemento: HTMLElement | null = null;
let cuerpoPanel: HTMLElement | null = null;

const ID_BOTON_AYUDA = "boton_panel_ayuda";

const ANCHO_AYUDA_DEFAULT = 280;
const ANCHO_AYUDA_MINIMO = 200;

function aplicarAncho(ancho: number): void {
  if (!panelElemento) {
    return;
  }

  panelElemento.style.setProperty(
    "--ancho-ayuda",
    `${Math.max(ANCHO_AYUDA_MINIMO, ancho)}px`,
  );
}

// ======================================================
// CREAR PANEL
// ======================================================

export function crearPanelAyuda(): HTMLElement {
  const panel = document.createElement("div");

  panel.className = "panel-ayuda";

  const resize = document.createElement("div");

  resize.className = "panel-ayuda-resize";

  cuerpoPanel = document.createElement("div");

  cuerpoPanel.className = "panel-ayuda-cuerpo";

  panel.append(resize, cuerpoPanel);

  panelElemento = panel;

  resize.addEventListener("mousedown", (evento) => {
    evento.preventDefault();

    iniciarArrastreAncho(evento.clientX);
  });

  void inicializarPanelAyuda();

  return panel;
}

// ======================================================
// ↔️ ARRASTRE DE ANCHO
// ======================================================

function iniciarArrastreAncho(inicioX: number): void {
  if (!panelElemento) {
    return;
  }

  const anchoInicial = panelElemento.getBoundingClientRect().width;

  const mover = (evento: MouseEvent) => {
    aplicarAncho(anchoInicial + (inicioX - evento.clientX));
  };

  const soltar = () => {
    window.removeEventListener("mousemove", mover);

    window.removeEventListener("mouseup", soltar);

    if (!panelElemento) {
      return;
    }

    const anchoFinal = panelElemento.getBoundingClientRect().width;

    invoke("establecer_ancho_panel_ayuda", {
      ancho: Math.round(anchoFinal),
    }).catch((error) => {
      console.error(
        "❌ No se pudo guardar el ancho del panel de ayuda:",
        error,
      );
    });
  };

  window.addEventListener("mousemove", mover);

  window.addEventListener("mouseup", soltar);
}

// ======================================================
// 🚀 INICIALIZACIÓN (primer inicio / estado persistido)
// ======================================================

async function inicializarPanelAyuda(): Promise<void> {
  let ancho = ANCHO_AYUDA_DEFAULT;

  try {
    ancho =
      (await invoke<number | null>("obtener_ancho_panel_ayuda")) ??
      ANCHO_AYUDA_DEFAULT;
  } catch (error) {
    console.error(
      "❌ No se pudo consultar el ancho del panel de ayuda:",
      error,
    );
  }

  aplicarAncho(ancho);

  let primerInicio = false;

  try {
    primerInicio = await invoke<boolean>("obtener_primer_inicio_ayuda");
  } catch (error) {
    console.error("❌ No se pudo consultar el primer inicio de ayuda:", error);
  }

  if (primerInicio) {
    if (cuerpoPanel) {
      try {
        const contenido = await invoke<string | null>("obtener_ayuda", {
          idObjeto: ID_BOTON_AYUDA,
        });

        if (contenido) {
          mostrarContenidoAyuda(contenido);
        }
      } catch (error) {
        console.error(
          "❌ No se pudo obtener el contenido de bienvenida:",
          error,
        );
      }
    }

    abrirPanelAyuda();

    return;
  }

  let visible = false;

  try {
    visible =
      (await invoke<boolean | null>("obtener_visible_panel_ayuda")) ?? false;
  } catch (error) {
    console.error(
      "❌ No se pudo consultar la visibilidad del panel de ayuda:",
      error,
    );
  }

  if (visible) {
    panelElemento?.classList.add("abierto");
  } else {
    panelElemento?.classList.remove("abierto");
  }
}

// ======================================================
// 🔌 ABRIR / CERRAR / ALTERNAR
// ======================================================

export function abrirPanelAyuda(): void {
  panelElemento?.classList.add("abierto");

  invoke("establecer_visible_panel_ayuda", { visible: true }).catch((error) => {
    console.error(
      "❌ No se pudo guardar la visibilidad del panel de ayuda:",
      error,
    );
  });
}

export function cerrarPanelAyuda(): void {
  panelElemento?.classList.remove("abierto");

  invoke("establecer_visible_panel_ayuda", { visible: false }).catch(
    (error) => {
      console.error(
        "❌ No se pudo guardar la visibilidad del panel de ayuda:",
        error,
      );
    },
  );
}

export function alternarPanelAyuda(): void {
  if (panelElemento?.classList.contains("abierto")) {
    cerrarPanelAyuda();
  } else {
    abrirPanelAyuda();
  }
}

export function mostrarContenidoAyuda(contenido: string): void {
  if (cuerpoPanel) {
    cuerpoPanel.replaceChildren(renderizarAyuda(contenido));
  }
}
