// ======================================================
// ⁝⁝ util_Arrastrable
// ------------------------------------------------------
// Componente GENÉRICO de reordenamiento de filas por
// arrastre (ratón) o teclado (flechas), con selección
// múltiple. No conoce el contenido de cada fila — solo
// recibe una lista de ids + los elementos DOM ya creados
// por el llamador (comp_popup_macro_editor.ts en la Etapa
// 5, y más adelante la tabla principal en la Etapa 9).
//
// Especificación completa: "Plan Arrastre en tabla
// principal" (documento del usuario). Resumen del
// comportamiento implementado acá:
//
// 1. Clic MANTENIDO sobre el botón ⟫ (asa) durante
//    config::tiempo_mantenido() → activa el modo "Mover" y
//    selecciona esa fila (borde cyan + indicador ⁝⁝ junto
//    al botón). Un clic CORTO no se toca acá — lo maneja el
//    llamador con su propio listener "click" (abre menú).
// 2. Ctrl + clic sobre el fondo de una fila → agrega/saca
//    esa fila de la selección. La decisión (agregar vs
//    sacar) se toma según el estado ANTES del clic, para
//    que un mismo clic nunca seleccione y deseleccione a la
//    vez.
// 3. Con selección activa, ↑ / ↓ mueven todo el grupo un
//    lugar, conservando el orden relativo original. En el
//    borde no pasa nada (todo el grupo, no solo el ítem de
//    punta).
// 4. Arrastre con ratón: fantasma semitransparente +
//    placeholder (1/3 del alto de fila) que indica dónde se
//    insertará el grupo. Al soltar, animación de 150ms
//    (var(--speed)).
// 5. El modo "Mover" se apaga solo o) al hacer clic fuera de
//    cualquier fila registrada (lo detecta este módulo), o
//    b) cuando el llamador invoca salirModoMover() a mano —
//    típicamente desde "Guardar" o desde el click de
//    cualquier OTRO botón de columna, que este componente no
//    puede conocer por ser genérico.
//
// Nota de diseño (no explícita en el documento original):
// si la selección no es contigua y el ítem de un extremo ya
// está pegado al borde, se interpretó que TODO el grupo se
// bloquea (no solo ese ítem) — es la lectura más consistente
// con "el grupo se mueve junto".
// ======================================================

import { invoke } from "@tauri-apps/api/core";

// ======================================================
// ⏱️ TIEMPO DE MANTENIDO (config::tiempo_mantenido())
// ------------------------------------------------------
// Se pide una sola vez al backend y se cachea en memoria.
// Mientras no llega la respuesta (o si falla), se usa un
// valor de reserva razonable — nunca bloquea la UI.
// ======================================================

const TIEMPO_MANTENIDO_RESERVA_MS = 300;

let tiempoMantenidoCacheado: number | null = null;
let tiempoMantenidoPromesa: Promise<number> | null = null;

function precargarTiempoMantenido(): void {
  if (tiempoMantenidoCacheado !== null || tiempoMantenidoPromesa) return;

  tiempoMantenidoPromesa = invoke<number>("obtener_tiempo_mantenido")
    .then((valor) => {
      tiempoMantenidoCacheado = valor;

      return valor;
    })
    .catch(() => {
      tiempoMantenidoCacheado = TIEMPO_MANTENIDO_RESERVA_MS;

      return TIEMPO_MANTENIDO_RESERVA_MS;
    });
}

function tiempoMantenidoActualMs(): number {
  return tiempoMantenidoCacheado ?? TIEMPO_MANTENIDO_RESERVA_MS;
}

// ======================================================
// 🎛️ CLASES CSS (ver src/styles/styl_arrastrable.css)
// ======================================================

const CLASE_FILA_SELECCIONADA = "arr-fila--seleccionada";
const CLASE_ASA_ACTIVA = "arr-asa--activa";
const CLASE_FILA_OCULTA = "arr-fila--oculta-arrastre";
const CLASE_CONTENEDOR_ARRASTRANDO = "arr-contenedor--arrastrando";
const CLASE_FANTASMA = "arr-fantasma";
const CLASE_FANTASMA_FILA = "arr-fantasma-fila";
const CLASE_FANTASMA_CONTADOR = "arr-fantasma-contador";
const CLASE_PLACEHOLDER = "arr-placeholder";

// Umbral de movimiento (px) para distinguir "clic" de
// "arrastre" — tanto para cancelar un mantenido que en
// realidad era un manotazo, como para decidir cuándo un
// mousedown sobre una fila ya seleccionada se convierte en
// arrastre en vez de quedar en un clic simple.
const UMBRAL_ARRASTRE_PX = 5;

// Bug (Macro): el placeholder usa el alto real de la fila
// arrastrada (rectPrimero.height más abajo), que para un paso
// colapsado del editor de Macro es bastante más bajo que
// --row-height (filas compactas de una sola línea) — quedaba
// "muy bajito" y costaba identificar dónde iba a quedar. Nunca
// baja de este mínimo (mismo alto de fila que ya usa la tabla
// principal), sin tocar el alto real cuando la fila arrastrada
// ya es más alta (paso expandido, fila de la tabla, etc.).
function alturaMinimaPlaceholder(): number {
  const valor = getComputedStyle(document.documentElement).getPropertyValue(
    "--row-height",
  );

  const numero = parseFloat(valor);

  return Number.isFinite(numero) ? numero : 40;
}

// Duración de la animación de reordenamiento por teclado —
// más rápida que la de soltar con ratón (var(--speed), ver
// styl_variables.css), y sin placeholder (spec, sección 3).
const DURACION_ANIMACION_TECLADO_MS = 90;

function duracionAnimacionArrastreMs(contenedor: HTMLElement): number {
  const crudo = getComputedStyle(contenedor).getPropertyValue("--speed").trim();

  const numero = parseFloat(crudo);

  if (Number.isNaN(numero)) return 150;

  // --speed puede venir en "s" (ej. ".15s") o sin unidad.
  return crudo.endsWith("ms") ? numero : numero * 1000;
}

// ======================================================
// 📦 TIPOS PÚBLICOS
// ======================================================

export interface OpcionesArrastrable {
  contenedor: HTMLElement;

  obtenerOrdenIds: () => string[];

  onReordenar: (nuevoOrden: string[]) => void;

  onSalirModoMover?: () => void;

  // Etapa D: notifica cualquier cambio de la selección que NO pasa
  // por onReordenar (Shift/Ctrl+clic sin arrastre, y salirModoMover)
  // — el editor de Macros lo usa para redibujar el popup entero y
  // así reflejar de inmediato los botones de lote de la cabecera.
  onSeleccionCambio?: () => void;

  // Etapa (fix): elementos que NO deben tratarse como "click afuera"
  // aunque no sean una fila registrada — ej. la cabecera de un editor
  // que agrega sus propios botones de acción sobre la selección
  // (Duplicar/Eliminar seleccionadas). Sin esto, cualquier pointerdown
  // sobre esos botones dispara salirModoMover() en fase de captura
  // ANTES de que el click del botón llegue a ejecutarse, vaciando la
  // selección antes de que la acción pueda leerla.
  elementosExentosClickAfuera?: HTMLElement[];

  obtenerIdsSeparadores?: () => string[];

  // Regla 11: consulta si `id` es un separador contraído. El
  // llamador (ver ui_tabla.ts) también aprovecha esta consulta
  // para expandirlo automáticamente (muta el modelo y reconstruye
  // la tabla) antes de que el controlador de arrastre calcule
  // posiciones — así las filas que quedaban ocultas por el
  // colapso ya existen en el DOM al iniciar el gesto.
  esSeparadorContraido?: (id: string) => boolean;
}

export interface ControladorArrastre {
  registrarFila(
    id: string,
    filaElemento: HTMLElement,
    botonAsa: HTMLElement,
  ): void;

  estaSeleccionada(id: string): boolean;

  // Devuelve una copia del Set de ids actualmente seleccionados.
  // Usado por el editor de Macros para capturar la selección completa
  // antes de un redibujo (que destruye y recrea el controlador).
  obtenerSeleccionadas(): string[];

  salirModoMover(): void;

  // Activa el modo Mover para una fila puntual sin pasar por
  // el clic mantenido sobre el asa — usado por el ítem "Mover"
  // del popup de Opciones (ver comp_popup_abrir.ts). Selecciona
  // solo esa fila (reemplaza cualquier selección previa), igual
  // que el mantenido cuando no hay Ctrl. No inicia un arrastre
  // por sí sola: a partir de acá el usuario arrastra con el
  // mouse desde el fondo de la fila, o mueve con las flechas.
  activarModoMoverPara(id: string): void;

  // Agrega una fila a la selección existente sin reemplazarla —
  // usado por el editor de Macros para restaurar una selección
  // múltiple tras recrear el controlador en cada redibujo.
  seleccionarAdicional(id: string): void;

  // Ancla de Shift+click (última fila de referencia para calcular el
  // rango) — igual que la selección, vive en el closure de esta
  // instancia y se pierde al destruir/recrear el controlador. El
  // editor de Macros la lee antes de destruir y la reestablece en la
  // instancia nueva junto con seleccionarAdicional, para que un
  // Shift+click después de un redibujo (ej. tras la primera fila
  // seleccionada, que dispara onSeleccionCambio → redibujar) siga
  // extendiendo el rango en vez de reemplazar la selección.
  obtenerAncla(): string | null;

  establecerAncla(id: string | null): void;

  // No pedido en la interfaz original — agregado porque este
  // controlador engancha listeners en `document` (clic afuera,
  // flechas) que viven mientras exista el controlador. Un
  // popup que se abre/cierra muchas veces (ej. el editor de
  // Macro, Etapa 5) debe llamar esto al cerrarse para no
  // acumular listeners húerfanos de instancias anteriores.
  destruir(): void;
}

// ======================================================
// 🏗️ FÁBRICA
// ======================================================

export function crearControladorArrastre(
  opciones: OpcionesArrastrable,
): ControladorArrastre {
  precargarTiempoMantenido();

  const {
    contenedor,
    obtenerOrdenIds,
    onReordenar,
    onSalirModoMover,
    onSeleccionCambio,
    elementosExentosClickAfuera,
    obtenerIdsSeparadores,
    esSeparadorContraido,
  } = opciones;

  // id → elementos registrados por el llamador. Se puede
  // volver a llamar registrarFila con el mismo id (ej. tras
  // un re-render) — simplemente reemplaza la entrada.
  const filas = new Map<string, { elemento: HTMLElement; asa: HTMLElement }>();

  // Elemento fila → id, para poder recorrer contenedor.children
  // durante un arrastre sin depender de que el llamador haya
  // puesto un dataset propio.
  const idPorElemento = new WeakMap<HTMLElement, string>();

  const seleccionadas = new Set<string>();

  // Último id usado como referencia de una selección simple o
  // Ctrl+click (no Shift) — punto de partida para extender el
  // rango con el próximo Shift+click, estilo selector de archivos.
  let anclaSeleccion: string | null = null;

  // ------------------------------------------------------
  // Estado de una "presión" en curso sobre el botón ⟫
  // (desde mousedown hasta que se resuelve: cancelada, clic
  // corto normal, o convertida en modo Mover / arrastre).
  // ------------------------------------------------------
  interface PresionAsa {
    id: string;
    timerId: ReturnType<typeof setTimeout>;
    inicioX: number;
    inicioY: number;
    cancelada: boolean;
    convertidaEnMover: boolean;
  }

  let presionAsaActual: PresionAsa | null = null;

  // ------------------------------------------------------
  // Estado de un posible arrastre iniciado desde el fondo de
  // una fila (background), a diferencia del asa: acá no hay
  // espera de mantenido, la fila ya tiene que estar
  // seleccionada (o agregarse con Ctrl) para poder arrastrar.
  // ------------------------------------------------------
  interface PresionFila {
    id: string;
    inicioX: number;
    inicioY: number;
  }

  let presionFilaActual: PresionFila | null = null;

  // ------------------------------------------------------
  // Estado del arrastre visual en curso (fantasma +
  // placeholder), si lo hay.
  // ------------------------------------------------------
  interface EstadoArrastre {
    idsGrupo: string[];
    fantasma: HTMLElement;
    contador: HTMLElement | null;
    placeholder: HTMLElement;
    offsetX: number;
    offsetY: number;
    seMovio: boolean;
  }

  let arrastreActual: EstadoArrastre | null = null;

  // ======================================================
  // 🎨 FEEDBACK VISUAL DE SELECCIÓN
  // ======================================================

  function refrescarClaseFila(id: string): void {
    const registro = filas.get(id);

    if (!registro) return;

    const activo = seleccionadas.has(id);

    registro.elemento.classList.toggle(CLASE_FILA_SELECCIONADA, activo);

    registro.asa.classList.toggle(CLASE_ASA_ACTIVA, activo);
  }

  function seleccionar(id: string): void {
    if (seleccionadas.has(id)) return;

    seleccionadas.add(id);

    refrescarClaseFila(id);
  }

  function deseleccionar(id: string): void {
    if (!seleccionadas.has(id)) return;

    seleccionadas.delete(id);

    refrescarClaseFila(id);
  }

  function reemplazarSeleccionPor(id: string): void {
    const anteriores = [...seleccionadas];

    seleccionadas.clear();

    seleccionadas.add(id);

    anteriores.forEach(refrescarClaseFila);

    refrescarClaseFila(id);
  }

  // Selecciona todo el tramo entre idDesde e idHasta (inclusive),
  // en el orden actual de la tabla — Shift+click estilo selector
  // de archivos. No limpia la selección previa: suma al tramo.
  function seleccionarRango(idDesde: string, idHasta: string): void {
    const orden = obtenerOrdenIds();

    const indiceDesde = orden.indexOf(idDesde);

    const indiceHasta = orden.indexOf(idHasta);

    if (indiceDesde === -1 || indiceHasta === -1) return;

    const inicio = Math.min(indiceDesde, indiceHasta);

    const fin = Math.max(indiceDesde, indiceHasta);

    for (let i = inicio; i <= fin; i++) {
      seleccionar(orden[i]);
    }
  }

  // ======================================================
  // 🚪 SALIR DEL MODO MOVER
  // ======================================================

  function salirModoMover(): void {
    if (seleccionadas.size === 0) return;

    const idsAfectados = [...seleccionadas];

    seleccionadas.clear();

    anclaSeleccion = null;

    idsAfectados.forEach(refrescarClaseFila);

    onSalirModoMover?.();
    onSeleccionCambio?.();
  }

  // ======================================================
  // 🧹 LIMPIEZA DE FILAS QUE YA NO EXISTEN
  // ------------------------------------------------------
  // Se corre antes de cualquier operación de orden, por si el
  // llamador dejó de registrar filas eliminadas sin avisar.
  // ======================================================

  function limpiarFilasFantasma(ordenVigente: string[]): void {
    const vigentes = new Set(ordenVigente);

    for (const id of filas.keys()) {
      if (!vigentes.has(id)) filas.delete(id);
    }

    for (const id of [...seleccionadas]) {
      if (!vigentes.has(id)) deseleccionar(id);
    }
  }

  // ======================================================
  // 🔼🔽 MOVIMIENTO CON FLECHAS (sección 3 del plan)
  // ======================================================

  function moverGrupoConFlecha(direccion: "arriba" | "abajo"): void {
    const orden = obtenerOrdenIds();

    limpiarFilasFantasma(orden);

    if (seleccionadas.size === 0) return;

    const arr = [...orden];

    const esSeleccionado = (id: string) => seleccionadas.has(id);

    if (direccion === "arriba") {
      const primerIndiceSeleccionado = arr.findIndex(esSeleccionado);

      if (primerIndiceSeleccionado <= 0) return; // ya en el borde: nada.

      for (let i = 1; i < arr.length; i++) {
        if (esSeleccionado(arr[i])) {
          [arr[i - 1], arr[i]] = [arr[i], arr[i - 1]];
        }
      }
    } else {
      let ultimoIndiceSeleccionado = -1;

      arr.forEach((id, i) => {
        if (esSeleccionado(id)) ultimoIndiceSeleccionado = i;
      });

      if (
        ultimoIndiceSeleccionado === -1 ||
        ultimoIndiceSeleccionado >= arr.length - 1
      ) {
        return; // ya en el borde: nada.
      }

      for (let i = arr.length - 2; i >= 0; i--) {
        if (esSeleccionado(arr[i])) {
          [arr[i], arr[i + 1]] = [arr[i + 1], arr[i]];
        }
      }
    }

    aplicarNuevoOrden(arr, DURACION_ANIMACION_TECLADO_MS);
  }

  function manejarTeclado(evento: KeyboardEvent): void {
    if (seleccionadas.size === 0) return;

    // Esc sale del modo Mover (Regla 1/2 de reglas_esc.txt) — mismo
    // camino que salir por click afuera o por Guardar: solo apaga
    // selección/modo, no revierte ningún reordenamiento ya aplicado.
    // stopPropagation: esta es la capa más externa (Regla 9) — un
    // solo Esc no debe seguir burbujeando hacia otros listeners
    // (popups, etc.) que puedan sumarse en etapas siguientes.
    if (evento.key === "Escape") {
      evento.stopPropagation();

      salirModoMover();

      return;
    }

    if (evento.key !== "ArrowUp" && evento.key !== "ArrowDown") return;

    const objetivo = evento.target as HTMLElement | null;

    const enCampoDeTexto =
      objetivo &&
      (objetivo.tagName === "INPUT" ||
        objetivo.tagName === "TEXTAREA" ||
        objetivo.isContentEditable);

    if (enCampoDeTexto) return;

    evento.preventDefault();

    moverGrupoConFlecha(evento.key === "ArrowUp" ? "arriba" : "abajo");
  }

  // ======================================================
  // 🔁 APLICAR NUEVO ORDEN AL DOM (con animación FLIP)
  // ------------------------------------------------------
  // Reordena contenedor.children según `nuevoOrden`, avisa al
  // llamador (onReordenar) y anima el desplazamiento de las
  // filas que cambiaron de posición. rectosOrigenPorId permite
  // pasar posiciones "de partida" alternativas (ej. la del
  // placeholder para las filas recién soltadas).
  // ======================================================

  function aplicarNuevoOrden(
    nuevoOrden: string[],
    duracionMs: number,
    rectosOrigenPorId?: Map<string, DOMRect>,
  ): void {
    const rectosAntes = new Map<string, DOMRect>();

    filas.forEach((registro, id) => {
      rectosAntes.set(id, registro.elemento.getBoundingClientRect());
    });

    nuevoOrden.forEach((id) => {
      const registro = filas.get(id);

      if (registro) contenedor.appendChild(registro.elemento);
    });

    onReordenar(nuevoOrden);

    nuevoOrden.forEach((id) => {
      const registro = filas.get(id);

      if (!registro) return;

      const origen = rectosOrigenPorId?.get(id) ?? rectosAntes.get(id);

      if (!origen) return;

      const destino = registro.elemento.getBoundingClientRect();

      const deltaX = origen.left - destino.left;
      const deltaY = origen.top - destino.top;

      if (deltaX === 0 && deltaY === 0) return;

      const el = registro.elemento;

      el.style.transition = "none";
      el.style.transform = `translate(${deltaX}px, ${deltaY}px)`;

      requestAnimationFrame(() => {
        el.style.transition = `transform ${duracionMs}ms ease`;
        el.style.transform = "";
      });

      setTimeout(() => {
        el.style.transition = "";
        el.style.transform = "";
      }, duracionMs + 30);
    });
  }

  // ======================================================
  // 🖱️ ARRASTRE CON RATÓN (sección 4 del plan)
  // ======================================================

  function crearFantasma(idsGrupo: string[], primerRect: DOMRect): HTMLElement {
    const fantasma = document.createElement("div");

    fantasma.className = CLASE_FANTASMA;

    fantasma.style.width = `${primerRect.width}px`;

    const primeraFila = filas.get(idsGrupo[0])?.elemento;

    if (primeraFila) {
      const clon = primeraFila.cloneNode(true) as HTMLElement;

      clon.classList.add(CLASE_FANTASMA_FILA);

      clon.style.width = `${primerRect.width}px`;
      clon.style.height = `${primerRect.height}px`;

      fantasma.appendChild(clon);
    }

    document.body.appendChild(fantasma);

    return fantasma;
  }

  // Contador "+N" de filas extra en el grupo arrastrado — elemento
  // propio (no hijo de .arr-fantasma, que recorta con overflow:
  // hidden) para poder anclarlo directo a la posición del cursor,
  // no a una esquina fija del fantasma (spec: "que se dibuje
  // completo y siga al mouse").
  function crearContador(cantidadFilas: number): HTMLElement {
    const contador = document.createElement("div");

    contador.className = CLASE_FANTASMA_CONTADOR;

    contador.textContent = `+${cantidadFilas}`;

    document.body.appendChild(contador);

    return contador;
  }

  function crearPlaceholder(anchoPx: number): HTMLElement {
    // Bug: si `contenedor` es un <tbody> real (tabla de Coordenadas),
    // insertar un <div> directo como hijo es HTML inválido — el motor
    // de layout de tabla lo trata como fila anónima de ancho 100% y
    // descoloca las columnas de las demás filas (el campo de número
    // terminaba ocupando toda la pantalla). Acá se arma un <tr><td>
    // real, con colSpan = cantidad de columnas de la tabla, para que
    // el resto de las filas no se vea afectado.
    if (contenedor.tagName === "TBODY") {
      const fila = document.createElement("tr");

      fila.className = CLASE_PLACEHOLDER;

      const celda = document.createElement("td");
      const filaEjemplo = contenedor.querySelector("tr");

      celda.colSpan = filaEjemplo ? filaEjemplo.children.length : 1;

      fila.append(celda);

      return fila;
    }

    const placeholder = document.createElement("div");

    placeholder.className = CLASE_PLACEHOLDER;

    placeholder.style.width = `${anchoPx}px`;

    return placeholder;
  }

  function filasVisiblesOrdenadas(idsGrupo: string[]): HTMLElement[] {
    const excluidos = new Set(idsGrupo);

    return [...contenedor.children].filter((hijo): hijo is HTMLElement => {
      if (!(hijo instanceof HTMLElement)) return false;

      const id = idPorElemento.get(hijo);

      return id !== undefined && !excluidos.has(id);
    });
  }

  function reposicionarPlaceholder(
    placeholder: HTMLElement,
    idsGrupo: string[],
    clientY: number,
  ): void {
    const visibles = filasVisiblesOrdenadas(idsGrupo);

    let referencia: HTMLElement | null = null;

    for (const fila of visibles) {
      const rect = fila.getBoundingClientRect();

      const medio = rect.top + rect.height / 2;

      if (clientY < medio) {
        referencia = fila;

        break;
      }
    }

    // D7 — separador colapsado: si el puntero cayó sobre un header de
    // separador (div.fila-separador) y ese separador está colapsado (el
    // elemento siguiente en el contenedor también es un header o no
    // existe), insertar el placeholder justo después del header — que
    // es donde terminaría el tramo de ese separador, produciendo el
    // resultado correcto al llamar ordenTrasPlaceholder.
    if (!referencia) {
      // El puntero está por debajo de todos los visibles — verificar
      // si el último visible es un header colapsado.
      const ultimoVisible = visibles[visibles.length - 1];

      if (
        ultimoVisible &&
        ultimoVisible.classList.contains("fila-separador") &&
        obtenerIdsSeparadores?.().includes(
          idPorElemento.get(ultimoVisible) ?? "",
        )
      ) {
        const siguiente =
          ultimoVisible.nextElementSibling as HTMLElement | null;

        const esColapsado =
          !siguiente ||
          (siguiente !== placeholder &&
            siguiente.classList.contains("fila-separador"));

        if (esColapsado) {
          const despuesDelHeader =
            ultimoVisible.nextSibling === placeholder
              ? null
              : ultimoVisible.nextSibling;

          if (despuesDelHeader !== placeholder) {
            contenedor.insertBefore(placeholder, ultimoVisible.nextSibling);
          }

          return;
        }
      }
    } else if (
      referencia.classList.contains("fila-separador") &&
      obtenerIdsSeparadores?.().includes(idPorElemento.get(referencia) ?? "")
    ) {
      // El puntero está sobre un header. Verificar si está colapsado.
      const siguiente = referencia.nextElementSibling as HTMLElement | null;

      const esColapsado =
        !siguiente ||
        (siguiente !== placeholder &&
          siguiente.classList.contains("fila-separador"));

      if (esColapsado) {
        // Insertar después del header (al final del tramo colapsado).
        const destino = referencia.nextSibling;

        if (placeholder.nextSibling !== destino) {
          contenedor.insertBefore(placeholder, destino);
        }

        return;
      }
    }

    if (referencia) {
      if (placeholder.nextSibling !== referencia) {
        contenedor.insertBefore(placeholder, referencia);
      }
    } else if (contenedor.lastElementChild !== placeholder) {
      contenedor.appendChild(placeholder);
    }
  }

  function ordenTrasPlaceholder(
    idsGrupo: string[],
    placeholder: HTMLElement,
  ): string[] {
    const resultado: string[] = [];

    for (const hijo of [...contenedor.children]) {
      if (hijo === placeholder) {
        resultado.push(...idsGrupo);

        continue;
      }

      if (!(hijo instanceof HTMLElement)) continue;

      const id = idPorElemento.get(hijo);

      if (id !== undefined && !idsGrupo.includes(id)) resultado.push(id);
    }

    // Si por algún motivo el placeholder no llegó a insertarse
    // (no debería pasar), el grupo va al final.
    if (!resultado.some((id) => idsGrupo.includes(id)))
      resultado.push(...idsGrupo);

    return resultado;
  }

  function iniciarArrastre(
    idsGrupoSinOrdenar: string[],
    eventoInicial: PointerEvent,
  ): void {
    // Regla 11: expandir automáticamente cualquier separador
    // contraído que vaya a arrastrarse, ANTES de leer el orden
    // vigente y de tomar el elemento del DOM — esSeparadorContraido
    // ya se encarga de mutar el modelo y reconstruir la tabla
    // cuando corresponde, dejando las filas antes ocultas
    // presentes en el DOM (y re-registradas en `filas`) para el
    // resto de este gesto.
    idsGrupoSinOrdenar.forEach((id) => {
      esSeparadorContraido?.(id);
    });

    const ordenVigente = obtenerOrdenIds();

    limpiarFilasFantasma(ordenVigente);

    // El grupo mantiene su orden relativo ORIGINAL (spec).
    const idsGrupo = ordenVigente.filter((id) =>
      idsGrupoSinOrdenar.includes(id),
    );

    if (idsGrupo.length === 0) return;

    const primerElemento = filas.get(idsGrupo[0])?.elemento;

    if (!primerElemento) return;

    const rectPrimero = primerElemento.getBoundingClientRect();

    const fantasma = crearFantasma(idsGrupo, rectPrimero);

    const contador =
      idsGrupo.length > 1 ? crearContador(idsGrupo.length) : null;

    const placeholder = crearPlaceholder(rectPrimero.width);

    placeholder.style.height = `${Math.max(
      rectPrimero.height,
      alturaMinimaPlaceholder(),
    )}px`;

    idsGrupo.forEach((id) => {
      filas.get(id)?.elemento.classList.add(CLASE_FILA_OCULTA);
    });

    contenedor.insertBefore(placeholder, primerElemento);

    contenedor.classList.add(CLASE_CONTENEDOR_ARRASTRANDO);

    const offsetX = eventoInicial.clientX - rectPrimero.left;
    const offsetY = eventoInicial.clientY - rectPrimero.top;

    posicionarFantasma(
      fantasma,
      eventoInicial.clientX,
      eventoInicial.clientY,
      offsetX,
      offsetY,
    );

    if (contador) {
      posicionarContador(
        contador,
        eventoInicial.clientX,
        eventoInicial.clientY,
      );
    }

    arrastreActual = {
      idsGrupo,
      fantasma,
      contador,
      placeholder,
      offsetX,
      offsetY,
      seMovio: false,
    };

    document.addEventListener("pointermove", manejarPointerMoveArrastre);
    document.addEventListener("pointerup", manejarPointerUpArrastre);
  }

  // El contador sigue al cursor directamente (con un pequeño
  // desplazamiento para no quedar tapado por el puntero), a
  // diferencia del fantasma que sigue con el offset del click
  // original dentro de la fila.
  function posicionarContador(
    contador: HTMLElement,
    clientX: number,
    clientY: number,
  ): void {
    contador.style.left = `${clientX + 12}px`;
    contador.style.top = `${clientY - 10}px`;
  }

  function posicionarFantasma(
    fantasma: HTMLElement,
    clientX: number,
    clientY: number,
    offsetX: number,
    offsetY: number,
  ): void {
    fantasma.style.left = `${clientX - offsetX}px`;
    fantasma.style.top = `${clientY - offsetY}px`;
  }

  function manejarPointerMoveArrastre(evento: PointerEvent): void {
    if (!arrastreActual) return;

    arrastreActual.seMovio = true;

    posicionarFantasma(
      arrastreActual.fantasma,
      evento.clientX,
      evento.clientY,
      arrastreActual.offsetX,
      arrastreActual.offsetY,
    );

    if (arrastreActual.contador) {
      posicionarContador(
        arrastreActual.contador,
        evento.clientX,
        evento.clientY,
      );
    }

    reposicionarPlaceholder(
      arrastreActual.placeholder,
      arrastreActual.idsGrupo,
      evento.clientY,
    );
  }

  function manejarPointerUpArrastre(): void {
    if (!arrastreActual) return;

    const { idsGrupo, fantasma, contador, placeholder } = arrastreActual;

    document.removeEventListener("pointermove", manejarPointerMoveArrastre);
    document.removeEventListener("pointerup", manejarPointerUpArrastre);

    const rectPlaceholder = placeholder.getBoundingClientRect();

    const nuevoOrden = ordenTrasPlaceholder(idsGrupo, placeholder);

    placeholder.remove();
    fantasma.remove();
    contador?.remove();

    contenedor.classList.remove(CLASE_CONTENEDOR_ARRASTRANDO);

    idsGrupo.forEach((id) => {
      filas.get(id)?.elemento.classList.remove(CLASE_FILA_OCULTA);
    });

    // Las filas del grupo "vienen" visualmente desde donde
    // estaba el placeholder — el resto usa su propia posición
    // anterior (comportamiento default de aplicarNuevoOrden).
    const origenesGrupo = new Map<string, DOMRect>();

    idsGrupo.forEach((id) => origenesGrupo.set(id, rectPlaceholder));

    aplicarNuevoOrden(
      nuevoOrden,
      duracionAnimacionArrastreMs(contenedor),
      origenesGrupo,
    );

    arrastreActual = null;
  }

  // ======================================================
  // ⟫ BOTÓN ASA — clic mantenido → modo Mover
  // ======================================================

  function grupoParaArrastrarDesde(id: string): string[] {
    return seleccionadas.has(id) ? [...seleccionadas] : [id];
  }

  function manejarAsaPointerDown(id: string, evento: PointerEvent): void {
    if (evento.button !== 0) return;

    cancelarPresionAsa();

    const conCtrl = evento.ctrlKey;

    const timerId = setTimeout(() => {
      if (!presionAsaActual || presionAsaActual.cancelada) return;

      presionAsaActual.convertidaEnMover = true;

      if (!seleccionadas.has(id)) {
        if (conCtrl) {
          seleccionar(id);
        } else {
          reemplazarSeleccionPor(id);
        }
      }

      anclaSeleccion = id;

      // Ya estamos en modo Mover: si el puntero se sigue
      // moviendo a partir de ahora, se convierte en arrastre.
      iniciarArrastre(grupoParaArrastrarDesde(id), evento);
    }, tiempoMantenidoActualMs());

    presionAsaActual = {
      id,
      timerId,
      inicioX: evento.clientX,
      inicioY: evento.clientY,
      cancelada: false,
      convertidaEnMover: false,
    };

    document.addEventListener("pointermove", manejarAsaPointerMove);
    document.addEventListener("pointerup", manejarAsaPointerUp);
  }

  function cancelarPresionAsa(): void {
    if (!presionAsaActual) return;

    clearTimeout(presionAsaActual.timerId);

    presionAsaActual = null;

    document.removeEventListener("pointermove", manejarAsaPointerMove);
    document.removeEventListener("pointerup", manejarAsaPointerUp);
  }

  function manejarAsaPointerMove(evento: PointerEvent): void {
    if (!presionAsaActual || presionAsaActual.cancelada) return;

    // Si ya se convirtió en arrastre, del movimiento se ocupa
    // manejarPointerMoveArrastre (el propio arrastre ya está
    // escuchando document en paralelo).
    if (presionAsaActual.convertidaEnMover) return;

    const dx = evento.clientX - presionAsaActual.inicioX;
    const dy = evento.clientY - presionAsaActual.inicioY;

    if (Math.hypot(dx, dy) > UMBRAL_ARRASTRE_PX) {
      // Se movió antes de completar el mantenido: se cancela
      // el gesto entero (no selecciona, no arrastra). El clic
      // corto tampoco debería llegar a disparar el menú porque
      // el mouseup terminará fuera del botón en la mayoría de
      // los casos — y si no, es un clic legítimo.
      presionAsaActual.cancelada = true;

      cancelarPresionAsa();
    }
  }

  function manejarAsaPointerUp(evento: PointerEvent): void {
    if (!presionAsaActual) return;

    const { id, convertidaEnMover } = presionAsaActual;

    document.removeEventListener("pointermove", manejarAsaPointerMove);
    document.removeEventListener("pointerup", manejarAsaPointerUp);

    presionAsaActual = null;

    if (!convertidaEnMover) {
      // Shift+click simple sobre el asa con modo Mover ya activo:
      // selecciona el tramo entre la última ancla y esta fila
      // (estilo selector de archivos), sin mover el ancla — así
      // se puede seguir extendiendo el rango con más Shift+click.
      if (evento.shiftKey && seleccionadas.size > 0) {
        const asa = filas.get(id)?.asa;

        if (asa) asasASuprimirClick.add(asa);

        if (anclaSeleccion) {
          seleccionarRango(anclaSeleccion, id);
        } else {
          reemplazarSeleccionPor(id);

          anclaSeleccion = id;
        }

        onSeleccionCambio?.();

        return;
      }

      // Ctrl+click simple sobre el asa con modo Mover ya activo:
      // alterna la selección de esta fila (igual que Ctrl+click sobre
      // el fondo) y suprime el click para que no abra el popup de
      // opciones. Sin modo Mover activo, Ctrl no cambia nada — el
      // click pasa normal y abre el popup.
      if (evento.ctrlKey && seleccionadas.size > 0) {
        const asa = filas.get(id)?.asa;

        if (asa) asasASuprimirClick.add(asa);

        if (seleccionadas.has(id)) {
          deseleccionar(id);
        } else {
          seleccionar(id);
        }

        anclaSeleccion = id;

        onSeleccionCambio?.();
      }

      return;
    }

    const asa = filas.get(id)?.asa;

    if (asa) asasASuprimirClick.add(asa);

    if (arrastreActual && !arrastreActual.seMovio) {
      // Se mantuvo pero nunca se movió: no hay nada que reordenar, así
      // que no se llama a aplicarNuevoOrden/onReordenar — pero la
      // selección SÍ cambió (reemplazarSeleccionPor/seleccionar ya se
      // ejecutó en manejarAsaPointerDown al activar modo Mover), así
      // que se notifica igual vía onSeleccionCambio para que la
      // cabecera del editor de Macros refleje la nueva selección sin
      // esperar a una segunda fila.
      const { idsGrupo, fantasma, contador, placeholder } = arrastreActual;

      document.removeEventListener("pointermove", manejarPointerMoveArrastre);
      document.removeEventListener("pointerup", manejarPointerUpArrastre);

      placeholder.remove();
      fantasma.remove();
      contador?.remove();

      contenedor.classList.remove(CLASE_CONTENEDOR_ARRASTRANDO);

      idsGrupo.forEach((id) => {
        filas.get(id)?.elemento.classList.remove(CLASE_FILA_OCULTA);
      });

      arrastreActual = null;

      onSeleccionCambio?.();
    }
  }

  // Asas cuyo próximo evento "click" hay que ahogar porque la
  // presión en curso ya se convirtió en modo Mover (spec: clic
  // corto = menú, clic mantenido = Mover, nunca ambos a la vez
  // para la misma presión física del botón).
  const asasASuprimirClick = new WeakSet<HTMLElement>();

  // ======================================================
  // 🧍 FONDO DE FILA — Ctrl+clic y arrastre de selección
  // ======================================================

  function manejarFilaPointerDown(id: string, evento: PointerEvent): void {
    if (evento.button !== 0) return;

    // Ignorar pointerdown que en realidad cayó sobre CUALQUIER
    // <button> hijo — otras columnas, o el propio asa ⟫ (que ya
    // tiene su propio manejo separado y no debe procesarse acá
    // también, aunque el evento burbujee hasta la fila).
    if ((evento.target as HTMLElement).closest("button")) return;

    const conCtrl = evento.ctrlKey;

    const conShift = evento.shiftKey;

    if (!conCtrl && !conShift && !seleccionadas.has(id)) {
      // Fondo de fila sin Ctrl/Shift y sin selección previa: fuera
      // del alcance de este componente (spec no lo define).
      return;
    }

    presionFilaActual = {
      id,
      inicioX: evento.clientX,
      inicioY: evento.clientY,
    };

    document.addEventListener("pointermove", manejarFilaPointerMove);
    document.addEventListener("pointerup", manejarFilaPointerUp);
  }

  function manejarFilaPointerMove(evento: PointerEvent): void {
    if (!presionFilaActual) return;

    const dx = evento.clientX - presionFilaActual.inicioX;
    const dy = evento.clientY - presionFilaActual.inicioY;

    if (Math.hypot(dx, dy) <= UMBRAL_ARRASTRE_PX) return;

    const { id } = presionFilaActual;

    document.removeEventListener("pointermove", manejarFilaPointerMove);
    document.removeEventListener("pointerup", manejarFilaPointerUp);

    presionFilaActual = null;

    // Shift + fila sin seleccionar + movimiento: Shift es para
    // seleccionar rango, no para arrastrar — se cancela el gesto
    // sin iniciar arrastre ni tocar la selección.
    if (evento.shiftKey && !seleccionadas.has(id)) return;

    // Ctrl + fila sin seleccionar + arrastre → se agrega al
    // grupo antes de largar el arrastre (spec, sección 4).
    if (!seleccionadas.has(id)) seleccionar(id);

    iniciarArrastre(grupoParaArrastrarDesde(id), evento);
  }

  function manejarFilaPointerUp(evento: PointerEvent): void {
    if (!presionFilaActual) return;

    const { id } = presionFilaActual;

    document.removeEventListener("pointermove", manejarFilaPointerMove);
    document.removeEventListener("pointerup", manejarFilaPointerUp);

    presionFilaActual = null;

    if (evento.shiftKey) {
      // Shift+clic sin arrastre sobre el fondo: mismo criterio que
      // en el asa — selecciona el tramo entre la última ancla y
      // esta fila, sin mover el ancla.
      if (anclaSeleccion) {
        seleccionarRango(anclaSeleccion, id);
      } else {
        reemplazarSeleccionPor(id);

        anclaSeleccion = id;
      }

      onSeleccionCambio?.();

      return;
    }

    if (!evento.ctrlKey) return; // clic simple sobre fila ya seleccionada: no hace nada (spec no lo define).

    // Ctrl+clic sin arrastre: alterna la selección según el
    // estado ANTES de este clic (evita seleccionar y
    // deseleccionar en el mismo clic).
    if (seleccionadas.has(id)) {
      deseleccionar(id);
    } else {
      seleccionar(id);
    }

    anclaSeleccion = id;

    onSeleccionCambio?.();
  }

  // ======================================================
  // 🖱️ CLIC FUERA DE CUALQUIER FILA → salir del modo Mover
  // ======================================================

  function manejarPointerDownGlobal(evento: PointerEvent): void {
    if (seleccionadas.size === 0) return;

    const objetivo = evento.target as Node | null;

    if (!objetivo) return;

    for (const registro of filas.values()) {
      if (
        registro.elemento.contains(objetivo) ||
        registro.asa.contains(objetivo)
      ) {
        return; // el clic fue sobre alguna fila registrada: no es "afuera".
      }
    }

    if (elementosExentosClickAfuera?.some((el) => el.contains(objetivo))) {
      return; // el clic fue sobre un elemento exento (ej. cabecera con botones de lote).
    }

    salirModoMover();
  }

  document.addEventListener("pointerdown", manejarPointerDownGlobal, true);
  document.addEventListener("keydown", manejarTeclado);

  // ======================================================
  // 📥 REGISTRO DE FILAS
  // ======================================================

  function registrarFila(
    id: string,
    filaElemento: HTMLElement,
    botonAsa: HTMLElement,
  ): void {
    filas.set(id, { elemento: filaElemento, asa: botonAsa });

    idPorElemento.set(filaElemento, id);

    refrescarClaseFila(id);

    botonAsa.addEventListener("pointerdown", (evento) =>
      manejarAsaPointerDown(id, evento),
    );

    // Capture:true para garantizar que corre ANTES que el
    // listener "click" que el llamador agrega en fase bubble
    // para abrir el menú de opciones (ver comp_accion.ts).
    botonAsa.addEventListener(
      "click",
      (evento) => {
        if (!asasASuprimirClick.has(botonAsa)) return;

        asasASuprimirClick.delete(botonAsa);

        evento.stopImmediatePropagation();
        evento.preventDefault();
      },
      true,
    );

    filaElemento.addEventListener("pointerdown", (evento) =>
      manejarFilaPointerDown(id, evento),
    );
  }

  function destruir(): void {
    cancelarPresionAsa();

    if (arrastreActual) manejarPointerUpArrastre();

    document.removeEventListener("pointerdown", manejarPointerDownGlobal, true);
    document.removeEventListener("keydown", manejarTeclado);
  }

  // ======================================================
  // ↕️ ACTIVAR MODO MOVER DESDE EL MENÚ (sin mantenido)
  // ------------------------------------------------------
  // Ver interfaz pública ControladorArrastre.activarModoMoverPara.
  // ======================================================

  function activarModoMoverPara(id: string): void {
    if (!filas.has(id)) return;

    const anteriores = [...seleccionadas];

    seleccionadas.clear();

    seleccionadas.add(id);

    anteriores.forEach(refrescarClaseFila);

    refrescarClaseFila(id);
  }

  return {
    registrarFila,
    estaSeleccionada: (id: string) => seleccionadas.has(id),
    obtenerSeleccionadas: () => [...seleccionadas],
    salirModoMover,
    activarModoMoverPara,
    seleccionarAdicional: (id: string) => {
      if (filas.has(id)) seleccionar(id);
    },
    obtenerAncla: () => anclaSeleccion,
    establecerAncla: (id: string | null) => {
      anclaSeleccion = id;
    },
    destruir,
  };
}
