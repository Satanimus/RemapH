// ======================================================
// 🔢 comp_Numero RemapH V3
// ------------------------------------------------------
// Columna número: botón "N ▾" con popup (Mover/Clonar/
// Eliminar) en modo normal, o flechas ▲▼ de reordenar
// cuando la tabla está en modo mover (global).
// ======================================================

import type { ContextoFila } from "../../core/core_contexto_fila";
import type { FilaPerfil } from "../../core/core_perfil";
import { crearBoton } from "./comp_boton";
import { abrirPopupNumero } from "./comp_popup_abrir";
import { moverFilaPorId } from "../../core/core_perfil_acciones";
import { estaEnModoMover, reconstruirTabla } from "../ui_tabla_control";

export function crearNumero(
  contexto: ContextoFila,
  filaPerfil: FilaPerfil,
  numero: number,
  total: number,
  alModificar: () => void,
): HTMLElement {
  if (estaEnModoMover()) {
    return crearNumeroMover(contexto, numero, total, alModificar);
  }

  const boton = crearBoton({
    texto: `${numero} ▾`,
    titulo: "Opciones de fila",
  });

  boton.addEventListener("click", (evento) => {
    abrirPopupNumero(evento, contexto, filaPerfil, alModificar);
  });

  return boton;
}

// ======================================================
// ↕️ VARIANTE MODO MOVER
// ------------------------------------------------------
// El número queda de fondo como referencia. Las flechas
// cubren todo el botón, sin zonas muertas (mitad de
// arriba sube, mitad de abajo baja). No se muestra la
// flecha si la fila está en el borde correspondiente.
// ======================================================

function crearNumeroMover(
  contexto: ContextoFila,
  numero: number,
  total: number,
  alModificar: () => void,
): HTMLElement {
  const contenedor = document.createElement("div");

  contenedor.className = "numero-mover";

  const texto = document.createElement("span");

  texto.className = "numero-mover-texto";
  texto.textContent = String(numero);

  contenedor.append(texto);

  if (numero > 1) {
    const arriba = document.createElement("button");

    arriba.className = "numero-mover-flecha numero-mover-arriba";
    arriba.textContent = "▲";
    arriba.title = "Subir fila";

    arriba.addEventListener("click", () => {
      moverFilaPorId(contexto.id, "arriba");
      alModificar();
      reconstruirTabla();
    });

    contenedor.append(arriba);
  }

  if (numero < total) {
    const abajo = document.createElement("button");

    abajo.className = "numero-mover-flecha numero-mover-abajo";
    abajo.textContent = "▼";
    abajo.title = "Bajar fila";

    abajo.addEventListener("click", () => {
      moverFilaPorId(contexto.id, "abajo");
      alModificar();
      reconstruirTabla();
    });

    contenedor.append(abajo);
  }

  return contenedor;
}
