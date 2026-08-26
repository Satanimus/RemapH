// ======================================================
// 🎨 vent_configuracion_Apariencia
// ------------------------------------------------------
// Tabla en árbol de la pestaña Apariencia (Nombre / Valor
// por Defecto / Editar / Valor Personalizado), separada de
// crearPestanaEditable() porque el catálogo ya no es una
// lista plana de filas sino un árbol Título/Subtítulo/
// Elemento con valores anidados (ver configuracion_listar_apariencia
// en comandos.rs).
//
// Placeholder de esqueleto: monta el árbol en la tabla nueva,
// pero hayEdicionesPendientes/validarYRecolectar/aplicarGuardado/
// marcarErroresGuardado/restablecerPestana todavía no tienen
// lógica real (llega en una etapa posterior de persistencia).
// ======================================================

import { invoke } from "@tauri-apps/api/core";

import type {
  CambioConfiguracion,
  ErrorConfiguracion,
  Pestana,
  ResultadoGuardado,
} from "./vent_configuracion_main";

// Modelo tal cual lo entrega configuracion_listar_apariencia
// (snake_case) — tipo/valor_defecto son null en filas de nivel 1/2/3.
interface FilaCssCruda {
  id: string;
  nivel: number;
  nombre_ui: string;
  tipo: string | null;
  valor_defecto: string | null;
  valor_personalizado: string | null;
}

// Fila de nivel 1/2/3 con sus valores de nivel 0 (hijos) ya agrupados.
interface NodoArbol {
  entrada: FilaCssCruda;
  hijos: FilaCssCruda[];
}

// ----------------------------------------------------
// Construcción del árbol
// ----------------------------------------------------

function construirArbol(filas: FilaCssCruda[]): NodoArbol[] {
  const nodos: NodoArbol[] = [];

  let actual: NodoArbol | null = null;

  for (const fila of filas) {
    if (fila.nivel === 0) {
      actual?.hijos.push(fila);
      continue;
    }

    actual = { entrada: fila, hijos: [] };
    nodos.push(actual);
  }

  return nodos;
}

// ----------------------------------------------------
// Montar una fila del árbol
// ----------------------------------------------------

function crearFilaArbol(nodo: NodoArbol): HTMLTableRowElement {
  const tr = document.createElement("tr");

  const tdNombre = document.createElement("td");
  tdNombre.textContent = nodo.entrada.nombre_ui;

  const tdDefecto = document.createElement("td");
  tdDefecto.textContent = nodo.hijos
    .map((hijo) => hijo.valor_defecto ?? "")
    .filter((valor) => valor.length > 0)
    .join(" ; ");

  const tdEditar = document.createElement("td");
  tdEditar.textContent = "✎";

  const tdPersonalizado = document.createElement("td");

  tr.append(tdNombre, tdDefecto, tdEditar, tdPersonalizado);

  return tr;
}

// ----------------------------------------------------
// Armar la tabla (thead de 4 columnas + tbody + scroll)
// ----------------------------------------------------

function crearTablaApariencia(panel: HTMLDivElement): HTMLTableSectionElement {
  const tabla = document.createElement("table");
  tabla.className = "configuracion-tabla-arbol";

  const thead = document.createElement("thead");
  const trEncabezado = document.createElement("tr");

  for (const texto of [
    "Nombre",
    "Valor por Defecto",
    "Editar",
    "Valor Personalizado",
  ]) {
    const th = document.createElement("th");
    th.textContent = texto;
    trEncabezado.append(th);
  }

  thead.append(trEncabezado);

  const tbody = document.createElement("tbody");

  tabla.append(thead, tbody);

  const scrollTabla = document.createElement("div");
  scrollTabla.className = "configuracion-tabla-scroll";
  scrollTabla.append(tabla);

  panel.append(scrollTabla);

  return tbody;
}

// ----------------------------------------------------
// Pestaña completa
// ----------------------------------------------------

export function crearPestanaApariencia(
  panel: HTMLDivElement,
  despuesDeAplicar: () => Promise<void>,
): Pestana {
  const tbody = crearTablaApariencia(panel);

  async function cargar(): Promise<void> {
    const filas = await invoke<FilaCssCruda[]>("configuracion_listar_apariencia");

    const arbol = construirArbol(filas);

    tbody.innerHTML = "";

    for (const nodo of arbol) {
      tbody.append(crearFilaArbol(nodo));
    }
  }

  function hayEdicionesPendientes(): boolean {
    return false;
  }

  function validarYRecolectar(): {
    cambios: CambioConfiguracion[];
    erroresLocales: string[];
  } {
    return { cambios: [], erroresLocales: [] };
  }

  async function aplicarGuardado(
    _cambios: CambioConfiguracion[],
  ): Promise<ResultadoGuardado> {
    return { errores: [] };
  }

  function marcarErroresGuardado(_errores: ErrorConfiguracion[]): void {}

  async function limpiarEstadoTrasGuardado(): Promise<void> {
    await despuesDeAplicar();
  }

  async function restablecerPestana(): Promise<void> {
    await cargar();
    await despuesDeAplicar();
  }

  return {
    cargar,
    hayEdicionesPendientes,
    validarYRecolectar,
    aplicarGuardado,
    marcarErroresGuardado,
    limpiarEstadoTrasGuardado,
    restablecerPestana,

    textoConfirmacionRestablecer:
      "¿Restablecer todos los valores de Apariencia a los de fábrica? " +
      "Se pierden los valores personalizados de esta pestaña.",
  };
}
