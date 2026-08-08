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
// 📝 TEXTO ACCIÓN (columna Acción de la tabla)
// ------------------------------------------------------
// Default "📋 Editar" hasta que tenga nombre — a partir de ahí,
// portapapeles + nombre (mismo criterio que textoMenuAccion en
// core_menu_express.ts).
// ======================================================

export function textoPortapapelesAccion(
  portapapelesAccion: PortapapelesAccionPerfil,
): string {
  return portapapelesAccion.nombre
    ? `📋 ${portapapelesAccion.nombre}`
    : "📋 Editar";
}

// ======================================================
// 📝 TEXTO EXTRA (columna Extra de la tabla)
// ======================================================

export function textoPortapapelesExtra(
  portapapelesExtra: PortapapelesExtraPerfil,
): string {
  return portapapelesExtra.comportamiento === "efimero" ? "Efímero" : "Toggle";
}
