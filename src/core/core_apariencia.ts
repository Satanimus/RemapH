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

export async function aplicarOverridesApariencia(): Promise<void> {
  try {
    const overrides = await invoke<Record<string, string>>(
      "obtener_overrides_apariencia",
    );

    const raiz = document.documentElement;

    for (const [clave, valor] of Object.entries(overrides)) {
      raiz.style.setProperty(`--${clave}`, valor);
    }

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
