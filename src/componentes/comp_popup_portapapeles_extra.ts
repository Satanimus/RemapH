// ======================================================
// 📋🎛️ comp_Popup_Portapapeles_Extra
// ------------------------------------------------------
// Popup Extra de Portapapeles (filaPerfil.tipo === "portapapeles"),
// abierto desde crearExtra() en comp_controles.ts. Mismo patrón
// persistente que el popup Extra de MenuExpress
// (comp_popup_menu_express_extra.ts): elegir una opción actualiza
// filaPerfil.portapapelesExtra y redibuja el mismo popup en el
// lugar, en vez de cerrarlo.
//
// Secciones (ver spec "Extra (popup)"):
//   COMPORTAMIENTO      → portapapelesExtra.comportamiento
//   UBICACIÓN           → portapapelesExtra.ubicacion
//   TAMAÑO DE BOTONES   → portapapelesExtra.tamanoBoton
//   TAMAÑO DE TEXTO     → portapapelesExtra.tamanoTexto
//   LÍMITE DE ELEMENTOS → portapapelesExtra.limite (solo rotatorios —
//     el límite REAL que aplica el pool compartido es el mayor entre
//     todos los Portapapeles en modo Registro, ver back_portapapeles.rs;
//     este campo es solo lo que ESTA fila pide)
// ======================================================

import { mostrarPopup } from "./comp_popup_contenedor";

import { reconstruirFila } from "../ui/ui_tabla_control";

import type { ContextoFila } from "../core/core_contexto_fila";

import type { FilaPerfil } from "../core/core_perfil";

import type {
  ComportamientoPortapapeles,
  UbicacionPortapapeles,
  TamanoBotonPortapapeles,
  TamanoTextoPortapapeles,
} from "../core/core_portapapeles";

import { crearGrupoOpciones, crearFilaPopup } from "./comp_popup_grupo";

// ======================================================
// ➖ SEPARADOR (mismo estilo que el resto de los popups)
// ======================================================

function crearSeparador(): HTMLElement {
  const separador = document.createElement("div");

  separador.className = "app-popup-separador";

  return separador;
}

// ======================================================
// 🔢 CAMPO "LÍMITE DE ELEMENTOS"
// ------------------------------------------------------
// Texto libre, se interpreta al perder foco o presionar Enter —
// mismo criterio de commit que Limitar Cuadrícula en
// comp_popup_menu_express_extra.ts, pero acá es un único número
// (siempre entero >= 1, sin pareja "Auto"/0).
// ======================================================

function interpretarLimite(texto: string): number {
  const valor = parseInt(texto, 10);

  return Number.isFinite(valor) && valor > 0 ? valor : 1;
}

function crearCampoLimite(
  valorActual: number,
  onCambiar: (nuevoValor: number) => void,
): HTMLInputElement {
  const input = document.createElement("input");

  input.type = "text";
  input.className = "popup-input popup-input-numero";
  input.value = String(valorActual);

  const confirmar = () => {
    onCambiar(interpretarLimite(input.value));
  };

  input.addEventListener("blur", confirmar);

  input.addEventListener("keydown", (evento) => {
    if (evento.key === "Enter") {
      input.blur();
    }
  });

  return input;
}

// ======================================================
// 📋🎛️ ABRIR POPUP EXTRA PORTAPAPELES
// ======================================================

export function abrirPopupExtraPortapapeles(
  evento: MouseEvent,
  contexto: ContextoFila,
  filaPerfil: FilaPerfil,
): void {
  const portapapelesExtra = filaPerfil.portapapelesExtra;

  const popup = document.createElement("div");

  popup.className = "popup-extra";

  const redibujar = () =>
    abrirPopupExtraPortapapeles(evento, contexto, filaPerfil);

  // ----------------------------------
  // COMPORTAMIENTO
  // ----------------------------------

  const comportamientoOpciones: {
    texto: string;
    valor: ComportamientoPortapapeles;
  }[] = [
    { texto: "Toggle", valor: "toggle" },
    { texto: "Efímero", valor: "efimero" },
  ];

  popup.append(
    crearFilaPopup(
      "Comportamiento",
      crearGrupoOpciones(
        comportamientoOpciones,
        portapapelesExtra.comportamiento,
        (valor) => {
          portapapelesExtra.comportamiento = valor;

          reconstruirFila(contexto.id);
          redibujar();
        },
      ),
    ),
  );

  // ----------------------------------
  // UBICACIÓN
  // ----------------------------------

  const ubicacionOpciones: { texto: string; valor: UbicacionPortapapeles }[] = [
    { texto: "Persistente", valor: "persistente" },
    { texto: "Cursor", valor: "cursor" },
  ];

  popup.append(
    crearFilaPopup(
      "Ubicación",
      crearGrupoOpciones(
        ubicacionOpciones,
        portapapelesExtra.ubicacion,
        (valor) => {
          portapapelesExtra.ubicacion = valor;

          reconstruirFila(contexto.id);
          redibujar();
        },
      ),
    ),
  );

  popup.append(crearSeparador());

  // ----------------------------------
  // TAMAÑO DE BOTONES
  // ----------------------------------

  const tamanoBotonOpciones: {
    texto: string;
    valor: TamanoBotonPortapapeles;
  }[] = [
    { texto: "Pequeño", valor: "pequeno" },
    { texto: "Mediano", valor: "mediano" },
    { texto: "Grande", valor: "grande" },
  ];

  popup.append(
    crearFilaPopup(
      "Tamaño de Botones",
      crearGrupoOpciones(
        tamanoBotonOpciones,
        portapapelesExtra.tamanoBoton,
        (valor) => {
          portapapelesExtra.tamanoBoton = valor;

          reconstruirFila(contexto.id);
          redibujar();
        },
      ),
    ),
  );

  // ----------------------------------
  // TAMAÑO DE TEXTO
  // ----------------------------------

  const tamanoTextoOpciones: {
    texto: string;
    valor: TamanoTextoPortapapeles;
  }[] = [
    { texto: "Pequeño", valor: "pequeno" },
    { texto: "Mediano", valor: "mediano" },
    { texto: "Grande", valor: "grande" },
  ];

  popup.append(
    crearFilaPopup(
      "Tamaño de Texto",
      crearGrupoOpciones(
        tamanoTextoOpciones,
        portapapelesExtra.tamanoTexto,
        (valor) => {
          portapapelesExtra.tamanoTexto = valor;

          reconstruirFila(contexto.id);
          redibujar();
        },
      ),
    ),
  );

  popup.append(crearSeparador());

  // ----------------------------------
  // LÍMITE DE ELEMENTOS
  // ----------------------------------

  popup.append(
    crearFilaPopup(
      "Límite de Elementos",
      crearCampoLimite(portapapelesExtra.limite, (nuevoValor) => {
        portapapelesExtra.limite = nuevoValor;

        reconstruirFila(contexto.id);
        redibujar();
      }),
    ),
  );

  mostrarPopup(popup, evento.clientX, evento.clientY);
}
