// ======================================================
// ⁝ comp_Opciones
// ------------------------------------------------------
// Columna "Opciones": botón "⁝" toggle que expande/contrae
// (para TODA la tabla, ver ui_tabla_control.ts::
// alternarOpcionesColumna) los 3 botones de acción — 🎨
// Color, ⧉ Duplicar, X Eliminar (crearBotonesOpcionesExtra,
// reutilizado también por el separador — ver ui_separador.ts).
// El botón "⁝" sigue sirviendo de asa para el componente de
// arrastre (util_arrastrable.ts): clic corto togglea la
// columna, clic mantenido/arrastrado lo maneja el componente
// por su cuenta (misma clase "opciones-asa" usada para
// ubicar el botón vía querySelector tras cada render — ver
// ui_tabla.ts).
// ======================================================

import type { ContextoFila } from "../core/core_contexto_fila";
import type { FilaPerfil } from "../core/core_perfil";
import { crearBoton } from "./comp_boton";
import { abrirPopupColor } from "./comp_popup_abrir";
import {
  clonarFilaPorId,
  eliminarFilaPorId,
  filaTieneAccion,
} from "../core/core_perfil_acciones";
import {
  reconstruirTabla,
  opcionesColumnaEstaExpandida,
  alternarOpcionesColumna,
} from "../ui/ui_tabla_control";

// ======================================================
// 🎨 ⧉ X — BOTONES EXTRA (compartidos con separador)
// ------------------------------------------------------
// Eliminar pide doble clic SOLO si requiereConfirmacion es
// true (mismo criterio que antes: filaTieneAccion para
// filas, false para separadores — ver ui_separador.ts). El
// primer clic expande el botón mostrando el texto de
// confirmación; el segundo clic ejecuta.
// ======================================================

export interface OpcionesExtraConfig {
  onAbrirColor: (evento: MouseEvent) => void;
  onDuplicar: () => void;
  onEliminar: () => void;
  requiereConfirmacion: boolean;
}

export function crearBotonesOpcionesExtra(
  config: OpcionesExtraConfig,
): HTMLElement {
  const extra = document.createElement("div");

  extra.className = "opciones-extra";

  const botonColor = crearBoton({
    texto: "🎨",
    titulo: "Color",
    clase: "opciones-boton-color",
  });

  botonColor.addEventListener("click", (evento) => {
    config.onAbrirColor(evento);
  });

  const botonDuplicar = crearBoton({
    texto: "⧉",
    titulo: "Duplicar",
    clase: "opciones-boton-duplicar",
  });

  botonDuplicar.addEventListener("click", () => {
    config.onDuplicar();
  });

  const botonEliminar = crearBoton({
    texto: "X",
    titulo: "Eliminar",
    clase: "opciones-boton-eliminar",
  });

  let confirmando = false;

  botonEliminar.addEventListener("click", () => {
    if (config.requiereConfirmacion && !confirmando) {
      confirmando = true;

      botonEliminar.textContent = "X Confirmar eliminación";
      botonEliminar.classList.add("opciones-boton-eliminar--confirmando");

      return;
    }

    config.onEliminar();
  });

  extra.append(botonColor, botonDuplicar, botonEliminar);

  return extra;
}

// ======================================================
// CREAR OPCIONES (fila)
// ======================================================

export function crearOpciones(
  contexto: ContextoFila,
  filaPerfil: FilaPerfil,
  _esUltima: boolean,
  alModificar: () => void,
): HTMLElement {
  const contenedor = document.createElement("div");

  contenedor.className = "opciones-celda";

  const boton = crearBoton({
    texto: "⁝",
    titulo: "Opciones de fila",
    clase: "opciones-asa",
  });

  boton.addEventListener("click", () => {
    alternarOpcionesColumna();
  });

  contenedor.append(boton);

  if (opcionesColumnaEstaExpandida()) {
    contenedor.append(
      crearBotonesOpcionesExtra({
        onAbrirColor: (evento) => {
          abrirPopupColor(evento, contexto, filaPerfil);
        },
        onDuplicar: () => {
          clonarFilaPorId(contexto.id);
          alModificar();
          reconstruirTabla();
        },
        onEliminar: () => {
          eliminarFilaPorId(contexto.id);
          alModificar();
          reconstruirTabla();
        },
        requiereConfirmacion: filaTieneAccion(filaPerfil),
      }),
    );
  }

  return contenedor;
}
