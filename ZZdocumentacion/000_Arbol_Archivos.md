---

## Arbol de archivos

---

## 🧭 Arquitectura general

RemapH V3 está dividido en cuatro capas conceptuales:

UI
│
▼
Core
│
▼
Runtime
│
▼
Platform
│
▼
Windows / Hardware

La separación por lenguaje no define la arquitectura.

TypeScript contiene UI y Core.

Rust contiene Runtime y Platform.

La arquitectura se define por responsabilidad.

---

🟦 UI — src/ui/
🟦 CORE — src/core/

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
Captura física.

📂 src/core/

📄 core_perfil.ts

Etapa: Modelo editable del perfil.

Contiene:

Perfil
FilaPerfil
crearFila()
crearPerfil()
clonarFila()

Responsabilidad:

Representar la configuración que la UI está editando.

Perfil contiene:

activo
filas

FilaPerfil contiene:

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

Este modelo representa la intención editable del usuario.

No es el modelo compilado del Runtime.

📄 core_perfil_ui.ts

Etapa: Estado vivo de edición de la UI.

Contiene:

obtenerPerfilUi()
establecerPerfilUi()

Responsabilidad:

Mantener el Perfil que la interfaz está editando actualmente.

Flujo:

PerfilJson
↓
Conversión en Tauri
↓
Perfil UI

La UI modifica este perfil.

El Runtime no ve estos cambios automáticamente.

📄 core_perfil_acciones.ts

Etapa: Operaciones sobre el Perfil UI.

Contiene:

clonarFilaPorId()

Responsabilidad:

Buscar filas dentro del Perfil UI y ejecutar operaciones sobre ellas.

Actualmente:

Clonar fila.

📄 core_contexto_fila.ts

Etapa: Identidad de una fila.

Contiene:

ContextoFila
crearContextoFila()

Responsabilidad:

Transportar el ID de una fila hacia sus componentes internos.

Regla:

Todos los componentes de una fila reciben el mismo ContextoFila.

📄 core_entrada.ts

Etapa: Modelo canónico de entradas.

Contiene:

TipoEntrada
Entrada
crearEntrada()

Responsabilidad:

Representar entradas físicas de forma común.

La UI utiliza nombres propios del DOM.

La captura los convierte al idioma canónico de RemapH.

Ejemplos:

ControlLeft
↓
LeftControl

KeyA
↓
A

Mouse 2
↓
RightButton

📄 core_evento_captura.ts

Etapa: Modelo de eventos durante captura.

Responsabilidad:

Representar los eventos de una captura.

Ejemplo:

Entrada
↓
Down / Up
↓
EventoBuffer

📄 core_analizar_trigger.ts

Etapa: Análisis de la captura.

Responsabilidad:

Recibir el bufferEventos de eventos capturados.

Analizar:

• Orden.
• Down / Up.
• Modificadores.
• Gatillo.

Resultado:

Trigger.

📄 core_configuracion_captura.ts

Etapa: Configuración de captura.

Responsabilidad:

Centralizar los valores utilizados por el sistema de captura.

📄 core_normalizar_trigger.ts

Etapa: Normalización del Trigger.

Responsabilidad:

Normalizar la representación de triggers antes de su uso.

📄 core_trigger.ts

Etapa: Modelo y representación visual de triggers.

Contiene:

CondicionTrigger
Trigger
crearTrigger()
triggerATexto()
triggerAHTML()

Modelo:

Trigger
├── modificadores[]
├── gatillo
└── condicion

Responsabilidad:

Representar triggers.
Convertirlos a texto.
Convertirlos a HTML visual.

📂 src/ui/

📄 ui_app.ts

Etapa: Raíz visual de la aplicación.

Responsabilidad:

Crear el Layout principal.

📄 ui_layout.ts

Etapa: Ensamblador de la pantalla principal.

Contiene:

Toolbar.
Tabla.
StatusBar.
Contenedor global de Popups.

Responsabilidad:

Construir la estructura principal de la UI.

📄 ui_columnas.ts

Etapa: Fuente única de verdad de columnas.

Contiene:

COLUMNAS

Define:

• ID.
• Título.
• Grupo.
• Ancho.

La cabecera y las filas utilizan esta misma definición.

📄 ui_fila.ts

Etapa: Ensamblador visual de una fila.

Flujo:

FilaPerfil
↓
ContextoFila
↓
COLUMNAS
↓
Componentes UI

Conoce las columnas:

numero
estado
app
trigger
tipo
accion
ejecucion
color
nota

Este archivo ensambla.

No contiene la lógica profunda de cada control.

📄 ui_tabla.ts

Etapa: Construcción de la tabla.

Responsabilidad:

Crear la tabla.
Crear la cabecera.
Crear el viewport.
Crear el contenedor de filas.
Recorrer Perfil UI.
Crear cada fila.

También contiene internamente:

reconstruirTabla()
reconstruirFila()

Además detecta interacción con controles de fila para notificar modificaciones.

📄 ui_tabla_control.ts

Etapa: Control externo de reconstrucción.

Contiene:

registrarReconstruccion()
reconstruirTabla()
reconstruirFila()

Responsabilidad:

Permitir que componentes soliciten la reconstrucción de una fila o de la tabla sin conocer cómo se construye internamente.

No contiene lógica de perfil.

📄 ui_redimension_columnas.ts

Etapa: Redimensionamiento de columnas.

Responsabilidad:

Gestionar el arrastre de los divisores de cabecera.

No modifica la lógica de filas.

📄 ui_toolbar.ts

Etapa: Barra superior.

Responsabilidad:

Mostrar:

• Nombre de la aplicación.
• Selector visual de perfil.
• Estado del perfil.
• Botón de nueva fila.
• Configuración.

En el estado actual de commit 004:

El sistema de perfiles completo todavía no está implementado en la UI.

📄 ui_statusbar.ts

Etapa: Barra inferior.

Responsabilidad:

Mostrar información contextual de la aplicación.

📂 src/ui/componentes/

⌨️ comp_capturador.ts

Etapa: Control visual de captura.

Responsabilidad:

Crear el botón capturador.
Mostrar el Trigger.
Mostrar modificadores.
Abrir el popup de modificadores.
Iniciar captura.
Guardar el resultado en:

filaPerfil.trigger

o:

filaPerfil.accion

La captura física fue separada en:

comp_capturador_trigger.ts

⌨️ comp_capturador_trigger.ts

Etapa: Captura de entradas de la UI.

Responsabilidad:

Escuchar:

• Teclado.
• Mouse.
• Rueda.

Construir un bufferEventos de eventos.

Convertir nombres DOM al idioma canónico de RemapH.

Analizar trigger.

Entregar un Trigger.

⚙️ comp_accion.ts

Etapa: Selector visual de Acción.

Responsabilidad:

Decidir qué componente visual representa la acción según filaPerfil.tipo.

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

Otros tipos
↓
crearCapturador(..., "Accion")

⚙️ comp_accion_contenido.ts

Etapa: Contenido visual de acciones especiales.

Contiene:

crearAccionTeclado()
crearAccionMultimedia()
crearAccionMacro()
crearAccionCoordenada()

No contiene captura física ni Runtime.

🎛️ comp_controles.ts

Etapa: Controles simples de fila.

Contiene:

crearEstado()
crearCondicion()
crearTipo()
crearNota()
crearEjecucion()
crearApp()
crearColor()

Responsabilidad:

Crear controles visuales.

Modificar el FilaPerfil correspondiente.

Solicitar reconstrucción visual cuando corresponde.

🪟 comp_popup_abrir.ts

Etapa: Opciones de los Popups.

Contiene actualmente:

abrirPopupCondicion()
abrirPopupTipo()
abrirPopupEstado()
abrirPopupApp()
abrirPopupColor()
abrirPopupEjecucion()
abrirPopupModificador()

Responsabilidad:

Definir las opciones disponibles y aplicar el resultado al modelo de fila.

🖼️ comp_popup.ts

Etapa: Control visual genérico.

Responsabilidad:

Crear controles que abren Popups.

🪟 comp_popup_contenedor.ts

Etapa: Contenedor global de Popups.

Responsabilidad:

Mostrar Popup.
Ocultar Popup.

---

🟩 RUNTIME / PLATFORM — src-tauri/src/

🚀 main.rs

Etapa: Entrada ejecutable.

Responsabilidad:

Llamar a:

remaph_lib::run();

🚀 lib.rs

Etapa: Entrada principal de Tauri.

Responsabilidad:

Declarar módulos.

Iniciar la entrada física.

Crear Tauri.

Registrar comandos.

No contiene la lógica del Runtime.

📦 eventos.rs

Etapa: Modelo de entrada física genérica.

Contiene:

InputId
InputState
InputEvent

InputId identifica:

fuente
control

Ejemplo:

keyboard:A
mouse:WheelUp

InputState:

Down
Up
Pulse

Regla:

El Runtime recibe InputEvent.

No recibe directamente eventos de Interception ni Windows.

📦 perfiljson.rs

Etapa: Modelo persistente.

Contiene:

PerfilJson
RemapeoJson
TriggerJson

Representa el perfil almacenado en JSON.

No es el modelo del Runtime.

📦 perfilcache.rs

Etapa: Modelo compilado.

Contiene:

RemapeoCache
TriggerCache
AccionCache

No se serializa.

No conoce JSON.

No conoce la UI.

⚙️ compilador.rs

Etapa: Compilación de perfiles.

Flujo:

PerfilJson
↓
Compilador
↓
PerfilCache
↓
Cache

Actualmente:

• Ignora remapeos OFF.
• Extrae gatillos.
• Convierte entradas a InputId.
• Construye acciones compiladas.

🧠 cache.rs

Etapa: Caché de remapeos compilados.

Responsabilidad:

Almacenar RemapeoCache.

Funciones conceptuales:

Reemplazar cache.
Borrar cache.
Buscar Trigger exacto.
Buscar Pulse.
Detectar prefijos de Trigger.

La Cache no conoce:

• UI.
• JSON.
• Windows.

⚙️ runtime.rs

Etapa: Motor de ejecución.

Responsabilidad:

Recibir InputEvent.

Mantener:

• Inputs activos.
• Inputs consumidos.

Consultar Cache.

Determinar:

Pasar.
Esperar.
Consumir.

Flujo:

InputEvent
↓
Runtime
↓
Cache
↓
AccionCache

👤 usuario.rs

Etapa: Propietario de archivos de usuario.

Responsabilidad:

Resolver la carpeta:

APPDATA
↓
RemapH V3
↓
Usuario

Buscar perfiles JSON.

Determinar el perfil actual.

No compila.

No conoce Runtime.

💾 persistencia.rs

Etapa: Lectura y escritura de PerfilJson.

Responsabilidad:

Guardar JSON.
Cargar JSON.

No decide rutas.

No compila.

No toca Cache.

📡 comandos.rs

Etapa: Puente Tauri.

Responsabilidad:

Recibir datos de la UI.

Convertirlos a PerfilJson.

Exponer comandos para:

• Compilar perfil.
• Activar perfil.
• Desactivar perfil.
• Iniciar captura.
• Obtener captura.
• Obtener perfil actual.

Flujo:

UI
↓
Tauri
↓
PerfilJson
↓
Persistencia / Compilador

📊 estado.rs

Etapa: Estado global del perfil.

Responsabilidad:

Mantener si el perfil está activo.

El Runtime consulta este estado.

No conoce Cache.

🖱️ entrada.rs

Etapa: Coordinador de entrada física.

Soporta dos modos:

Full
Portable

Ambos entregan InputEvent genérico al Runtime.

Entrada no interpreta remapeos.

Entrada no compila configuraciones.

Entrada no ejecuta acciones directamente.

📂 src-tauri/src/backend/

🧩 back_interception.rs

Etapa: Backend de entrada Full.

Responsabilidad:

Inicializar Interception.

Recibir Strokes.

Traducirlos a InputEvent.

Reenviar eventos originales.

No conoce el Runtime.

🪟 back_windows.rs

Etapa: Backend Portable.

Utiliza:

WH_KEYBOARD_LL
WH_MOUSE_LL
SendInput

Responsabilidad:

Capturar eventos físicos.

Convertirlos a InputEvent.

Emitir eventos físicos.

No conoce Cache ni Runtime.

⌨️ back_teclas.rs

Etapa: Conversión de teclado.

Responsabilidad:

Convertir códigos físicos a InputId.

Convertir InputId a salida de teclado.

🖱️ back_mouse.rs

Etapa: Conversión de mouse.

Responsabilidad:

Convertir eventos físicos de mouse a InputEvent.

Convertir InputId a salida de mouse.

▶️ back_salida.rs

Etapa: Salida física Full.

Responsabilidad:

Recibir AccionCache.

Traducirla a:

• Teclado.
• Mouse.

Emitir mediante Interception.

🔄 Flujo actual de datos

UI
↓
Perfil UI
↓
Tauri
↓
PerfilJson
↓
Persistencia
↓
Compilador
↓
PerfilCache
↓
Cache
↓
Runtime
↓
AccionCache
↓
Platform
↓
Windows / Hardware

Entrada:

Hardware
↓
Platform
↓
InputEvent
↓
Runtime
↓
AccionCache
↓
Platform
↓
Hardware
-----