// ======================================================
// 🗂️ back_registro
// ======================================================
// ETAPA 7B DEL FLUJO "ABRIR ARCHIVO/APP"
// ------------------------------------------------------
// 1. ¿Qué hace este archivo?
//
// Backend AISLADO del resto (solo lee el registro de Windows, sin
// ninguna dependencia de la Windows API gráfica — eso vive en
// back_app.rs). Responsable de armar el listado de "Abrir con..."
// que ofrece el popup Extra del tipo "Abrir Archivo/App" (Etapa 11):
// qué programas usó recientemente el usuario para una extensión
// dada, y qué programas hay instalados en general.
//
// ------------------------------------------------------
// 2. ¿Qué información recibe?
//
// Recibe una extensión de archivo (ej. "txt", ".txt") o ningún
// parámetro, según la consulta.
//
// ------------------------------------------------------
// 3. ¿Quién llama este archivo?
//
// comandos.rs::obtener_programas_abrir_con().
//
// ------------------------------------------------------
// 4. ¿Qué información entrega?
//
// Listas de ProgramaRegistro{nombre, ruta} — la ruta ya resuelta al
// ejecutable real, lista para usarse como "Abrir con" o para pedirle
// su ícono a back_app.rs::extraer_icono_ruta(). También, suelta, la
// ruta del programa predeterminado de una extensión puntual.
//
// ------------------------------------------------------
// 5. Funciones del archivo
//
// obtener_recientes(extension)
//     Lee HKCU\...\FileExts\<ext>\OpenWithList, en el orden real de
//     uso (MRUList) cuando está disponible.
//
// obtener_instalados()
//     Enumera HKCR\Applications — todo lo que tiene un handler
//     shell\open\command asociado.
//
// resolver_ruta_comando() / ruta_desde_applications()
//     Parser compartido: de un valor crudo de shell\open\command (o
//     de un nombre de exe a resolver contra Applications\<exe>) saca
//     la ruta real del ejecutable, sin comillas ni "%1", con sus
//     variables de entorno ya expandidas (ver expandir_variables_entorno).
//
// expandir_variables_entorno(ruta)
//     BUG 1: expande %SystemRoot%, %ProgramFiles%, etc. vía
//     ExpandEnvironmentStringsW — el registro a veces las devuelve
//     sin expandir y ShellExecuteExW no lo hace por su cuenta.
//
// programa_predeterminado(extension)
//     El programa que Windows abriría por defecto para esa extensión
//     (HKCR\.ext → ProgID → HKCR\<ProgID>\shell\open\command). Usado
//     por runtime.rs (nombre_proceso_objetivo() y abrir_archivo()) del
//     tipo "Abrir Archivo/App" para lanzar el visor/editor asociado
//     directamente en vez de delegar en ShellExecuteExW "open" sobre
//     el documento — necesario para que Instancias Múltiple/Iniciar
//     Minimizado-Maximizado tengan efecto real (ver runtime.rs).
// ======================================================

use std::collections::HashSet;
use std::path::Path;

use winreg::enums::{HKEY_CLASSES_ROOT, HKEY_CURRENT_USER};
use winreg::RegKey;

use windows_sys::Win32::System::Environment::ExpandEnvironmentStringsW;

// ======================================================
// MODELO
// ======================================================

pub struct ProgramaRegistro {
    pub nombre: String,

    pub ruta: String,
}

// ======================================================
// PARSER COMPARTIDO — VALOR CRUDO → RUTA REAL
// ------------------------------------------------------
// El valor por defecto de una clave shell\open\command viene en
// formas como:
//   "C:\Program Files\App\app.exe" "%1"
//   C:\Windows\notepad.exe %1
//   "C:\Program Files\App\app.exe"
// Esta función se queda solo con la ruta del ejecutable, sin
// comillas ni el resto de los argumentos/placeholders.
// ======================================================

fn resolver_ruta_comando(comando: &str) -> Option<String> {
    let comando = comando.trim();

    if comando.is_empty() {
        return None;
    }

    let ruta = if let Some(resto) = comando.strip_prefix('"') {
        let fin = resto.find('"')?;

        &resto[..fin]
    } else {
        comando.split_whitespace().next()?
    };

    if ruta.is_empty() {
        None
    } else {
        Some(expandir_variables_entorno(ruta))
    }
}

// ======================================================
// EXPANDIR VARIABLES DE ENTORNO (BUG 1)
// ------------------------------------------------------
// El valor crudo de shell\open\command puede venir con variables sin
// expandir, ej. "%SystemRoot%\System32\notepad.exe" — el Explorador
// las expande solo al ejecutar, pero ShellExecuteExW (lo que usa
// runtime.rs) NO lo hace, así que sin este paso Windows recibe la
// ruta literal "%SystemRoot%\..." y falla con "no puede encontrar el
// archivo". Se expande acá, una única vez, para que toda ruta que
// sale de este archivo ya esté lista para pasarle directo a
// ShellExecuteExW. Si ExpandEnvironmentStringsW falla o la ruta no
// tenía variables, devuelve la ruta original sin tocar.
// ======================================================

fn expandir_variables_entorno(ruta: &str) -> String {
    let entrada: Vec<u16> = ruta.encode_utf16().chain(std::iter::once(0)).collect();

    let mut buffer = [0u16; 1024];

    let escritos = unsafe {
        ExpandEnvironmentStringsW(entrada.as_ptr(), buffer.as_mut_ptr(), buffer.len() as u32)
    };

    // escritos incluye el terminador nulo; 0 es error, y un valor
    // mayor al buffer significa que no entró completo (ruta
    // absurdamente larga) — en ambos casos se prefiere devolver la
    // ruta cruda antes que una expansión truncada/corrupta.
    if escritos == 0 || escritos as usize > buffer.len() {
        return ruta.to_string();
    }

    let largo = (escritos - 1) as usize;

    String::from_utf16_lossy(&buffer[..largo])
}

// ======================================================
// RESOLVER RUTA DESDE Applications\<exe>
// ------------------------------------------------------
// Punto de entrada compartido por obtener_recientes() (que solo
// tiene el nombre del exe, ej. "notepad++.exe") y obtener_instalados()
// (que ya está parada sobre la clave Applications\<exe>): ambas
// terminan leyendo el mismo valor shell\open\command y pasándolo por
// resolver_ruta_comando().
// ======================================================

fn ruta_desde_applications(exe: &str) -> Option<String> {
    let raiz = RegKey::predef(HKEY_CLASSES_ROOT);

    let clave = raiz
        .open_subkey(format!("Applications\\{}\\shell\\open\\command", exe))
        .ok()?;

    let valor: String = clave.get_value("").ok()?;

    resolver_ruta_comando(&valor)
}

// ======================================================
// NOMBRE AMIGABLE
// ------------------------------------------------------
// El registro solo da el nombre de archivo del exe (ej.
// "notepad++.exe") — nunca un nombre "bonito" tipo "Notepad++". Se
// usa el nombre de archivo sin extensión como aproximación simple;
// la UI (Etapa 11) puede mostrarlo tal cual, junto al ícono real.
// ======================================================

fn nombre_amigable(exe: &str) -> String {
    Path::new(exe)
        .file_stem()
        .map(|nombre| nombre.to_string_lossy().to_string())
        .unwrap_or_else(|| exe.to_string())
}

// ======================================================
// RECIENTES POR EXTENSIÓN
// ======================================================

pub fn obtener_recientes(extension: &str) -> Vec<ProgramaRegistro> {
    let extension = extension.trim_start_matches('.');

    let raiz = RegKey::predef(HKEY_CURRENT_USER);

    let ruta_clave = format!(
        "Software\\Microsoft\\Windows\\CurrentVersion\\Explorer\\FileExts\\.{}\\OpenWithList",
        extension
    );

    let Ok(clave) = raiz.open_subkey(&ruta_clave) else {
        return Vec::new();
    };

    // MRUList da el orden real de uso (ej. "ba" = primero el valor
    // "b", después el "a"). Si no está, se recorre en el orden que
    // entregue el propio registro — sigue siendo mejor que nada.
    let mut letras: Vec<String> = clave
        .get_value::<String, _>("MRUList")
        .map(|mru| mru.chars().map(|letra| letra.to_string()).collect())
        .unwrap_or_default();

    if letras.is_empty() {
        letras = clave
            .enum_values()
            .filter_map(|entrada| entrada.ok())
            .map(|(nombre, _)| nombre)
            .filter(|nombre| nombre != "MRUList")
            .collect();
    }

    let mut vistos = HashSet::new();

    let mut lista = Vec::new();

    for letra in letras {
        let Ok(exe) = clave.get_value::<String, _>(&letra) else {
            continue;
        };

        if !vistos.insert(exe.to_lowercase()) {
            continue;
        }

        let Some(ruta) = ruta_desde_applications(&exe) else {
            continue;
        };

        lista.push(ProgramaRegistro {
            nombre: nombre_amigable(&exe),
            ruta,
        });
    }

    lista
}

// ======================================================
// PROGRAMA PREDETERMINADO DE UNA EXTENSIÓN
// ------------------------------------------------------
// A diferencia de obtener_recientes() (OpenWithList, historial del
// usuario), esto resuelve el programa que Windows usaría HOY por
// defecto para abrir esa extensión: HKCR\.ext da el ProgID asociado,
// y HKCR\<ProgID>\shell\open\command da el comando real — mismo
// parser (resolver_ruta_comando) que ya usa el resto del archivo.
// ======================================================

pub fn programa_predeterminado(extension: &str) -> Option<String> {
    let extension = extension.trim_start_matches('.');

    if extension.is_empty() {
        return None;
    }

    let raiz = RegKey::predef(HKEY_CLASSES_ROOT);

    let clave_extension = raiz.open_subkey(format!(".{}", extension)).ok()?;

    let prog_id: String = clave_extension.get_value("").ok()?;

    if prog_id.is_empty() {
        return None;
    }

    let clave_comando = raiz
        .open_subkey(format!("{}\\shell\\open\\command", prog_id))
        .ok()?;

    let comando: String = clave_comando.get_value("").ok()?;

    resolver_ruta_comando(&comando)
}

// ======================================================
// TODOS LOS INSTALADOS
// ======================================================

pub fn obtener_instalados() -> Vec<ProgramaRegistro> {
    let raiz = RegKey::predef(HKEY_CLASSES_ROOT);

    let Ok(aplicaciones) = raiz.open_subkey("Applications") else {
        return Vec::new();
    };

    let mut lista = Vec::new();

    for exe in aplicaciones.enum_keys().filter_map(|entrada| entrada.ok()) {
        let Some(ruta) = ruta_desde_applications(&exe) else {
            continue;
        };

        lista.push(ProgramaRegistro {
            nombre: nombre_amigable(&exe),
            ruta,
        });
    }

    lista
}
