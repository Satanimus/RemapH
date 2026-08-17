// ======================================================
// 🔢 comp_Numero
// ------------------------------------------------------
// Columna número: botón "N ▾" con popup (Clonar/Eliminar).
// Sirve también de asa para el componente de arrastre
// (util_arrastrable.ts, ver ui_tabla.ts): clic corto abre
// este popup, clic mantenido lo maneja el componente por su
// cuenta (misma clase "numero-asa" usada para ubicar el
// botón vía querySelector tras cada render).
// ======================================================

import type { ContextoFila } from "../../core/core_contexto_fila";
import type { FilaPerfil } from "../../core/core_perfil";
import { crearFila } from "../../core/core_perfil";
import { crearBoton } from "./comp_boton";
import { abrirPopupNumero } from "./comp_popup_abrir";
import { obtenerPerfilUi } from "../../core/core_perfil_ui";
import { reconstruirTabla } from "../ui_tabla_control";

export function crearNumero(
  contexto: ContextoFila,
  filaPerfil: FilaPerfil,
  numero: number,
  total: number,
  alModificar: () => void,
): HTMLElement {
  const contenedor = document.createElement("div");

  contenedor.className = "numero-celda";

  const boton = crearBoton({
    texto: `${numero} ▾`,
    titulo: "Opciones de fila",
    clase: "numero-asa",
  });

  boton.addEventListener("click", (evento) => {
    abrirPopupNumero(evento, contexto, filaPerfil, alModificar);
  });

  contenedor.append(boton);

  // ==================================================
  // ➕ AGREGAR FILA (solo debajo de la última fila)
  // ==================================================

  if (numero === total) {
    const botonAgregar = document.createElement("button");

    botonAgregar.className = "btn-agregar-fila";
    botonAgregar.type = "button";
    botonAgregar.title = "Agregar fila";

    const simbolo = document.createElement("span");

    simbolo.textContent = "+";

    botonAgregar.append(simbolo);

    botonAgregar.addEventListener("click", (evento) => {
      evento.stopPropagation();

      const perfil = obtenerPerfilUi();

      perfil.filas.push(crearFila());

      alModificar();
      reconstruirTabla();
    });

    contenedor.append(botonAgregar);
  }

  return contenedor;
}
