// Todo este código salió de compilador.rs
// Debe ir en el mismo archivo de ui que modifica las opciones visibles
// de la columna Accion cuando cambia Tipo.
// Es de una familia de recursos para impedir valores no permitidos en el json.
//Esa es la lógica. No olvidar agregarla.
////////////////////////////////////////////////////

use crate::perfil_json::{perfil_json, RemapeoJson};

use std::collections::HashSet;

// ======================================================
// ⚠️ ÍNDICES CONFLICTIVOS
// ======================================================

pub fn indices_conflictivos(perfil: &perfil_json) -> HashSet<usize> {
    let mut resultado = HashSet::new();

    for indice_a in 0..perfil.remapeos.len() {
        for indice_b in (indice_a + 1)..perfil.remapeos.len() {
            let fila_a = &perfil.remapeos[indice_a];

            let fila_b = &perfil.remapeos[indice_b];

            if !triggers_iguales(fila_a, fila_b) {
                continue;
            }

            if !apps_conflictivas(fila_a, fila_b) {
                continue;
            }

            resultado.insert(indice_a);

            resultado.insert(indice_b);
        }
    }

    resultado
}

// ======================================================
// 🎯 TRIGGER IDÉNTICO
// ======================================================

pub fn triggers_iguales(fila_a: &RemapeoJson, fila_b: &RemapeoJson) -> bool {
    let trigger_a = &fila_a.trigger;

    let trigger_b = &fila_b.trigger;

    let Some(gatillo_a) = &trigger_a.gatillo else {
        return false;
    };

    let Some(gatillo_b) = &trigger_b.gatillo else {
        return false;
    };

    if trigger_a.condicion != trigger_b.condicion {
        return false;
    }

    if trigger_a.modificadores.len() != trigger_b.modificadores.len() {
        return false;
    }

    for indice in 0..trigger_a.modificadores.len() {
        let entrada_a = &trigger_a.modificadores[indice];

        let entrada_b = &trigger_b.modificadores[indice];

        if entrada_a != entrada_b {
            return false;
        }
    }

    gatillo_a == gatillo_b
}

// ======================================================
// 🖥️ APPS CONFLICTIVAS
// ======================================================

pub fn apps_conflictivas(fila_a: &RemapeoJson, fila_b: &RemapeoJson) -> bool {
    match (&fila_a.app.programa, &fila_b.app.programa) {
        (None, None) => true,

        (None, Some(_)) => fila_b.app.segundo_plano,

        (Some(_), None) => fila_a.app.segundo_plano,

        (Some(programa_a), Some(programa_b)) => programa_a.eq_ignore_ascii_case(programa_b),
    }
}
