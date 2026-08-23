// ======================================================
// 📂🗂️ comp_Popup_Abrir_Con
// ------------------------------------------------------
// Sección "Abrir con..." del popup Extra del tipo "Abrir Archivo/App"
// (filaPerfil.tipo === "abrir"). Ya no es un listado siempre visible
// ni un popup aparte: es un botón más, misma fila que el resto del
// popup Extra (ver crearBotonAbrirCon), que muestra "Predeterminado"
// o el nombre del programa guardado. Al hacerle click se despliega
// el listado justo debajo, dentro del mismo popup Extra (ver
// crearListaAbrirCon, usada desde comp_popup_abrir_extra.ts). Elegir
// cualquier ítem del listado lo colapsa de nuevo y el botón pasa a
// mostrar la nueva selección.
//
// Fuente del listado: el registro de Windows —
// obtener_programas_abrir_con() (recientes de esa extensión primero,
// luego instalados, ver back_registro.rs) — más dos ítems fijos:
// "Predeterminado" arriba (limpia abrirCon → vuelve al programa que
// Windows tiene asociado por defecto para esa extensión) y
// "Examinar..." al final, que cae al selector manual ya existente
// (seleccionar_archivo filtrado a .exe) para cuando el programa
// deseado no aparece listado.
//
// El ícono de cada ítem NO viaja en la respuesta del listado — se
// pide aparte por ítem con obtener_icono_ruta() (mismo patrón que ya
// usa la columna App), para no bloquear el listado esperando todos
// los íconos antes de mostrarlo.
// ======================================================

import { invoke } from "@tauri-apps/api/core";

import { reconstruirFila } from "../ui/ui_tabla_control";

import type { ContextoFila } from "../core/core_contexto_fila";

import type { FilaPerfil } from "../core/core_perfil";

import { extensionDeRuta, nombreDeRuta } from "../core/core_abrir";

import { crearIndicadorActivo } from "./comp_popup_grupo";

// ======================================================
// 📦 MODELOS BACKEND
// ======================================================

interface IconoJson {
  ancho: number;

  alto: number;

  pixeles: string;
}

interface ProgramaJson {
  nombre: string;

  ruta: string;
}

// ======================================================
// 🔘 BOTÓN "ABRIR CON" (colapsado)
// ------------------------------------------------------
// Misma fila que Iniciar/Instancias en el popup Extra (ver
// crearFilaPopup en comp_popup_abrir_extra.ts) — muestra la
// selección actual, alternar() despliega/colapsa el listado.
// ======================================================

export function crearBotonAbrirCon(
  filaPerfil: FilaPerfil,
  alternar: () => void,
): HTMLButtonElement {
  const boton = document.createElement("button");

  boton.className = "ui-btn";

  const abrirCon = filaPerfil.abrirExtra.abrirCon;

  boton.textContent = abrirCon ? nombreDeRuta(abrirCon) : "Predeterminado";

  if (abrirCon) {
    boton.title = abrirCon;
  }

  boton.addEventListener("click", alternar);

  return boton;
}

// ======================================================
// 🎨 ÍCONO — FALLBACK Y REAL
// ------------------------------------------------------
// Mismo patrón de conversión base64 → canvas RGBA que ya usan
// comp_popup_app.ts / comp_popup_abrir_accion.ts.
// ======================================================

function crearIconoFallback(): HTMLElement {
  const icono = document.createElement("span");

  icono.className = "app-icono-fallback";

  icono.textContent = "▣";

  return icono;
}

function crearIcono(datos: IconoJson): HTMLElement {
  const canvas = document.createElement("canvas");

  canvas.width = datos.ancho;

  canvas.height = datos.alto;

  const contexto = canvas.getContext("2d");

  if (!contexto) {
    return crearIconoFallback();
  }

  const pixeles = Uint8ClampedArray.from(atob(datos.pixeles), (caracter) =>
    caracter.charCodeAt(0),
  );

  contexto.putImageData(new ImageData(pixeles, datos.ancho, datos.alto), 0, 0);

  canvas.className = "app-icono";

  return canvas;
}

// ======================================================
// 🔘 BOTÓN DE PROGRAMA
// ------------------------------------------------------
// Ícono en fallback hasta que resuelve obtener_icono_ruta() (mismo
// patrón asíncrono que crearAccionAbrir() en
// comp_popup_abrir_accion.ts) — no bloquea el listado esperando el
// ícono de cada ítem antes de mostrarlo. Muestra el indicador cyan
// (mismo patrón que crearGrupoOpciones) cuando es el programa
// guardado actualmente en abrirCon.
// ======================================================

function crearBotonPrograma(
  contexto: ContextoFila,
  filaPerfil: FilaPerfil,
  programa: ProgramaJson,
  activo: boolean,
  alModificar: () => void,
  alSeleccionar: () => void,
): HTMLButtonElement {
  const boton = document.createElement("button");

  boton.className = "ui-btn app-popup-programa";

  boton.dataset.activo = activo ? "true" : "false";

  if (activo) {
    boton.append(crearIndicadorActivo());
  }

  const icono = crearIconoFallback();

  boton.append(icono);

  const nombre = document.createElement("span");

  nombre.className = "app-popup-nombre";

  nombre.textContent = programa.nombre;

  boton.append(nombre);

  boton.title = programa.ruta;

  invoke<IconoJson | null>("obtener_icono_ruta", { ruta: programa.ruta })
    .then((iconoJson) => {
      if (!iconoJson) {
        return;
      }

      boton.replaceChild(crearIcono(iconoJson), icono);
    })
    .catch(() => {});

  boton.addEventListener("click", () => {
    filaPerfil.abrirExtra.abrirCon = programa.ruta;

    reconstruirFila(contexto.id);

    alModificar();

    alSeleccionar();
  });

  return boton;
}

// ======================================================
// ⭯ PREDETERMINADO (fija arriba del listado)
// ------------------------------------------------------
// Limpia abrirCon (vuelve a null): el archivo se vuelve a abrir con
// el programa que Windows tiene asociado por defecto para esa
// extensión, igual que antes de personalizar "Abrir con".
// ======================================================

function crearBotonPredeterminado(
  contexto: ContextoFila,
  filaPerfil: FilaPerfil,
  activo: boolean,
  alModificar: () => void,
  alSeleccionar: () => void,
): HTMLButtonElement {
  const boton = document.createElement("button");

  boton.className = "ui-btn app-popup-programa";

  boton.dataset.activo = activo ? "true" : "false";

  if (activo) {
    boton.append(crearIndicadorActivo());
  }

  const icono = document.createElement("span");

  icono.className = "app-popup-global-icono";

  icono.textContent = "⭯";

  boton.append(icono);

  const nombre = document.createElement("span");

  nombre.className = "app-popup-nombre";

  nombre.textContent = "Predeterminado";

  boton.append(nombre);

  boton.addEventListener("click", () => {
    filaPerfil.abrirExtra.abrirCon = null;

    reconstruirFila(contexto.id);

    alModificar();

    alSeleccionar();
  });

  return boton;
}

// ======================================================
// 🔍 EXAMINAR... (selector manual, fija al final del listado)
// ------------------------------------------------------
// Mismo comando que ya usaba el botón "Abrir con" antes de esta
// etapa (seleccionar_archivo filtrado a .exe) — queda como vía de
// escape para cuando el programa deseado no aparece en el listado
// del registro. Si se cancela el selector nativo, el listado queda
// desplegado tal cual estaba (no hay nada que colapsar).
// ======================================================

function crearBotonExaminar(
  contexto: ContextoFila,
  filaPerfil: FilaPerfil,
  alModificar: () => void,
  alSeleccionar: () => void,
): HTMLButtonElement {
  const boton = document.createElement("button");

  boton.className = "ui-btn app-popup-programa";

  const icono = document.createElement("span");

  icono.className = "app-popup-global-icono";

  icono.textContent = "📂";

  boton.append(icono);

  const nombre = document.createElement("span");

  nombre.className = "app-popup-nombre";

  nombre.textContent = "Examinar...";

  boton.append(nombre);

  boton.addEventListener("click", async () => {
    const ruta = await invoke<string | null>("seleccionar_archivo", {
      extensiones: ["exe"],
    });

    if (!ruta) {
      return;
    }

    filaPerfil.abrirExtra.abrirCon = ruta;

    reconstruirFila(contexto.id);

    alModificar();

    alSeleccionar();
  });

  return boton;
}

// ======================================================
// ➖ SEPARADOR (mismo estilo que el resto de los popups)
// ======================================================

function crearSeparador(): HTMLElement {
  const separador = document.createElement("div");

  separador.className = "app-popup-separador";

  return separador;
}

// ======================================================
// 📂🗂️ LISTADO DESPLEGABLE "ABRIR CON"
// ------------------------------------------------------
// Se muestra debajo del botón (ver crearBotonAbrirCon) solo cuando
// está desplegado — la ubicación y el estado expandido/colapsado los
// decide comp_popup_abrir_extra.ts, acá solo se arma el contenido.
//
// alSeleccionar la llama cualquier ítem elegido (Predeterminado, un
// programa, o Examinar con selección exitosa) — quien la pasa
// (comp_popup_abrir_extra.ts) es quien colapsa el listado y redibuja
// el popup Extra completo.
// ======================================================

export async function crearListaAbrirCon(
  contexto: ContextoFila,
  filaPerfil: FilaPerfil,
  alModificar: () => void,
  alSeleccionar: () => void,
): Promise<HTMLElement> {
  const extension = extensionDeRuta(filaPerfil.abrirAccion.ruta);

  const programas = await invoke<ProgramaJson[]>(
    "obtener_programas_abrir_con",
    { extension },
  );

  const abrirCon = filaPerfil.abrirExtra.abrirCon;

  const caja = document.createElement("div");

  caja.className = "popup-caja-interna app-popup-lista-caja";

  const lista = document.createElement("div");

  lista.className = "app-popup-lista";

  lista.append(
    crearBotonPredeterminado(
      contexto,
      filaPerfil,
      abrirCon === null,
      alModificar,
      alSeleccionar,
    ),
  );

  programas.forEach((programa) => {
    lista.append(
      crearBotonPrograma(
        contexto,
        filaPerfil,
        programa,
        programa.ruta === abrirCon,
        alModificar,
        alSeleccionar,
      ),
    );
  });

  caja.append(lista);

  const contenedor = document.createElement("div");

  contenedor.append(caja);

  contenedor.append(crearSeparador());

  contenedor.append(
    crearBotonExaminar(contexto, filaPerfil, alModificar, alSeleccionar),
  );

  return contenedor;
}
