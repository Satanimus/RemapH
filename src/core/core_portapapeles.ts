// ======================================================
// 📋 core_Portapapeles
// ------------------------------------------------------
// Modelo del tipo "Portapapeles" (filaPerfil.tipo === "portapapeles").
// El id de la propia fila ES el id del Portapapeles — no se genera
// ni guarda ningún identificador aparte (mismo criterio que
// MenuExpress, ver core_menu_express.ts).
//
// A diferencia de MenuExpress, la fila NO es dueña de ningún
// contenido propio: es solo un VISUALIZADOR de un pool de
// elementos rotatorios compartido por todo RemapH (ver
// back_portapapeles.rs). portapapelesAccion.nombre es simplemente
// el título de su ventana — no hay lista de botones que armar acá.
//
// Accion (portapapelesAccion) y Extra (portapapelesExtra) son dos
// objetos siempre presentes en la fila, igual que menuAccion/
// menuExtra — solo tienen efecto cuando tipo === "portapapeles".
// compilador.rs (Rust) los empaqueta juntos al compilar en una sola
// AccionCache::Portapapeles.
//
// Espejo exacto de PortapapelesAccionJson / PortapapelesExtraJson en
// perfil_json.rs (mismos nombres de campo, camelCase) — viaja tal
// cual hacia Rust en compilar_perfil, sin traducción adicional.
// ======================================================

export type ComportamientoPortapapeles = "toggle" | "efimero";

export type UbicacionPortapapeles = "persistente" | "cursor";

// Tamaño de TEXTO: reutiliza el mismo vocabulario/valores en px que
// ya usa MenuExpress (ver core_menu_express.ts / config.rs) — no se
// declara un tipo nuevo, se reusa TamanoMenu tal cual desde ese
// módulo en los archivos que lo necesiten.
// (el `export type ... from` de abajo solo reexporta el nombre hacia
// afuera — no lo trae al scope de ESTE archivo, por eso hace falta
// también el `import type` para poder usarlo más abajo en este mismo
// módulo, ver PortapapelesExtraPerfil).
import type { TamanoMenu as TamanoTextoPortapapeles } from "./core_menu_express";
export type { TamanoMenu as TamanoTextoPortapapeles } from "./core_menu_express";

// Tamaño de BOTÓN: tipo propio. Los botones de Portapapeles son
// filas alargadas (ícono + nombre + acciones), no cuadrados como los
// de MenuExpress, así que usan sus propios valores en px
// (config.rs: portapapeles_boton_pequeno/mediano/grande) aunque
// comparten el mismo vocabulario Pequeño/Mediano/Grande.
export type TamanoBotonPortapapeles = "pequeno" | "mediano" | "grande";

// ======================================================
// 🎯 ACCIÓN (columna Acción)
// ------------------------------------------------------
// Único campo: el nombre de la ventana. El popup de Acción
// (comp_popup_portapapeles_editor.ts) solo edita esto — a
// diferencia de MenuExpress no hay botones/referencias que
// armar acá.
// ======================================================

export interface PortapapelesAccionPerfil {
  nombre: string;
}

// ======================================================
// 🎛️ EXTRA (columna Extra)
// ------------------------------------------------------
// limite: máximo de elementos ROTATORIOS que este Portapapeles
// pide mantener cuando está en modo Registro (los fijados no
// cuentan). El límite REAL que aplica el pool compartido es el
// mayor límite configurado entre todos los Portapapeles
// actualmente en modo Registro (ver back_portapapeles.rs) — este
// campo es solo lo que la fila "pide", no lo que termina rigiendo.
// ======================================================

export interface PortapapelesExtraPerfil {
  comportamiento: ComportamientoPortapapeles;

  ubicacion: UbicacionPortapapeles;

  tamanoBoton: TamanoBotonPortapapeles;

  tamanoTexto: TamanoTextoPortapapeles;

  limite: number;
}

// ======================================================
// ➕ CREAR ACCIÓN / EXTRA POR DEFECTO
// ======================================================

export function crearPortapapelesAccion(): PortapapelesAccionPerfil {
  return {
    nombre: "",
  };
}

export function crearPortapapelesExtra(): PortapapelesExtraPerfil {
  return {
    comportamiento: "toggle",

    ubicacion: "persistente",

    tamanoBoton: "mediano",

    tamanoTexto: "mediano",

    limite: 10,
  };
}

// ======================================================
// 📋 TEXTO ACCIÓN (columna Acción de la tabla)
// ------------------------------------------------------
// Default "Editar" hasta que tenga nombre — a partir de ahí,
// solo el nombre (sin ícono: los íconos de columna Acción se
// sacaron de la ventana principal, quedan solo en el popup de
// Tipo, ver comp_popup_abrir.ts).
// ======================================================

export function textoPortapapelesAccion(
  portapapelesAccion: PortapapelesAccionPerfil,
): string {
  return portapapelesAccion.nombre ? portapapelesAccion.nombre : "Editar";
}

// ======================================================
// 📋 TOOLTIP DEL BOTÓN EXTRA (columna Extra, ícono 🔧)
// ------------------------------------------------------
// Lista de líneas "Subtítulo: Elección" — todos los campos de
// portapapelesExtra.
// ======================================================

function textoTamanoPortapapeles(
  tamano: TamanoBotonPortapapeles | TamanoTextoPortapapeles,
): string {
  switch (tamano) {
    case "pequeno":
      return "Pequeño";

    case "mediano":
      return "Mediano";

    case "grande":
      return "Grande";
  }
}

export function textoPortapapelesExtra(
  portapapelesExtra: PortapapelesExtraPerfil,
): string {
  const comportamiento =
    portapapelesExtra.comportamiento === "efimero" ? "Efímero" : "Toggle";

  const ubicacion =
    portapapelesExtra.ubicacion === "cursor" ? "Cursor" : "Persistente";

  return [
    `Comportamiento: ${comportamiento}`,
    `Ubicación: ${ubicacion}`,
    `Botones: ${textoTamanoPortapapeles(portapapelesExtra.tamanoBoton)}`,
    `Texto: ${textoTamanoPortapapeles(portapapelesExtra.tamanoTexto)}`,
    `Límite: ${portapapelesExtra.limite}`,
  ].join("\n");
}
