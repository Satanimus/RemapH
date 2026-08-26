// ======================================================
// 🎨 core_Apariencia
// ------------------------------------------------------
// Aplica los overrides CSS guardados (pestaña Apariencia de
// la Ventana de Configuración) como estilo inline en <html>,
// apenas arranca cada ventana — así todas parten ya con la
// paleta/tamaños personalizados, sin depender de un segundo
// paso posterior al primer pintado.
//
// Lo importan main.ts, captura_main.ts, menu_express_main.ts,
// portapapeles_main.ts y configuracion_main.ts: cada uno lo
// llama una sola vez, apenas se ejecuta el módulo. Cuando se
// guarda un cambio de Apariencia (o se carga un tema), el
// backend recarga (location.reload) todas las ventanas
// abiertas — ver comandos.rs::configuracion_refrescar_ventanas_apariencia —
// y, al recargar, este módulo vuelve a correr y aplica los
// valores nuevos.
//
// El estilo inline en <html> siempre gana por especificidad
// sobre las reglas ":root { ... }" de styl_variables.css (sin
// importar en qué orden se importen los CSS), así que no hace
// falta ninguna coordinación extra con ese archivo.
// ======================================================

import { invoke } from "@tauri-apps/api/core";

// Claves "--xxx" aplicadas la última vez que corrió esta función en
// ESTA ventana. Las demás ventanas no lo necesitan (se recargan
// enteras vía location.reload y arrancan con <html> limpio), pero la
// propia Ventana de Configuración llama esta función de nuevo sin
// recargar (ver vent_configuracion_main.ts::refrescarTrasCambioApariencia)
// — sin este registro, una clave que tenía valor en la llamada
// anterior y ya no viene en el mapa nuevo (cambio de tema con menos
// overrides, restablecer, etc.) se quedaba pisando el valor viejo.
let clavesAplicadasPrevias: string[] = [];

export async function aplicarOverridesApariencia(): Promise<void> {
  try {
    const overrides = await invoke<Record<string, string>>(
      "obtener_overrides_apariencia",
    );

    const raiz = document.documentElement;

    const clavesNuevas = new Set(
      Object.keys(overrides).map((clave) => `--${clave}`),
    );

    for (const clave of clavesAplicadasPrevias) {
      if (!clavesNuevas.has(clave)) {
        raiz.style.removeProperty(clave);
      }
    }

    for (const [clave, valor] of Object.entries(overrides)) {
      raiz.style.setProperty(`--${clave}`, valor);
    }

    clavesAplicadasPrevias = [...clavesNuevas];

    const modo = overrides["fondo-general-modo"] ?? "plano";

    raiz.style.setProperty(
      "--fondo-general",
      modo === "degradado"
        ? "linear-gradient(135deg, var(--bg), var(--fondo-general-color2))"
        : "var(--bg)",
    );
  } catch (error) {
    console.error(
      "⚠️ No se pudieron aplicar los overrides de Apariencia:",
      error,
    );
  }
}
