// ======================================================
// ❔ Ayuda
// ======================================================

use std::collections::HashMap;
use std::sync::OnceLock;

static CATALOGO_AYUDA: OnceLock<HashMap<String, String>> = OnceLock::new();

pub fn cargar_catalogo() -> &'static HashMap<String, String> {
    CATALOGO_AYUDA.get_or_init(|| {
        let texto = include_str!("ayuda.txt");

        let mut catalogo: HashMap<String, String> = HashMap::new();

        let mut bloques: Vec<(Vec<String>, String)> = Vec::new();

        let mut ids_actuales: Vec<String> = Vec::new();
        let mut contenido_actual = String::new();

        for linea in texto.lines() {
            let linea = linea.strip_suffix('\r').unwrap_or(linea);

            if let Some(cabecera) = linea.strip_prefix("## ") {
                if !ids_actuales.is_empty() {
                    bloques.push((ids_actuales, contenido_actual));
                }

                ids_actuales = cabecera.split(',').map(|id| id.trim().to_string()).collect();
                contenido_actual = String::new();
                continue;
            }

            contenido_actual.push_str(linea);
            contenido_actual.push('\n');
        }

        if !ids_actuales.is_empty() {
            bloques.push((ids_actuales, contenido_actual));
        }

        catalogo
    })
}

