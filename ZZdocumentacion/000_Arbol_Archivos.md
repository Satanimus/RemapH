------------------------------------------
Arbol de archivos
------------------------------------------

---------------------------
🧭 Arquitectura general
---------------------------

Frontend TypeScript
│
├── core/
│   └── Modelo editable y estado temporal de la UI
│
├── ui/
│   ├── Construcción visual
│   └── Componentes interactivos
│
└── main.ts
        │
        ▼
   Perfil temporal
        │
        ▼
   [Futuro botón Guardar cambios]
        │
        ▼
Backend Rust / Tauri
        │
        ├── usuario.rs
        ├── compilador.rs
        ├── cache.rs
        └── runtime.rs
                │
                ▼
          Interception
-----------------------------


🟦 FRONTEND — src/
🚀 main.ts

Etapa: Entrada de la aplicación.

Responsabilidad:

Importar estilos.
Esperar DOMContentLoaded.
Crear la aplicación.
Insertarla en document.body.

Flujo:

main.ts
  ↓
crearApp()

No contiene:

Lógica de perfiles.
Runtime.
Backend.
Captura.
📂 src/core/
📄 core_perfil.ts

Etapa: Modelo oficial del perfil editable.

Contiene:

Perfil
FilaPerfil
crearFila()
crearPerfil()
clonarFila()

Responsabilidad:

Representar la configuración que el usuario está editando.

FilaPerfil contiene actualmente:
id
estado
trigger
tipo
accion
condicion
ejecucion
app
color
nota

Importante:

FilaPerfil es el modelo de la UI.

No es el modelo del Runtime.

📄 core_perfil_ui.ts (renombrado de temporal a ui)

Etapa: Estado vivo de edición.

Contiene:

obtenerPerfilTemporal()
establecerPerfilTemporal()

Responsabilidad:

Mantener el perfil que la UI está editando actualmente.

UI
 ↓
Perfil temporal

Regla arquitectónica acordada:

Los cambios de la UI viven aquí antes de guardarse.

El Runtime no ve estos cambios automáticamente.

📄 core_perfil_acciones.ts

Etapa: Operaciones sobre el perfil temporal.

Contiene:

clonarFilaPorId()

Responsabilidad:

Buscar filas dentro del perfil temporal y ejecutar operaciones sobre ellas.

Actualmente:

clonar fila
🎛️ core_entrada.ts

Etapa: Modelo de entrada física.

Contiene:

TipoEntrada
Entrada
crearEntrada()

Tipos actuales:

Teclado
Mouse
Multimedia
Joystick

Responsabilidad:

Representar una tecla o botón físico de forma común.

Ejemplo:

Teclado
ControlLeft
Control

También normaliza algunos nombres:

Quote       → ´
Backquote   → `
AltGraph    → AltGr
🎯 core_trigger.ts

Etapa: Modelo y representación visual de triggers.

Contiene:

CondicionTrigger
Trigger
crearTrigger()
triggerATexto()
triggerAHTML()
Modelo actual:
Trigger
├── modificadores[]
├── gatillo
└── condicion

Ejemplo:

[Ctrl + Shift] + Q

Responsabilidad:

Representar triggers.
Convertirlos a texto.
Convertirlos a HTML visual.
Aplicar la condición visual.

Importante:

Aquí está la lógica de cómo se representa un trigger en la UI.

📂 src/ui/
🖥️ ui_app.ts

Etapa: Raíz visual de la aplicación.

Contiene:

crearApp()

Responsabilidad:

Crear el elemento raíz:

main.app

y añadir el Layout.

📄 ui_fila.ts

Etapa: Construcción visual de una fila del perfil.

Contiene:

crearFila()

Responsabilidad:

Convertir un FilaPerfil en una fila visual.

Conecta:
FilaPerfil
    ↓
ContextoFila
    ↓
COLUMNAS
    ↓
Componentes UI
Conoce estas columnas:
numero
estado
app
trigger
tipo
accion
ejecucion
color
nota

Este archivo es el ensamblador de la fila.

No debería contener lógica profunda de cada control.

📋 ui_tabla.ts

Etapa: Construcción de la tabla.

Contiene:

crearTabla()

Responsabilidad:

Crear tabla.
Crear cabecera.
Crear viewport.
Crear contenedor de filas.
Recorrer perfil.filas.
Crear cada ui_fila.

También contiene las funciones internas de:

reconstruirTabla()
reconstruirFila()

Importante:

Es el lugar donde el modelo temporal se transforma en UI visible.

📋 ui_tabla_control.ts

Etapa: Control externo de reconstrucción de la UI.

Contiene:

registrarReconstruccion()
reconstruirTabla()
reconstruirFila()

Responsabilidad:

Permitir que otros componentes digan:

"reconstruye esta fila"

o:

"reconstruye toda la tabla"

sin conocer internamente cómo se construye la tabla.

📂 src/ui/componentes/
⌨️🖱️ comp_capturador.ts

Etapa: Captura de teclado y mouse para crear un Trigger.

Responsabilidad actual:

Crear botón capturador.
Mostrar trigger existente.
Mostrar botón +.
Abrir modificadores.
Capturar eventos de teclado.
Capturar eventos de mouse.
Capturar rueda.
Construir timeline.
Finalizar captura.
Pasar timeline a analizarCaptura().
Guardar resultado en:
filaPerfil.trigger

o:

filaPerfil.accion

Importante:

Actualmente este archivo fue dividido en dos archivos manejables.

comp_capturador.ts ya no debería volver a crecer hasta 800+ líneas.

🎛️ comp_accion.ts

Etapa: Construcción del control de acción.

Responsabilidad:

Decidir qué componente visual aparece en la columna Acción.

Actualmente:
Multimedia
    ↓
crearAccionMultimedia()

Macro
    ↓
crearAccionMacro()

Click coordenada
    ↓
crearAccionCoordenada()

Teclado / Mouse
    ↓
crearCapturador(..., "Accion")

Importante:

Aquí está la decisión:

“Qué control representa la acción según el tipo”.

🎛️ comp_accion_contenido.ts

Etapa: Contenido visual de acciones especiales.

Contiene actualmente:

crearAccionTeclado()
crearAccionMultimedia()
crearAccionMacro()
crearAccionCoordenada()

Responsabilidad:

Crear botones visuales simples.

No contiene:

Captura.
Runtime.
Ejecución.
🎛️ comp_controles.ts

Etapa: Controles simples de la fila.

Contiene:

crearEstado()
crearCondicion()
crearTipo()
crearNota()
crearEjecucion()
crearApp()
crearColor()

Responsabilidad:

Crear controles UI y modificar directamente:

filaPerfil

que pertenece al:

perfil temporal

Regla importante:

Estos cambios todavía no llegan al Runtime.

🪟 comp_popup_abrir.ts

Etapa: Opciones de los popups.

Contiene:

abrirPopupCondicion()
abrirPopupTipo()
abrirPopupEstado()
abrirPopupApp()
abrirPopupColor()
abrirPopupEjecucion()
abrirPopupModificador()

Responsabilidad:

Definir:

qué opciones aparecen

y:

qué ocurre al seleccionar una opción
Importante

Aquí está actualmente la lista:

Normal
Turbo
Mantener

para ejecución.

Y aquí también se agregan modificadores:

Win
Ctrl
Shift
Alt
🖼️ comp_popup.ts

Etapa: Componente visual genérico de popup.

Responsabilidad conocida:

Crear el botón/control que abre un popup.

🪟 comp_popup_contenedor.ts

Etapa: Contenedor global de popups.

Responsabilidad conocida:

Mostrar popup.
Ocultar popup.


---------------------------------------------------------------


🟩 BACKEND — src-tauri/src/
🚀 main.rs

Etapa: Entrada ejecutable.

Responsabilidad:

remaph_lib::run();

No contiene lógica.

🚀 lib.rs

Etapa: Entrada principal del backend Tauri.

Responsabilidad:

Declarar módulos.
Iniciar Runtime.
Iniciar entrada física.
Crear Tauri.
Registrar comandos.
Flujo actual:
run()
 ├── runtime::iniciar()
 ├── entrada::iniciar()
 └── Tauri
📦 modelos.rs

Etapa: Modelos internos compartidos de Rust.

Contiene:

Remapeo
Accion
Modelo actual:
Remapeo
├── trigger: Evento
└── accion: Accion
Accion actual:
Tecla(String)
Mouse(String)

Importante:

Este es el modelo que usa actualmente el Backend.

📦 eventos.rs

Etapa: Eventos internos del Backend.

Contiene:

Evento::Teclado
Evento::Mouse
Modelo actual:
Teclado
├── tecla
└── presionado

Mouse
├── boton
└── presionado

Regla:

Es independiente de Interception.

👤 usuario.rs

Etapa: Configuración editable del usuario en Rust.

Contiene:

Configuracion

con:

remapeos: Vec<Remapeo>

Responsabilidad conceptual acordada:

Propietario de la configuración editable del usuario.

🔨 compilador.rs

Etapa: Compilación de configuraciones.

Responsabilidad:

Remapeo
    ↓
Normalización
    ↓
Remapeo compilado
    ↓
Cache

Actualmente normaliza:

Eventos de teclado.
Eventos de mouse.
Acciones.

No debería:

Ejecutar acciones.
Capturar hardware.
Ser consultado por Runtime.
🧠 cache.rs

Etapa: Caché de remapeos compilados.

Responsabilidad:

Almacenar remapeos compilados.
Reemplazar el conjunto completo.
Buscar por evento.
Modelo:
HashMap<String, Remapeo>

Propietario conceptual:

cache.rs es dueño de los remapeos compilados.

🚀 runtime.rs

Etapa: Motor de ejecución.

Responsabilidad actual:

Recibir evento.
Buscar coincidencia en cache.
Ejecutar acción.
Devolver si el evento fue consumido.
Evento
 ↓
Cache
 ↓
Acción

Regla arquitectónica acordada:

Runtime no interpreta configuración.

No debería:

Leer JSON.
Normalizar.
Entender configuraciones de usuario.
Decidir qué significa un modo.
Compilar remapeos.
🖱️ entrada.rs

Etapa: Bucle principal de entrada física.

Responsabilidad:

Inicializar Runtime.
Crear Interception.
Recibir eventos.
Consultar reentrada.
Consultar captura.
Traducir eventos.
Pasar eventos al Runtime.
Ejecutar salidas.
Reenviar eventos no bloqueados.
Flujo actual:
Interception
    ↓
entrada.rs
    ↓
captura
    ↓
runtime
    ↓
salida

Este archivo es el coordinador del flujo físico.

📂 src-tauri/src/backend/
🖱️ back_interception.rs

Etapa: Adaptador físico de Interception.

Responsabilidad:

Inicializar Interception.
Configurar filtros.
Recibir Stroke.
Enviar teclas.
Reenviar eventos.
Identificar teclado/mouse.
Traducir Stroke a Evento.

Importante:

Aquí se habla directamente con:

Interception
🎹 back_teclas.rs

Etapa: Adaptador físico de teclado.

Responsabilidad:

ScanCode
    ↓
Evento::Teclado

También convierte:

String
    ↓
ScanCode

para salida.

🖱️ back_mouse.rs

Etapa: Adaptador físico de mouse.

Responsabilidad:

Convertir:

MouseFilter
MouseFlags
rolling

en:

Evento::Mouse

Actualmente reconoce:

WHEEL_UP
WHEEL_DOWN

y botones físicos.

🖱️ back_salida.rs

Etapa: Ejecución física de acciones.

Responsabilidad:

Recibir:

Accion

y producir salida física.

Actualmente:

Accion::Tecla
    ↓
back_teclas
    ↓
Interception

También controla:

reentrada::bloquear()
reentrada::liberar()

Importante:

Aquí se ejecuta la salida física.

🔄 Flujo actual de datos
UI
Usuario edita
    ↓
FilaPerfil
    ↓
Perfil temporal

Aquí termina actualmente el cambio de UI.

Guardado futuro
Perfil temporal
    ↓
Botón principal
"Perfil en pausa, ¿guardar cambios?"
    ↓
JSON
    ↓
Rust
Backend
Configuración de usuario
    ↓
usuario.rs
    ↓
compilador.rs
    ↓
cache.rs
    ↓
runtime.rs
    ↓
back_salida.rs
    ↓
Interception