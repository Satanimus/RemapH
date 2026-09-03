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
  // Color ya asignado a la fila (clave de --tag-<valor>, ver
  // COLOR_OPCIONES en comp_popup_abrir.ts). Si viene vacío/undefined
  // (encabezado con selección múltiple, separador) el botón se queda
  // con el ícono 🎨 genérico.
  colorActual?: string;
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

  if (config.colorActual) {
    const muestra = document.createElement("span");

    muestra.className = "color-control-muestra";
    muestra.style.background = `var(--tag-${config.colorActual})`;

    botonColor.replaceChildren(muestra);
  }

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

  botonEliminar.classList.add("boton-peligro");

  let confirmando = false;

  // Cancela la doble confirmación: clic en cualquier otro lugar o
  // cualquier tecla la descarta y el botón vuelve a su estado normal
  // (no debe quedar "pegado" en confirmación tras otras acciones).
  const cancelarConfirmacion = (): void => {
    confirmando = false;

    botonEliminar.textContent = "X";
    botonEliminar.classList.remove("opciones-boton-eliminar--confirmando");

    document.documentElement.style.removeProperty(
      "--ancho-opciones-confirmando",
    );

    document.removeEventListener("pointerdown", alHacerClickFuera, true);
    document.removeEventListener("keydown", cancelarConfirmacion, true);
  };

  const alHacerClickFuera = (evento: Event): void => {
    if (evento.target instanceof Node && botonEliminar.contains(evento.target)) {
      return;
    }

    cancelarConfirmacion();
  };

  botonEliminar.addEventListener("click", () => {
    if (config.requiereConfirmacion && !confirmando) {
      confirmando = true;

      botonEliminar.textContent = "X Confirmar eliminación";
      botonEliminar.classList.add("opciones-boton-eliminar--confirmando");

      // Mide el botón ya expandido (⁝/Color/Duplicar de esta celda
      // quedan ocultos por CSS al agregar la clase de arriba, ver
      // styl_tabla.css) y publica ese ancho + el padding horizontal
      // de la celda (2 × var(--gap8) = 16px) para que TODA la
      // columna Opciones (cabecera, filas, separadores) lo adopte
      // por igual — ver selector .viewport:has(...) en
      // styl_tabla.css.
      const anchoBoton = botonEliminar.getBoundingClientRect().width;

      document.documentElement.style.setProperty(
        "--ancho-opciones-confirmando",
        `${anchoBoton + 16}px`,
      );

      document.addEventListener("pointerdown", alHacerClickFuera, true);
      document.addEventListener("keydown", cancelarConfirmacion, true);

      return;
    }

    if (confirmando) {
      cancelarConfirmacion();
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
        colorActual: filaPerfil.color,
      }),
    );
  }

  return contenedor;
}
