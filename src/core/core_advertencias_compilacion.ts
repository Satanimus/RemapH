// ======================================================
// ⚠️ core_Advertencias_Compilacion
// ------------------------------------------------------
// Guarda en memoria el Vec<AdvertenciaCompilacion> que devuelve
// compilar_perfil() (ver ResultadoCompilacion en compilador.rs) tras
// cada guardado — hoy solo lo genera convertir_abrir() cuando la
// ruta de una fila "abrir" ya no existe en disco.
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
