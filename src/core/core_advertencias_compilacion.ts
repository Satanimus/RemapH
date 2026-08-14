// ======================================================
// ⚠️ core_Advertencias_Compilacion
// ------------------------------------------------------
// Guarda en memoria el Vec<AdvertenciaCompilacion> de la ÚLTIMA
// compilación conocida — hoy solo lo genera convertir_abrir() en
// compilador.rs cuando la ruta de una fila "abrir" ya no existe en
// disco. Se actualiza en cada punto donde el backend compila y
// devuelve ResultadoCompilacion/advertencias: guardar cambios
// (main.ts::guardarPerfil), activar perfil manualmente
// (ui_toolbar.ts::botonEstado), cargar el perfil al abrir la app
// (main.ts::iniciarApp) y cambiar/clonar/renombrar/eliminar perfil
// (ui_toolbar.ts::aplicarResultadoPerfil). NO se actualiza al
// revertir cambios sin guardar (restaurar_perfil_actual no
// recompila a propósito — ver perfil.rs), para no pisar advertencias
// vigentes con una lista vacía.
//
// A diferencia de core_conflictos.ts (que recalcula en vivo, en el
// frontend, cada vez que se edita una fila), esto es un snapshot
// fijo de la ÚLTIMA compilación: no se recalcula solo, y AdvertenciaCompilacion.fila
// viaja como índice de POSICIÓN (base 1, ver compilador.rs::
// compilar_perfil) — no id de fila. Si el usuario reordena filas
// después de guardar, la advertencia queda apuntando a la posición
// vieja hasta la próxima compilación (mismo criterio documentado en
// Rust: "la ruta ya validada acá nunca se vuelve a comprobar").
// ======================================================

import type { FilaPerfil } from "./core_perfil";

// ======================================================
// 📦 MODELO (espejo de AdvertenciaCompilacion en compilador.rs)
// ======================================================

export interface AdvertenciaCompilacion {
  fila: number;

  mensaje: string;
}

// ======================================================
// 📦 RESULTADO COMPILACIÓN (espejo de ResultadoCompilacion en
// compilador.rs)
// ------------------------------------------------------
// Lo que devuelven los comandos que compilan: compilar_perfil,
// activar_perfil. Tipo compartido para no duplicarlo en cada
// llamador (ver main.ts, ui_toolbar.ts).
// ======================================================

export interface ResultadoCompilacion {
  activo: boolean;

  advertencias: AdvertenciaCompilacion[];
}

// ======================================================
// 🧠 ESTADO
// ======================================================

let advertenciasActuales: AdvertenciaCompilacion[] = [];

// ======================================================
// 📥 ESTABLECER ADVERTENCIAS
// ------------------------------------------------------
// Llamado tras cada compilar_perfil() exitoso (ver main.ts::
// guardarPerfil()) — reemplaza el snapshot anterior entero, nunca
// se acumula entre compilaciones (mismo criterio que documenta
// ResultadoCompilacion del lado Rust).
// ======================================================

export function establecerAdvertenciasCompilacion(
  advertencias: AdvertenciaCompilacion[],
): void {
  advertenciasActuales = advertencias;
}

// ======================================================
// 📤 OBTENER ADVERTENCIAS
// ======================================================

export function obtenerAdvertenciasCompilacion(): AdvertenciaCompilacion[] {
  return advertenciasActuales;
}

// ======================================================
// ❓ FILA CON ADVERTENCIA (por id, vía posición actual)
// ------------------------------------------------------
// AdvertenciaCompilacion.fila es la posición (base 1) que tenía la
// fila EN EL MOMENTO DE COMPILAR — se resuelve acá contra la
// posición actual del id en `filas` para saber si sigue
// correspondiendo a la misma fila. Mismo mecanismo que
// filaTieneConflicto() en core_conflictos.ts, usado por
// comp_controles.ts::crearEstado() para el aviso "OFF ⚠️".
// ======================================================

export function filaTieneAdvertencia(
  id: string,

  filas: FilaPerfil[],
): boolean {
  const indice = filas.findIndex((fila) => fila.id === id);

  if (indice < 0) {
    return false;
  }

  const numeroFila = indice + 1;

  return advertenciasActuales.some(
    (advertencia) => advertencia.fila === numeroFila,
  );
}
