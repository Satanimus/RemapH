// ======================================================
// ⚡🎛️ comp_Popup_Menu_Express_Extra
// ------------------------------------------------------
// Popup Extra de MenuExpress (filaPerfil.tipo === "menu_express"),
// abierto desde crearExtra() en comp_controles.ts. Mismo patrón
// persistente que el popup Extra de Tecla/Mouse (comp_popup_coordenada.ts)
// y de Multimedia (comp_popup_multimedia_extra.ts): elegir una opción
// actualiza filaPerfil.menuExtra y redibuja el mismo popup en el
// lugar, en vez de cerrarlo.
//
// Todo lo que no es un botón de grupo es texto "subtítulo" — mismo
// estilo tenue (popup-fila-label) que usan las etiquetas del popup
// de Coordenada.
//
// Secciones (ver spec):
//   FORMA               → menuExtra.forma
//   LIMITAR CUADRÍCULA  → menuExtra.columnas / menuExtra.filas
//     (solo visible si forma === "cuadricula")
//   COMPORTAMIENTO      → menuExtra.comportamiento
//   UBICACIÓN           → menuExtra.ubicacion
//   BOTONES             → menuExtra.tamanoBoton + menuExtra.colorBoton
//     (misma fila — antes "Tamaño de Botones" y "Color Botón" separados)
//   TEXTO               → menuExtra.tamanoTexto (antes "Tamaño de Texto")
// ======================================================

import { mostrarPopup } from "./comp_popup_contenedor";

import { reconstruirFila } from "../ui/ui_tabla_control";

import type { ContextoFila } from "../core/core_contexto_fila";

import type { FilaPerfil } from "../core/core_perfil";

import type {
  FormaMenu,
  ComportamientoMenu,
  UbicacionMenu,
  TamanoMenu,
  ColorBotonMenu,
} from "../core/core_menu_express";

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
// 🔘🎨 FILA "BOTONES" (Tamaño + Color, misma línea)
// ------------------------------------------------------
// Fusiona lo que antes eran dos filas separadas ("Tamaño de
// Botones" y "Color Botón") en una sola — el prefijo "Tamaño
// de" se omite acá, mismo criterio que la fila "Texto" de
// abajo (antes "Tamaño de Texto").
// ======================================================

function crearFilaBotones(
  contexto: ContextoFila,
  filaPerfil: FilaPerfil,
  alModificar: () => void,
  redibujar: () => void,
  tamanoOpciones: { texto: string; valor: TamanoMenu }[],
  colorBotonOpciones: { texto: string; valor: ColorBotonMenu }[],
): HTMLElement {
  const menuExtra = filaPerfil.menuExtra;

  const grupo = document.createElement("div");

  grupo.className = "popup-grupo-doble";

  grupo.append(
    crearGrupoOpciones(tamanoOpciones, menuExtra.tamanoBoton, (valor) => {
      menuExtra.tamanoBoton = valor;

      reconstruirFila(contexto.id);
      alModificar();
      redibujar();
    }),

    crearGrupoOpciones(colorBotonOpciones, menuExtra.colorBoton, (valor) => {
      menuExtra.colorBoton = valor;

      reconstruirFila(contexto.id);
      alModificar();
      redibujar();
    }),
  );

  return crearFilaPopup("Botones", grupo);
}

// ======================================================
// 🔢 FILA "LIMITAR CUADRÍCULA" (Columnas / Filas)
// ------------------------------------------------------
// Ambos campos son texto libre — se interpreta al perder foco o
// presionar Enter. Reglas (ya acordadas):
//   • Solo uno de los dos puede ser distinto de 0 a la vez — el
//     que tiene 0 es el flexible ("Auto"), se acomoda al número
//     de atajos.
//   • Editar un campo con un valor válido (entero > 0) lo vuelve
//     el limitado, y pone el OTRO en 0 automáticamente.
//   • Si lo escrito es 0 o inválido, se toma como 1 (no se puede
//     dejar en 0 el campo que se está editando — 0 ya lo tiene
//     asegurado el otro campo).
// ======================================================

function interpretarNumero(texto: string): number {
  const valor = parseInt(texto, 10);

  return Number.isFinite(valor) && valor > 0 ? valor : 1;
}

function crearCampoNumero(
  valor: number,
  onCambiar: (nuevoValor: number) => void,
): HTMLInputElement {
  const input = document.createElement("input");

  input.type = "text";
  input.className = "popup-input popup-input-numero";
  input.value = valor === 0 ? "Auto" : String(valor);

  const confirmar = () => {
    if (input.value.trim().toLowerCase() === "auto") {
      return;
    }

    onCambiar(interpretarNumero(input.value));
  };

  input.addEventListener("blur", confirmar);

  input.addEventListener("keydown", (evento) => {
    if (evento.key === "Enter") {
      input.blur();
    }
  });

  return input;
}

function crearFilaLimitarCuadricula(
  contexto: ContextoFila,
  filaPerfil: FilaPerfil,
  alModificar: () => void,
  redibujar: () => void,
): HTMLElement {
  const menuExtra = filaPerfil.menuExtra;

  const contenedor = document.createElement("div");

  contenedor.className = "popup-fila";

  const etiqueta = document.createElement("span");

  etiqueta.className = "popup-fila-label";
  etiqueta.textContent = "Limitar Cuadrícula";

  contenedor.append(etiqueta);

  const grilla = document.createElement("div");

  grilla.className = "popup-grupo";

  // ------- Columnas -------
  const columnasContenedor = document.createElement("div");

  columnasContenedor.className = "popup-numero-campo";

  const columnasLabel = document.createElement("span");

  columnasLabel.textContent = "Columnas:";

  const columnasInput = crearCampoNumero(menuExtra.columnas, (nuevoValor) => {
    menuExtra.columnas = nuevoValor;
    menuExtra.filas = 0;

    reconstruirFila(contexto.id);
    alModificar();
    redibujar();
  });

  columnasContenedor.append(columnasLabel, columnasInput);

  // ------- Filas -------
  const filasContenedor = document.createElement("div");

  filasContenedor.className = "popup-numero-campo";

  const filasLabel = document.createElement("span");

  filasLabel.textContent = "Filas:";

  const filasInput = crearCampoNumero(menuExtra.filas, (nuevoValor) => {
    menuExtra.filas = nuevoValor;
    menuExtra.columnas = 0;

    reconstruirFila(contexto.id);
    alModificar();
    redibujar();
  });

  filasContenedor.append(filasLabel, filasInput);

  grilla.append(columnasContenedor, filasContenedor);

  contenedor.append(grilla);

  return contenedor;
}

// ======================================================
// ⚡🎛️ ABRIR POPUP EXTRA MENUEXPRESS
// ======================================================

export function abrirPopupExtraMenuExpress(
  evento: MouseEvent,
  contexto: ContextoFila,
  filaPerfil: FilaPerfil,
  alModificar: () => void,
): void {
  const menuExtra = filaPerfil.menuExtra;

  const popup = document.createElement("div");

  popup.className = "popup-extra";

  const redibujar = () =>
    abrirPopupExtraMenuExpress(evento, contexto, filaPerfil, alModificar);

  // ----------------------------------
  // FORMA
  // ----------------------------------

  const formaOpciones: { texto: string; valor: FormaMenu }[] = [
    { texto: "Radial", valor: "radial" },
    { texto: "Cuadrícula", valor: "cuadricula" },
  ];

  popup.append(
    crearFilaPopup(
      "Forma",
      crearGrupoOpciones(formaOpciones, menuExtra.forma, (valor) => {
        menuExtra.forma = valor;

        reconstruirFila(contexto.id);
        alModificar();
        redibujar();
      }),
    ),
  );

  // ----------------------------------
  // LIMITAR CUADRÍCULA (solo si forma === "cuadricula")
  // ----------------------------------

  if (menuExtra.forma === "cuadricula") {
    popup.append(crearSeparador());

    popup.append(
      crearFilaLimitarCuadricula(contexto, filaPerfil, alModificar, redibujar),
    );
  }

  popup.append(crearSeparador());

  // ----------------------------------
  // COMPORTAMIENTO
  // ----------------------------------

  const comportamientoOpciones: { texto: string; valor: ComportamientoMenu }[] =
    [
      { texto: "Toggle", valor: "toggle" },
      { texto: "Efímero", valor: "efimero" },
    ];

  popup.append(
    crearFilaPopup(
      "Comportamiento",
      crearGrupoOpciones(
        comportamientoOpciones,
        menuExtra.comportamiento,
        (valor) => {
          menuExtra.comportamiento = valor;

          reconstruirFila(contexto.id);
          alModificar();
          redibujar();
        },
      ),
    ),
  );

  // ----------------------------------
  // UBICACIÓN
  // ----------------------------------

  const ubicacionOpciones: { texto: string; valor: UbicacionMenu }[] = [
    { texto: "Persistente", valor: "persistente" },
    { texto: "Cursor", valor: "cursor" },
  ];

  popup.append(
    crearFilaPopup(
      "Ubicación",
      crearGrupoOpciones(ubicacionOpciones, menuExtra.ubicacion, (valor) => {
        menuExtra.ubicacion = valor;

        reconstruirFila(contexto.id);
        alModificar();
        redibujar();
      }),
    ),
  );

  popup.append(crearSeparador());

  // ----------------------------------
  // BOTONES (Tamaño + Color, misma línea)
  // ----------------------------------

  const tamanoOpciones: { texto: string; valor: TamanoMenu }[] = [
    { texto: "Pequeño", valor: "pequeno" },
    { texto: "Mediano", valor: "mediano" },
    { texto: "Grande", valor: "grande" },
  ];

  // Monocromo (default): los botones heredan el color de fondo de
  // la ventana (color de la fila MenuExpress, sin cambios acá).
  // Color: cada botón toma el borde del color de SU PROPIA fila
  // referenciada — resuelto del lado de Rust (compilador.rs), esta
  // fila del popup solo elige el modo.
  const colorBotonOpciones: { texto: string; valor: ColorBotonMenu }[] = [
    { texto: "Color", valor: "color" },
    { texto: "Monocromo", valor: "monocromo" },
  ];

  popup.append(
    crearFilaBotones(
      contexto,
      filaPerfil,
      alModificar,
      redibujar,
      tamanoOpciones,
      colorBotonOpciones,
    ),
  );

  // ----------------------------------
  // TEXTO (antes "Tamaño de Texto")
  // ----------------------------------

  popup.append(
    crearFilaPopup(
      "Texto",
      crearGrupoOpciones(tamanoOpciones, menuExtra.tamanoTexto, (valor) => {
        menuExtra.tamanoTexto = valor;

        reconstruirFila(contexto.id);
        alModificar();
        redibujar();
      }),
    ),
  );

  mostrarPopup(popup, evento.clientX, evento.clientY);
}
